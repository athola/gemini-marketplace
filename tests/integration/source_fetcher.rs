use std::env;
use std::time::Duration;

use gemini_marketplace::marketplace::config::Config;
use gemini_marketplace::marketplace::models::domain::{MarketplaceSource, SourceType, SyncStatus, UserPreferences};
use gemini_marketplace::marketplace::services::preferences::PreferencesService;
use gemini_marketplace::marketplace::services::source_fetcher::SourceFetcher;
use tempfile::tempdir;
use tokio::time::timeout;
use url::Url;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn default_source(url: Url) -> MarketplaceSource {
    MarketplaceSource {
        slug: "athola".to_string(),
        display_name: "Athola Default".to_string(),
        url,
        source_type: SourceType::GithubRepo,
        default: true,
        enabled: true,
        requires_auth: false,
        last_synced_at: None,
        last_sync_status: SyncStatus::Idle,
        etag: None,
        poll_interval_hours: None,
    }
}

fn preferences(ttl_hours: u16) -> PreferencesService {
    PreferencesService::new(UserPreferences {
        cache_ttl_hours: ttl_hours,
        auto_refresh_on_launch: false,
        search_mode: gemini_marketplace::marketplace::models::domain::SearchMode::LocalFilter,
        output_format: gemini_marketplace::marketplace::models::domain::OutputFormat::Table,
    })
}

#[tokio::test]
async fn sync_source_fetches_extensions_and_uses_cache() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let manifests = vec![format!("{}/manifests/ext1.json", server.uri())];

    Mock::given(method("GET"))
        .and(path("/index.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifests))
        .mount(&server)
        .await;

    let manifest_body = serde_json::json!({
        "name": "demo",
        "description": "Demo extension",
        "repository": "https://github.com/example/demo",
        "version": "1.0.0",
        "author": "Example",
        "categories": ["CLI"],
        "tags": ["utility"],
        "compatibility": ["cli>=1.0"],
        "readme": "# Demo"
    });

    Mock::given(method("GET"))
        .and(path("/manifests/ext1.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifest_body))
        .mount(&server)
        .await;

    let temp = tempdir()?;
    env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    let config = Config::new()?;
    config.ensure_dirs()?;

    let fetcher = SourceFetcher::new(&config, preferences(24))?;
    let mut source = default_source(Url::parse(&format!("{}/index.json", server.uri()))?);

    let extensions = fetcher.sync_source(&source).await?;
    assert_eq!(extensions.len(), 1);
    assert_eq!(extensions[0].extension_slug, "demo");

    // Clear mocks so any network call would fail.
    server.reset().await;

    // Second call should return cached data without hitting network.
    let cached = fetcher.sync_source(&source).await?;
    assert_eq!(cached.len(), 1);
    assert_eq!(cached[0].extension_slug, "demo");

    env::remove_var("GEMINI_MARKETPLACE_HOME");
    Ok(())
}

#[tokio::test]
async fn sync_source_respects_rate_limits() -> anyhow::Result<()> {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/index.json"))
        .respond_with(
            ResponseTemplate::new(403)
                .append_header("x-ratelimit-reset", "9999999999")
                .append_header("x-ratelimit-remaining", "0")
                .append_header("x-ratelimit-limit", "60"),
        )
        .mount(&server)
        .await;

    let temp = tempdir()?;
    env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    let config = Config::new()?;
    config.ensure_dirs()?;

    let fetcher = SourceFetcher::new(&config, preferences(24))?;
    let source = default_source(Url::parse(&format!("{}/index.json", server.uri()))?);

    let err = fetcher.sync_source(&source).await.expect_err("rate limit");
    match err {
        gemini_marketplace::marketplace::error::MarketplaceError::RateLimited { source, .. } => {
            assert_eq!(source, "athola");
        }
        other => panic!("expected RateLimited, got {:?}", other),
    }

    env::remove_var("GEMINI_MARKETPLACE_HOME");
    Ok(())
}
