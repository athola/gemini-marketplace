//! Catalog service responsible for aggregating extensions across sources,
//! applying filtering, and translating results for CLI/API consumers.

use anyhow::Result;
use serde::Serialize;

use crate::marketplace::error::MarketplaceError;
use crate::marketplace::models::domain::{
    Extension, InstallStatus, MarketplaceSource, OutputFormat, SearchMode,
};
use crate::marketplace::services::preferences::PreferencesService;
use crate::marketplace::services::source_fetcher::SourceFetcher;

const DEFAULT_PAGE_SIZE: usize = 25;

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
    pub page: Option<usize>,
    pub page_size: Option<usize>,
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
    pub page: usize,
    pub page_size: usize,
    pub total_entries: usize,
    pub total_pages: usize,
    pub used_cached_data: bool,
}

impl CatalogService {
    pub fn new(
        fetcher: SourceFetcher,
        prefs: PreferencesService,
        sources: Vec<MarketplaceSource>,
    ) -> Self {
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
        let mut used_cached_data = false;

        for source in self.sources.iter().filter(|src| src.enabled) {
            match self.fetch_source(source, request.prefetch_filter).await? {
                FetchResult::Fresh(extensions) => entries.extend(make_entries(source, extensions)),
                FetchResult::Cached {
                    extensions,
                    warning,
                } => {
                    used_cached_data = true;
                    warnings.push(warning);
                    entries.extend(make_entries(source, extensions));
                }
            }
        }

        let filtered = filter_entries(entries, request);
        let total_entries = filtered.len();
        let page_size = request
            .page_size
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .max(1);
        let total_pages = ((total_entries + page_size - 1) / page_size).max(1);
        let requested_page = request.page.unwrap_or(1).max(1);
        let page = requested_page.min(total_pages);
        let start = page_size.saturating_mul(page.saturating_sub(1));
        let paginated: Vec<ListEntry> = filtered
            .into_iter()
            .skip(start)
            .take(page_size)
            .collect();
        Ok(ListResponse {
            entries: paginated,
            warnings,
            page,
            page_size,
            total_entries,
            total_pages,
            used_cached_data,
        })
    }

