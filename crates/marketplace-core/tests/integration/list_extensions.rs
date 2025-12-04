use std::net::SocketAddr;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use axum::{Router, routing::get, Json, response::IntoResponse};
use axum::http::{StatusCode, Response};
use serde_json::json;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use url::Url;

use gemini_marketplace::marketplace::cache::init;
use gemini_marketplace::marketplace::config::Config;
use gemini_marketplace::marketplace::models::domain::{MarketplaceSource, OutputFormat, SearchMode, UserPreferences};
use gemini_marketplace::marketplace::services::catalog::{CatalogService, ListRequest};
use gemini_marketplace::marketplace::services::preferences::PreferencesService;
use gemini_marketplace::marketplace::services::source_fetcher::SourceFetcher;
use gemini_marketplace::marketplace::status::StatusStore;

async fn spawn(app: Router) -> anyhow::Result<(SocketAddr, JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    });
    Ok((addr, handle))
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

#[tokio::test]
async fn catalog_service_lists_and_filters_extensions() -> anyhow::Result<()> {
    let manifests = json!([
        "http://placeholder/manifests/analytics.json",
        "http://placeholder/manifests/storage.json",
    ]);
    let analytics_manifest = json!({
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
    let storage_manifest = json!({
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

    let manifests_clone = manifests.clone();
    let analytics_clone = analytics_manifest.clone();
    let storage_clone = storage_manifest.clone();

    let app = Router::new()
        .route(
            "/index.json",
            get(move || {
                let data = manifests_clone.clone();
                async move { Json(data) }
            }),
        )
        .route(
            "/manifests/analytics.json",
            get(move || {
                let data = analytics_clone.clone();
                async move { Json(data) }
            }),
        )
        .route(
            "/manifests/storage.json",
            get(move || {
                let data = storage_clone.clone();
                async move { Json(data) }
            }),
        );

    let (addr, handle) = spawn(app).await?;
    let base_url = format!("http://{addr}");

    let temp = tempdir()?;
    std::env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    std::env::set_var("GEMINI_MARKETPLACE_SOURCE_URL", format!("{base_url}/index.json"));

    let config = Config::new()?;
    init::ensure_layout(&config)?;
    let preferences = prefs();
    let fetcher = SourceFetcher::new(&config, preferences.clone())?;
    let sources = vec![source(Url::parse(&format!("{base_url}/index.json"))?)];
    let status_store = StatusStore::new(&config);
    let service = CatalogService::new(fetcher, preferences, sources, status_store);

    let response = service
        .list(&ListRequest {
            search: Some("analytics"),
            category: None,
            source: None,
            installed_only: false,
            prefetch_filter: false,
        })
        .await?;
    assert_eq!(response.entries.len(), 1);
    assert_eq!(response.entries[0].namespace, "athola/analytics");

    let response = service
        .list(&ListRequest {
            search: None,
            category: Some("infrastructure"),
            source: None,
            installed_only: false,
            prefetch_filter: false,
        })
        .await?;
    assert_eq!(response.entries.len(), 1);
    assert_eq!(response.entries[0].namespace, "athola/storage");

    let response = service
        .list(&ListRequest {
            search: None,
            category: None,
            source: Some("athola"),
            installed_only: false,
            prefetch_filter: false,
        })
        .await?;
    assert_eq!(response.entries.len(), 1);
    assert_eq!(response.entries[0].namespace, "athola/analytics");

    let response = service
        .list(&ListRequest {
            search: None,
            category: None,
            source: Some("ath"),
            installed_only: false,
            prefetch_filter: false,
        })
        .await?;
    assert!(response.entries.is_empty());

    handle.abort();
    std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    std::env::remove_var("GEMINI_MARKETPLACE_SOURCE_URL");
    Ok(())
}

#[tokio::test]
async fn catalog_service_returns_cached_entries_on_network_failure() -> anyhow::Result<()> {
    let manifests = Arc::new(json!(["http://placeholder/manifests/tool.json"]));
    let manifest_body = json!({
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

    let index_calls = Arc::new(AtomicUsize::new(0));
    let index_calls_clone = index_calls.clone();
    let manifests_clone = manifests.clone();
    let manifest_body_clone = manifest_body.clone();

    let app = Router::new()
        .route(
            "/index.json",
            get(move || {
                let manifests = manifests_clone.clone();
                let counter = index_calls_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count == 0 {
                        Json((*manifests).clone()).into_response()
                    } else {
                        Response::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .body(axum::body::Body::empty())
                            .unwrap()
                            .into_response()
                    }
                }
            }),
        )
        .route(
            "/manifests/tool.json",
            get(move || {
                let manifest = manifest_body_clone.clone();
                async move { Json(manifest) }
            }),
        );

    let (addr, handle) = spawn(app).await?;
    let base_url = format!("http://{addr}");

    let temp = tempdir()?;
    std::env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    std::env::set_var("GEMINI_MARKETPLACE_SOURCE_URL", format!("{base_url}/index.json"));

    let config = Config::new()?;
    init::ensure_layout(&config)?;
    let preferences = prefs();
    let fetcher = SourceFetcher::new(&config, preferences.clone())?;
    let sources = vec![source(Url::parse(&format!("{base_url}/index.json"))?)];
    let status_store = StatusStore::new(&config);
    let service = CatalogService::new(fetcher, preferences, sources, status_store);

    // Warm cache
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

    // Subsequent call should return cached data and warning.
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

    handle.abort();
    std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    std::env::remove_var("GEMINI_MARKETPLACE_SOURCE_URL");
    Ok(())
}
