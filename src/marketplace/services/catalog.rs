//! Catalog service responsible for aggregating extensions across sources,
//! applying filtering, and translating results for CLI/API consumers.

use std::time::Duration;

use anyhow::Result;
use serde::Serialize;

use crate::marketplace::error::MarketplaceError;
use crate::marketplace::models::domain::{
    Extension, InstallStatus, MarketplaceSource, OutputFormat, SearchMode,
};
use crate::marketplace::services::preferences::PreferencesService;
use crate::marketplace::services::source_fetcher::SourceFetcher;

pub struct CatalogService {
    fetcher: SourceFetcher,
    prefs: PreferencesService,
    sources: Vec<MarketplaceSource>,
}

#[derive(Debug, Default)]
pub struct ListRequest<'a> {
    pub search: Option<&'a str>,
    pub category: Option<&'a str>,
    pub source: Option<&'a str>,
    pub installed_only: bool,
    pub prefetch_filter: bool,
}

#[derive(Debug, Serialize)]
pub struct ListEntry {
    pub namespace: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub source: String,
    pub installed: bool,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct ListResponse {
    pub entries: Vec<ListEntry>,
    pub warnings: Vec<String>,
}

impl CatalogService {
    pub fn new(fetcher: SourceFetcher, prefs: PreferencesService, sources: Vec<MarketplaceSource>) -> Self {
        Self {
            fetcher,
            prefs,
            sources,
        }
    }

    pub fn preferences(&self) -> &PreferencesService {
        &self.prefs
    }

    pub async fn list(&self, request: &ListRequest<'_>) -> Result<ListResponse> {
        let mut warnings = Vec::new();
        let mut entries = Vec::new();

        for source in self.sources.iter().filter(|src| src.enabled) {
            match self.fetch_source(source, request.prefetch_filter).await {
                Ok(extensions) => entries.extend(make_entries(source, extensions)),
                Err(SourceWarning::Cached { extensions, warning }) => {
                    warnings.push(warning);
                    entries.extend(make_entries(source, extensions));
                }
                Err(SourceWarning::Fatal { warning }) => warnings.push(warning),
            }
        }

        let filtered = filter_entries(entries, request);
        Ok(ListResponse {
            entries: filtered,
            warnings,
        })
    }

    async fn fetch_source(
        &self,
        source: &MarketplaceSource,
        _prefetch_filter: bool,
    ) -> Result<Vec<Extension>, SourceWarning> {
        match self.fetcher.sync_source(source).await {
            Ok(list) => Ok(list),
            Err(err) => match err {
                MarketplaceError::RateLimited { source: slug, reset_at } => {
                    let mut message = format!("Source {slug} is rate limited");
                    if let Some(ts) = reset_at {
                        message.push_str(&format!(" (resets at {ts})"));
                    }
                    let cached = self
                        .fetcher
                        .cached_extensions(&source.slug)
                        .map_err(|cache_err| SourceWarning::Fatal {
                            warning: format!(
                                "{}; additionally failed to read cache: {}",
                                message, cache_err
                            ),
                        })?;
                    cached.map_or_else(
                        || Err(SourceWarning::Fatal { warning: message }),
                        |data| Err(SourceWarning::Cached { extensions: data, warning: message }),
                    )
                }
                MarketplaceError::Network(detail) => {
                    let message = format!("Network error for {}: {}", source.slug, detail);
                    let cached = self
                        .fetcher
                        .cached_extensions(&source.slug)
                        .map_err(|cache_err| SourceWarning::Fatal {
                            warning: format!(
                                "{}; additionally failed to read cache: {}",
                                message, cache_err
                            ),
                        })?;
                    cached.map_or_else(
                        || Err(SourceWarning::Fatal { warning: message }),
                        |data| Err(SourceWarning::Cached { extensions: data, warning: message }),
                    )
                }
                other => Err(SourceWarning::Fatal {
                    warning: other.to_string(),
                }),
            },
        }
    }
}

fn filter_entries(entries: Vec<ListEntry>, request: &ListRequest<'_>) -> Vec<ListEntry> {
    let search = request.search.map(|s| s.to_lowercase());
    let category = request.category.map(|c| c.to_lowercase());
    let source = request.source.map(|s| s.to_lowercase());

    let mut filtered: Vec<ListEntry> = entries
        .into_iter()
        .filter(|entry| {
            if let Some(search) = &search {
                let name = entry.display_name.to_lowercase();
                let description = entry.description.to_lowercase();
                if !name.contains(search) && !description.contains(search) {
                    return false;
                }
            }
            if let Some(category) = &category {
                if !entry
                    .categories
                    .iter()
                    .any(|c| c.to_lowercase() == *category)
                {
                    return false;
                }
            }
            if let Some(source_filter) = &source {
                if entry.source.to_lowercase() != *source_filter
                    && !entry.namespace.to_lowercase().starts_with(source_filter)
                {
                    return false;
                }
            }
            if request.installed_only && !entry.installed {
                return false;
            }
            true
        })
        .collect();

    filtered.sort_by(|a, b| a.namespace.cmp(&b.namespace));
    filtered
}

fn make_entries(source: &MarketplaceSource, extensions: Vec<Extension>) -> Vec<ListEntry> {
    extensions
        .into_iter()
        .map(|ext| {
            let installed = matches!(
                ext.install_status,
                InstallStatus::Installed { .. } | InstallStatus::UpdateAvailable { .. }
            );
            ListEntry {
                namespace: ext.id.0,
                display_name: ext.display_name,
                description: ext.summary,
                version: ext.version.to_string(),
                source: source.slug.clone(),
                installed,
                categories: ext.categories,
                tags: ext.tags,
            }
        })
        .collect()
}

enum SourceWarning {
    Cached { extensions: Vec<Extension>, warning: String },
    Fatal { warning: String },
}

pub fn default_preferences() -> PreferencesService {
    PreferencesService::new(crate::marketplace::models::domain::UserPreferences {
        cache_ttl_hours: 24,
        auto_refresh_on_launch: false,
        search_mode: SearchMode::LocalFilter,
        output_format: OutputFormat::Table,
    })
}

pub fn default_sources() -> Vec<MarketplaceSource> {
    let base = std::env::var("GEMINI_MARKETPLACE_SOURCE_URL")
        .unwrap_or_else(|_| "https://raw.githubusercontent.com/athola/gemini-marketplace/main/index.json".to_string());
    let url = base.parse().unwrap_or_else(|_| "https://raw.githubusercontent.com/athola/gemini-marketplace/main/index.json".parse().unwrap());
    vec![MarketplaceSource::default_curated(url)]
}
*** End Patch
