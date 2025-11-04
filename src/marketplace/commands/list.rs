use std::collections::BTreeMap;
use std::io::{self, Write};

use anyhow::Result;
use serde::Serialize;

use crate::marketplace::config::Config;
use crate::marketplace::models::domain::SearchMode;
use crate::marketplace::services::catalog::{
    default_preferences, default_sources, CatalogService, ListEntry, ListRequest, ListResponse,
};
use crate::marketplace::services::source_fetcher::SourceFetcher;
use crate::marketplace::status::StatusStore;

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
    page: usize,
    page_size: usize,
    total_entries: usize,
    total_pages: usize,
    used_cached_data: bool,
}

pub async fn execute(opts: ListOptions) -> Result<()> {
    let config = Config::new()?;
    config.ensure_dirs()?;

    let prefs = default_preferences();
    let sources = default_sources();
    let fetcher = SourceFetcher::new(&config, prefs.clone())?;
    let status_store = StatusStore::new(&config);
    let service = CatalogService::new(fetcher, prefs, sources, status_store);

    let request = ListRequest {
        search: opts.search.as_deref(),
        category: opts.category.as_deref(),
        source: opts.source.as_deref(),
        installed_only: opts.installed_only,
        prefetch_filter: matches!(
            service.preferences().search_mode(),
            SearchMode::PreFetchFilter
        ),
        page: None,
        page_size: None,
    };

    let response = service.list(&request).await?;

    for warning in &response.warnings {
        writeln!(io::stderr(), "warning: {warning}")?;
    }

    if opts.json {
        let payload = CliListResponse {
            entries: &response.entries,
            warnings: &response.warnings,
            page: response.page,
            page_size: response.page_size,
            total_entries: response.total_entries,
            total_pages: response.total_pages,
            used_cached_data: response.used_cached_data,
        };
        let json = serde_json::to_string_pretty(&payload)?;
        println!("{json}");
        return Ok(());
    }

    render_table(&response);
    Ok(())
}

fn render_table(response: &ListResponse) {
    println!(
        "Page {} of {} (showing {} of {} extensions)",
        response.page,
        response.total_pages,
        response.entries.len(),
        response.total_entries
    );

    if response.used_cached_data {
        println!("Note: using cached data while sources refresh.");
    }

    if response.entries.is_empty() {
        println!("No extensions found.");
        return;
    }

    let mut grouped: BTreeMap<&str, Vec<&ListEntry>> = BTreeMap::new();
    for entry in &response.entries {
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