    async fn fetch_source(
        &self,
        source: &MarketplaceSource,
        _prefetch_filter: bool,
    ) -> std::result::Result<FetchResult, MarketplaceError> {
        match self.fetcher.sync_source(source).await {
            Ok(list) => Ok(FetchResult::Fresh(list)),
            Err(err) => match err {
                MarketplaceError::RateLimited {
                    source_slug,
                    reset_at,
                } => {
                    let mut warning = format!("Source {source_slug} is rate limited");
                    if let Some(ts) = &reset_at {
                        warning.push_str(&format!(" (resets at {ts})"));
                    }
                    match self.fetcher.cached_extensions(&source.slug) {
                        Ok(Some(data)) => Ok(FetchResult::Cached {
                            extensions: data,
                            warning,
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
                    let warning_operation = operation.clone();
                    let warning_url = url.clone();
                    let cause_msg = cause.to_string();
                    let mut warning = format!(
                        "Network error for {} during {}: {}",
                        source.slug, warning_operation, cause_msg
                    );
                    if let Some(url_str) = warning_url.as_ref() {
                        warning.push_str(&format!(" ({url_str})"));
                    }
                    match self.fetcher.cached_extensions(&source.slug) {
                        Ok(Some(data)) => Ok(FetchResult::Cached {
                            extensions: data,
                            warning,
                        }),
                        Ok(None) => Err(MarketplaceError::network(operation, cause, url)),
                        Err(cache_err) => Err(cache_err),
                    }
                }
                other => Err(other),
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
                let slug_matches = entry.source.to_lowercase() == *source_filter;
                let namespace_matches = entry
                    .namespace
                    .to_lowercase()
                    .starts_with(&format!("{source_filter}/"));
                if !slug_matches && !namespace_matches {
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
                display_name: ext.display_name.into_owned(),
                description: ext.summary.into_owned(),
                version: ext.version.to_string(),
                source: source.slug.clone(),
                installed,
                categories: ext.categories,
                tags: ext.tags,
            }
        })
        .collect()
}

enum FetchResult {
    Fresh(Vec<Extension>),
    Cached {
        extensions: Vec<Extension>,
        warning: String,
    },
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
    let base = std::env::var("GEMINI_MARKETPLACE_SOURCE_URL").unwrap_or_else(|_| {
        "https://raw.githubusercontent.com/athola/gemini-marketplace/main/index.json".to_string()
    });
    let url = base.parse().unwrap_or_else(|_| {
        "https://raw.githubusercontent.com/athola/gemini-marketplace/main/index.json"
            .parse()
            .expect("Default curated source URL should be valid")
    });
    vec![MarketplaceSource::default_curated(url)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::StatusCode, routing::get, Router};
    use semver::Version;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};
    use tempfile::TempDir;
    use tokio::net::TcpListener;
    use url::Url;

    use crate::marketplace::cache::store::CacheStore;
    use crate::marketplace::config::Config;
    use crate::marketplace::models::domain::{
        Extension, ExtensionId, InstallStatus, MarketplaceSource, OutputFormat, SearchMode,
        UserPreferences, ValidationSummary,
    };
    use crate::marketplace::services::preferences::PreferencesService;
    use crate::marketplace::services::source_fetcher::SourceFetcher;
    use crate::marketplace::services::sources::tests::env_lock;

    fn sample_extension(source: &str, slug: &str) -> Extension {
        Extension::new(
            ExtensionId::new(source, slug),
            format!("{slug} Extension"),
            "Cached copy",
            Url::parse("https://example.com/repo").unwrap(),
            Version::parse("1.0.0").unwrap(),
            "Tester",
            source,
            ["utilities"],
            ["Gemini CLI >=1.0"],
            InstallStatus::NotInstalled,
            Some(SystemTime::UNIX_EPOCH),
            ValidationSummary::new_pending(SystemTime::UNIX_EPOCH),
            format!("repos/{slug}"),
            Some("README".into()),
        )
    }

    async fn rate_limited() -> axum::response::Response {
        let reset_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 30;
        axum::response::Response::builder()
            .status(StatusCode::FORBIDDEN)
            .header("x-ratelimit-reset", reset_at.to_string())
            .header("x-ratelimit-remaining", "0")
            .body(Body::from("rate limited"))
            .unwrap()
    }

    #[tokio::test]
    async fn list_returns_cached_entries_on_rate_limit() {
        let _guard = env_lock().lock().unwrap();
        let temp = TempDir::new().expect("temp dir");
        let home = temp.path().to_path_buf();
        std::env::set_var("GEMINI_MARKETPLACE_HOME", &home);

        let source_router = Router::new().route("/catalog.json", get(rate_limited));
        let catalog_listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let catalog_addr = catalog_listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(catalog_listener, source_router.into_make_service())
                .await
                .expect("source server");
        });

        let config = Config::new().expect("config");
        config.ensure_dirs().expect("dirs");

        let prefs = PreferencesService::new(UserPreferences {
            cache_ttl_hours: 1,
            auto_refresh_on_launch: false,
            search_mode: SearchMode::LocalFilter,
            output_format: OutputFormat::Table,
        });

        // Seed cache with data so the service can return something when rate-limited.
        let cache = CacheStore::new(&config).expect("cache store");
        let cached_extension = sample_extension("rate-limited", "cached");
        cache
            .save(
                "rate-limited",
                &[cached_extension.clone()],
                Some("etag".to_string()),
                Duration::from_millis(1),
            )
            .expect("save cache");

        // Ensure the cached entry appears expired so the fetcher attempts a refresh.
        thread::sleep(Duration::from_millis(10));

        let fetcher = SourceFetcher::new(&config, prefs.clone()).expect("fetcher");
        let source = MarketplaceSource::new(
            "rate-limited",
            "Rate Limited",
            format!("http://{}/catalog.json", catalog_addr)
                .parse()
                .expect("url"),
            crate::marketplace::models::domain::SourceType::GithubRepo,
            true,
            1,
        );
        let catalog = CatalogService::new(fetcher, prefs, vec![source]);
        let response = catalog
            .list(&ListRequest {
                search: None,
                category: None,
                source: None,
                installed_only: false,
                prefetch_filter: false,
                page: None,
                page_size: None,
            })
            .await
            .expect("list");

        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].namespace, cached_extension.id.0);
        assert!(response
            .warnings
            .iter()
            .any(|warning| warning.contains("rate limited")));

        std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    }
}
