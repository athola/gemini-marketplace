use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use humantime::format_rfc3339_seconds;
use reqwest::{Client, StatusCode};
use serde::Serialize;

use crate::marketplace::cache::init;
use crate::marketplace::config::Config;
use crate::marketplace::models::domain::{MarketplaceSource, RetryJob};
use crate::marketplace::services::sources::SourcesService;
use crate::marketplace::status::StatusStore;
use crate::telemetry::{Telemetry, TelemetrySnapshot};

#[derive(Debug, Clone)]
pub struct StatusOptions {
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct StatusReport {
    cache_dir_exists: bool,
    network: NetworkStatus,
    installed_extensions: InstalledSummary,
    refresh_queue: RefreshQueueStatus,
    last_errors: Vec<LastErrorView>,
    telemetry: TelemetrySnapshot,
}

#[derive(Debug, Serialize)]
struct NetworkStatus {
    reachable: bool,
    checked_source: Option<String>,
    message: Option<String>,
}

#[derive(Debug, Default, Serialize)]
struct InstalledSummary {
    total: usize,
    extensions: Vec<String>,
    installing: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RefreshQueueStatus {
    pending_jobs: usize,
    next_job_due: Option<String>,
}

#[derive(Debug, Serialize)]
struct LastErrorView {
    source_slug: String,
    message: String,
    occurred_at: String,
}

const NETWORK_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn execute(opts: StatusOptions) -> Result<()> {
    let config = Config::new()?;
    init::ensure_layout(&config)?;

    let cache_dir_exists = config.cache_dir().exists();

    let sources_service = SourcesService::new()?;
    let sources = sources_service.list_sources()?;

    let network = check_network(&sources).await;
    let installed_summary = installed_extensions(&config);
    let refresh_status = refresh_queue_status(&config);

    let status_store = StatusStore::new(&config);
    let snapshot = status_store.load().unwrap_or_else(|_| Default::default());
    let last_errors = snapshot
        .last_errors
        .iter()
        .map(|entry| LastErrorView {
            source_slug: entry.source_slug.clone(),
            message: entry.message.clone(),
            occurred_at: format_rfc3339_seconds(entry.occurred_at).to_string(),
        })
        .collect::<Vec<_>>();

    let report = StatusReport {
        cache_dir_exists,
        network,
        installed_extensions: installed_summary,
        refresh_queue: refresh_status,
        last_errors,
        telemetry: Telemetry::global().snapshot(),
    };

    if opts.json {
        let json = serde_json::to_string_pretty(&report)?;
        println!("{json}");
        return Ok(());
    }

    render_human(&report);
    Ok(())
}

fn render_human(report: &StatusReport) {
    println!("Marketplace Status");
    println!("------------------");
    println!(
        "Cache directory: {}",
        if report.cache_dir_exists {
            "ready"
        } else {
            "missing"
        }
    );

    if report.network.reachable {
        match &report.network.checked_source {
            Some(slug) => println!("Network: OK (checked {slug})"),
            None => println!("Network: OK"),
        }
        if let Some(message) = &report.network.message {
            println!("  note: {message}");
        }
    } else {
        println!("Network: ERROR");
        if let Some(message) = &report.network.message {
            println!("  reason: {message}");
        }
    }

    println!(
        "Installed extensions: {}",
        report.installed_extensions.total
    );
    if !report.installed_extensions.extensions.is_empty() {
        println!("  {}", report.installed_extensions.extensions.join(", "));
    }
    if report.installed_extensions.installing.is_empty() {
        println!("Installs in progress: none");
    } else {
        println!(
            "Installs in progress: {}",
            report.installed_extensions.installing.join(", ")
        );
    }

    println!(
        "Refresh queue: {} pending",
        report.refresh_queue.pending_jobs
    );
    if let Some(next_due) = &report.refresh_queue.next_job_due {
        println!("  next run at {next_due}");
    }

    if report.last_errors.is_empty() {
        println!("Recent errors: none");
    } else {
        println!("Recent errors:");
        for entry in &report.last_errors {
            println!("  {} at {}", entry.source_slug, entry.occurred_at);
            println!("    {}", entry.message);
        }
    }

    println!("Telemetry:");
    println!(
        "  cache hits: {} | cache misses: {}",
        report.telemetry.cache_hits, report.telemetry.cache_misses
    );
    println!(
        "  rate-limit waits: {} | refresh queue depth: {}",
        report.telemetry.rate_limit_waits, report.telemetry.refresh_queue_depth
    );
    if report.telemetry.search_terms.is_empty() {
        println!("  top searches: none recorded");
    } else {
        let terms: Vec<_> = report
            .telemetry
            .search_terms
            .iter()
            .take(3)
            .map(|(term, count)| format!("{term} ({count})"))
            .collect();
        println!("  top searches: {}", terms.join(", "));
    }
}

async fn check_network(sources: &[MarketplaceSource]) -> NetworkStatus {
    let enabled_source = sources.iter().find(|source| source.enabled);
    let Some(source) = enabled_source else {
        return NetworkStatus {
            reachable: true,
            checked_source: None,
            message: Some("No sources configured; network check skipped".to_string()),
        };
    };

    let client = match Client::builder()
        .timeout(NETWORK_TIMEOUT)
        .user_agent("gemini-marketplace-extension/0.1.0")
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            return NetworkStatus {
                reachable: false,
                checked_source: Some(source.slug.clone()),
                message: Some(format!("Failed to create HTTP client: {err}")),
            };
        }
    };

    match client.head(source.url.as_str()).send().await {
        Ok(response) => {
            let status = response.status();
            let message = if status.is_success() || status == StatusCode::METHOD_NOT_ALLOWED {
                None
            } else {
                Some(format!("HTTP {status} from {}", source.url))
            };
            NetworkStatus {
                reachable: true,
                checked_source: Some(source.slug.clone()),
                message,
            }
        }
        Err(err) => NetworkStatus {
            reachable: false,
            checked_source: Some(source.slug.clone()),
            message: Some(err.to_string()),
        },
    }
}

fn installed_extensions(config: &Config) -> InstalledSummary {
    let mut summary = InstalledSummary::default();
    let Some(root) = config.extensions_root() else {
        return summary;
    };

    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return summary,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = match entry.file_name().to_str() {
            Some(name) if !name.starts_with('.') => name.to_string(),
            _ => continue,
        };

        if looks_installing(&path, &name) {
            summary.installing.push(name);
            continue;
        }

        if path.join(".gemini-extension-install.json").exists() {
            summary.extensions.push(name);
        }
    }

    summary.extensions.sort_unstable();
    summary.installing.sort_unstable();
    summary.total = summary.extensions.len();
    summary
}

fn looks_installing(path: &Path, name: &str) -> bool {
    if name.ends_with(".tmp") || name.contains("install") {
        return true;
    }
    path.join(".installing").exists()
}

fn refresh_queue_status(config: &Config) -> RefreshQueueStatus {
    let path = config.config_dir().join("refresh_queue.json");
    if !path.exists() {
        return RefreshQueueStatus {
            pending_jobs: 0,
            next_job_due: None,
        };
    }

    let data = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(_) => {
            return RefreshQueueStatus {
                pending_jobs: 0,
                next_job_due: None,
            };
        }
    };

    let queue: Vec<RetryJob> = serde_json::from_str(&data).unwrap_or_default();
    let next_due = queue
        .iter()
        .map(|job| job.scheduled_for)
        .min()
        .map(|time| format_rfc3339_seconds(time).to_string());

    RefreshQueueStatus {
        pending_jobs: queue.len(),
        next_job_due: next_due,
    }
}
