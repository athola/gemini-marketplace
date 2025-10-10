use std::collections::BTreeMap;
use std::io::{self, Write};

use anyhow::Result;
use serde::Serialize;

use crate::marketplace::config::Config;
use crate::marketplace::models::domain::SearchMode;
use crate::marketplace::services::catalog::{
    default_preferences, default_sources, CatalogService, ListEntry, ListRequest,
};
use crate::marketplace::services::source_fetcher::SourceFetcher;

#[derive(Debug, Clone)]
pub struct ListOptions {
    pub search: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub installed_only: bool,
    pub json: bool,
}

#[derive(Serialize)]
struct CliListResponse<'a> {
    entries: &'a [ListEntry],
    warnings: &'a [String],
}

pub async fn execute(opts: ListOptions) -> Result<()> {
    let config = Config::new()?;
    config.ensure_dirs()?;

    let prefs = default_preferences();
    let sources = default_sources();
    let fetcher = SourceFetcher::new(&config, prefs.clone())?;
    let service = CatalogService::new(fetcher, prefs, sources);

    let request = ListRequest {
        search: opts.search.as_deref(),
        category: opts.category.as_deref(),
        source: opts.source.as_deref(),
        installed_only: opts.installed_only,
        prefetch_filter: matches!(service.preferences().search_mode(), SearchMode::PreFetchFilter),
    };

    let response = service.list(&request).await?;

    for warning in &response.warnings {
        writeln!(io::stderr(), "warning: {warning}")?;
    }

    if opts.json {
        let payload = CliListResponse {
            entries: &response.entries,
            warnings: &response.warnings,
        };
        let json = serde_json::to_string_pretty(&payload)?;
        println!("{json}");
        return Ok(());
    }

    render_table(&response.entries);
    Ok(())
}

fn render_table(entries: &[ListEntry]) {
    if entries.is_empty() {
        println!("No extensions found.");
        return;
    }

    let mut grouped: BTreeMap<&str, Vec<&ListEntry>> = BTreeMap::new();
    for entry in entries {
        grouped.entry(&entry.source).or_default().push(entry);
    }

    for (source, items) in grouped {
        println!("Source: {source}");
        println!("{:<40} {:<}", "Extension", "Description");
        println!("{:-<80}", "");
        for item in items {
            println!(
                "{:<40} {}",
                format!("{} ({})", item.display_name, item.version),
                item.description
            );
        }
        println!();
    }
}
*** End Patch
