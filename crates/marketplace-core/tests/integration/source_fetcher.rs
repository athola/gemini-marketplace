use std::net::SocketAddr;
use axum::{Router, routing::get, Json};
use axum::http::{StatusCode, Response};
use axum::response::IntoResponse;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use url::Url;

use gemini_marketplace::marketplace::cache::init;
use gemini_marketplace::marketplace::config::Config;
use gemini_marketplace::marketplace::models::domain::{MarketplaceSource, OutputFormat, SearchMode, SyncStatus, UserPreferences};
use gemini_marketplace::marketplace::services::preferences::PreferencesService;
use gemini_marketplace::marketplace::services::source_fetcher::SourceFetcher;

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

fn prefs(ttl_hours: u16) -> PreferencesService {
    PreferencesService::new(UserPreferences {
        cache_ttl_hours: ttl_hours,
        auto_refresh_on_launch: false,
        search_mode: SearchMode::LocalFilter,
        output_format: OutputFormat::Table,
    })
}

fn default_source(url: Url) -> MarketplaceSource {
    let mut source = MarketplaceSource::default_curated(url);
    source.last_sync_status = SyncStatus::Idle;
    source
}

#[tokio::test]
async fn sync_source_fetches_extensions_and_uses_cache() -> anyhow::Result<()> {
    let manifest_list = serde_json::json!(["http://placeholder/manifests/ext1.json"]);
    let manifest_body = serde_json::json!({
        "name": "demo",
        "description": "Demo extension",
        "repository": "https://github.com/example/demo",
        "version": "1.0.0",
        "author": "Example",
        "categories": ["utility"],
        "tags": ["tool"],
        "compatibility": ["cli>=1.0"],
        "readme": "# Demo"
    });

    let list_clone = manifest_list.clone();
    let manifest_clone = manifest_body.clone();

    let app = Router::new()
        .route(
            "/index.json",
            get(move || {
                let data = list_clone.clone();
                async move { Json(data) }
            }),
        )
        .route(
            "/manifests/ext1.json",
            get(move || {
                let data = manifest_clone.clone();
                async move { Json(data) }
            }),
        );

    let (addr, handle) = spawn(app).await?;
    let base = format!("http://{addr}");

    let temp = tempdir()?;
    std::env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    let config = Config::new()?;
    init::ensure_layout(&config)?;

    let fetcher = SourceFetcher::new(&config, prefs(24))?;
    let mut source = default_source(Url::parse(&format!("{base}/index.json"))?);

    let extensions = fetcher.sync_source(&source).await?;
    assert_eq!(extensions.len(), 1);
    assert_eq!(extensions[0].extension_slug, "demo");

    // Drop server so subsequent calls cannot fetch new data; expect cached result.
    handle.abort();
    let cached = fetcher.sync_source(&source).await?;
    assert_eq!(cached.len(), 1);

    std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    Ok(())
}

#[tokio::test]
async fn sync_source_respects_rate_limits() -> anyhow::Result<()> {
    let rate_limit_app = Router::new().route(
        "/index.json",
        get(|| async {
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .header("x-ratelimit-reset", "9999999999")
                .header("x-ratelimit-remaining", "0")
                .header("x-ratelimit-limit", "60")
                .body(axum::body::Body::empty())
                .unwrap()
        }),
    );

    let (addr, handle) = spawn(rate_limit_app).await?;
    let base = format!("http://{addr}");

    let temp = tempdir()?;
    std::env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    let config = Config::new()?;
    init::ensure_layout(&config)?;

    let fetcher = SourceFetcher::new(&config, prefs(24))?;
    let source = default_source(Url::parse(&format!("{base}/index.json"))?);

    let err = fetcher.sync_source(&source).await.expect_err("rate limit");
    match err {
        gemini_marketplace::marketplace::error::MarketplaceError::RateLimited { source_slug, .. } => {
            assert_eq!(source_slug, "athola");
        }
        other => panic!("expected RateLimited, got {:?}", other),
    }

    handle.abort();
    std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    Ok(())
}
