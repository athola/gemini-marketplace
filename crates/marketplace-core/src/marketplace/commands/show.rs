//! Handler for the `show` command.

use std::io::{self, Write};

use anyhow::Result;
use serde::Serialize;

use crate::marketplace::cache::init;
use crate::marketplace::config::Config;
use crate::marketplace::error::{MarketplaceError, Result as MarketplaceResult};
use crate::marketplace::models::api::ExtensionRecord;
use crate::marketplace::models::domain::{Extension, MarketplaceSource};
use crate::marketplace::services::catalog::{default_preferences, default_sources};
use crate::marketplace::services::source_fetcher::SourceFetcher;
use crate::marketplace::status::StatusStore;

/// Options for the `show` command.
#[derive(Debug, Clone)]
pub struct ShowOptions {
    pub id: String,
    pub json: bool,
}

#[derive(Debug, Serialize)]
struct ShowResponse<'a> {
    extension: ExtensionRecord,
    warnings: &'a [String],
    used_cached_data: bool,
}

struct FetchOutcome {
    extensions: Vec<Extension>,
    warnings: Vec<String>,
    used_cached_data: bool,
}

pub async fn execute(opts: ShowOptions) -> Result<()> {
    let (source_slug, extension_slug) = parse_id(&opts.id)?;

    let config = Config::new()?;
    init::ensure_layout(&config)?;

    let prefs = default_preferences();
    let sources = default_sources();
    let fetcher = SourceFetcher::new(&config, prefs.clone())?;
    let status_store = StatusStore::new(&config);

    let source = sources
        .iter()
        .find(|src| src.slug == source_slug)
        .ok_or_else(|| MarketplaceError::SourceNotFound {
            slug: source_slug.clone(),
        })?;

    let outcome = fetch_source_with_cache(&fetcher, &status_store, source).await?;

    let extension = outcome
        .extensions
        .iter()
        .find(|ext| ext.extension_slug == extension_slug || ext.id.0 == opts.id)
        .ok_or_else(|| MarketplaceError::ExtensionNotFound {
            id: opts.id.clone(),
        })?;

    if opts.json {
        let payload = ShowResponse {
            extension: ExtensionRecord::from(extension),
            warnings: &outcome.warnings,
            used_cached_data: outcome.used_cached_data,
        };
        let json = serde_json::to_string_pretty(&payload)?;
        println!("{json}");
        return Ok(());
    }

    render_human(extension, &outcome)?;
    Ok(())
}

fn parse_id(id: &str) -> MarketplaceResult<(String, String)> {
    let mut parts = id.splitn(2, '/');
    let source = parts.next().unwrap_or_default().trim();
    let extension = parts.next().unwrap_or_default().trim();
    if source.is_empty() || extension.is_empty() {
        return Err(MarketplaceError::Configuration(format!(
            "invalid extension id '{id}', expected SOURCE/EXTENSION"
        )));
    }
    Ok((source.to_string(), extension.to_string()))
}

async fn fetch_source_with_cache(
    fetcher: &SourceFetcher,
    status_store: &StatusStore,
    source: &MarketplaceSource,
) -> MarketplaceResult<FetchOutcome> {
    match fetcher.sync_source(source).await {
        Ok(extensions) => {
            let _ = status_store.clear_error(&source.slug);
            Ok(FetchOutcome {
                extensions,
                warnings: Vec::new(),
                used_cached_data: false,
            })
        }
        Err(err) => match err {
            MarketplaceError::RateLimited {
                source_slug,
                reset_at,
            } => {
                let mut warning = format!("Source {source_slug} is rate limited");
                if let Some(ts) = &reset_at {
                    warning.push_str(&format!(" (resets at {ts})"));
                }
                let _ = status_store.record_error(&source.slug, &warning);
                match fetcher.cached_extensions(&source.slug) {
                    Ok(Some(data)) => Ok(FetchOutcome {
                        extensions: data,
                        warnings: vec![warning],
                        used_cached_data: true,
                    }),
                    Ok(None) => Err(MarketplaceError::RateLimited {
                        source_slug,
                        reset_at,
                    }),
                    Err(cache_err) => Err(cache_err),
                }
            }
            MarketplaceError::Network {
                operation,
                source: cause,
                url,
            } => {
                let mut warning = format!(
                    "Network error for {} during {}: {}",
                    source.slug, operation, cause
                );
                if let Some(url) = &url {
                    warning.push_str(&format!(" ({url})"));
                }
                let _ = status_store.record_error(&source.slug, &warning);
                match fetcher.cached_extensions(&source.slug) {
                    Ok(Some(data)) => Ok(FetchOutcome {
                        extensions: data,
                        warnings: vec![warning],
                        used_cached_data: true,
                    }),
                    Ok(None) => Err(MarketplaceError::Network {
                        operation,
                        source: cause,
                        url,
                    }),
                    Err(cache_err) => Err(cache_err),
                }
            }
            other => {
                let _ = status_store.record_error(&source.slug, other.to_string());
                Err(other)
            }
        },
    }
}

fn render_human(extension: &Extension, outcome: &FetchOutcome) -> Result<()> {
    for warning in &outcome.warnings {
        writeln!(io::stderr(), "warning: {warning}")?;
    }

    println!(
        "Extension: {} ({})",
        extension.display_name, extension.version
    );
    println!("Namespace: {}", extension.id.0);
    println!("Source: {}", extension.source_slug);
    println!("Author: {}", extension.author);
    println!("Description: {}", extension.summary);
    println!("Repository: {}", extension.repository_url);
    if let Some(homepage) = &extension.homepage_url {
        println!("Homepage: {homepage}");
    }
    if let Some(doc) = &extension.documentation_url {
        println!("Docs: {doc}");
    }
    if !extension.categories.is_empty() {
        println!("Categories: {}", extension.categories.join(", "));
    }
    if !extension.compatibility.is_empty() {
        println!("Compatibility: {}", extension.compatibility.join(", "));
    }
    if outcome.used_cached_data {
        println!("Note: using cached data while sources refresh.");
    }
    if let Some(readme) = &extension.readme_excerpt {
        println!("\nREADME (excerpt):\n{readme}");
    }

    Ok(())
}
