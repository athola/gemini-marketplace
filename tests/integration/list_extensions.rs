use std::env;

use gemini_marketplace::marketplace::config::Config;
use gemini_marketplace::marketplace::models::domain::{MarketplaceSource, OutputFormat, SearchMode, UserPreferences};
use gemini_marketplace::marketplace::services::catalog::{CatalogService, ListRequest};
use gemini_marketplace::marketplace::services::preferences::PreferencesService;
use gemini_marketplace::marketplace::services::source_fetcher::SourceFetcher;
use tempfile::tempdir;
use tokio::runtime::Runtime;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn prefs() -> PreferencesService {
    PreferencesService::new(UserPreferences {
        cache_ttl_hours: 1,
        auto_refresh_on_launch: false,
        search_mode: SearchMode::LocalFilter,
        output_format: OutputFormat::Table,
    })
}

fn source(url: Url) -> MarketplaceSource {
    MarketplaceSource::default_curated(url)
}

#[test]
fn catalog_service_lists_and_filters_extensions() -> anyhow::Result<()> {
    let rt = runtime();
    rt.block_on(async move {
        let server = MockServer::start().await;

        let manifests = vec![
            format!("{}/manifests/analytics.json", server.uri()),
            format!("{}/manifests/storage.json", server.uri()),
        ];
        Mock::given(method("GET"))
            .and(path("/index.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&manifests))
            .mount(&server)
            .await;

        let analytics_manifest = serde_json::json!({
            "name": "analytics",
            "displayName": "Analytics Toolkit",
            "description": "Tools for analytics workflows",
            "repository": "https://github.com/example/analytics",
            "version": "1.0.0",
            "author": "Example",
            "categories": ["Data Science"],
            "tags": ["analysis"],
            "compatibility": ["cli>=1.0"]
        });
        let storage_manifest = serde_json::json!({
            "name": "storage",
            "displayName": "Storage Helper",
            "description": "Manage storage buckets",
            "repository": "https://github.com/example/storage",
            "version": "2.0.0",
            "author": "Example",
            "categories": [" Infrastructure "],
            "tags": ["storage"],
            "compatibility": ["cli>=1.0"]
        });

        Mock::given(method("GET"))
            .and(path("/manifests/analytics.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&analytics_manifest))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/manifests/storage.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&storage_manifest))
            .mount(&server)
            .await;

        let temp = tempdir()?;
        env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
        let config = Config::new()?;
        config.ensure_dirs()?;

        let preferences = prefs();
        let fetcher = SourceFetcher::new(&config, preferences.clone())?;
        let sources = vec![source(Url::parse(&format!("{}/index.json", server.uri()))?)];
        let service = CatalogService::new(fetcher, preferences, sources);

        let request = ListRequest {
            search: Some("analytics"),
            category: None,
            source: None,
            installed_only: false,
            prefetch_filter: false,
        };

        let response = service.list(&request).await?;
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].namespace, "athola/analytics");

        let request = ListRequest {
            search: None,
            category: Some("infrastructure"),
            source: None,
            installed_only: false,
            prefetch_filter: false,
        };
        let response = service.list(&request).await?;
        assert_eq!(response.entries.len(), 1);
        assert_eq!(response.entries[0].namespace, "athola/storage");

        env::remove_var("GEMINI_MARKETPLACE_HOME");
        server.stop().await;
        Ok(())
    })
}

#[test]
fn catalog_service_returns_cached_entries_on_network_failure() -> anyhow::Result<()> {
    let rt = runtime();
    rt.block_on(async move {
        let server = MockServer::start().await;

        let manifests = vec![format!("{}/manifests/tool.json", server.uri())];
        Mock::given(method("GET"))
            .and(path("/index.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&manifests))
            .mount(&server)
            .await;

        let manifest = serde_json::json!({
            "name": "tool",
            "displayName": "Tool",
            "description": "Useful tool",
            "repository": "https://github.com/example/tool",
            "version": "1.0.0",
            "author": "Example",
            "categories": ["Utility Tools"],
            "tags": ["tool"],
            "compatibility": ["cli>=1.0"]
        });

        Mock::given(method("GET"))
            .and(path("/manifests/tool.json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&manifest))
            .mount(&server)
            .await;

        let temp = tempdir()?;
        env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
        let config = Config::new()?;
        config.ensure_dirs()?;

        let preferences = prefs();
        let fetcher = SourceFetcher::new(&config, preferences.clone())?;
        let sources = vec![source(Url::parse(&format!("{}/index.json", server.uri()))?)];
        let service = CatalogService::new(fetcher, preferences, sources);

        // Initial fetch populates the cache.
        let response = service
            .list(&ListRequest {
                search: None,
                category: None,
                source: None,
                installed_only: false,
                prefetch_filter: false,
            })
            .await?;
        assert_eq!(response.entries.len(), 1);
        assert!(response.warnings.is_empty());

        // Simulate subsequent network failure.
        server.reset().await;
        Mock::given(method("GET"))
            .and(path("/index.json"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let response = service
            .list(&ListRequest {
                search: None,
                category: None,
                source: None,
                installed_only: false,
                prefetch_filter: false,
            })
            .await?;
        assert_eq!(response.entries.len(), 1);
        assert!(!response.warnings.is_empty());

        env::remove_var("GEMINI_MARKETPLACE_HOME");
        server.stop().await;
        Ok(())
    })
}
