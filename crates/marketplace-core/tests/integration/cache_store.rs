use std::env;
use std::time::{Duration, SystemTime};

use gemini_marketplace::marketplace::cache::{init, store::CacheStore};
use gemini_marketplace::marketplace::config::Config;
use gemini_marketplace::marketplace::models::domain::{Extension, ExtensionId, InstallStatus};
use semver::Version;
use tempfile::tempdir;
use url::Url;

fn sample_extension(source_slug: &str, slug: &str) -> Extension {
    Extension {
        id: ExtensionId::new(source_slug, slug),
        source_slug: source_slug.to_string(),
        extension_slug: slug.to_string(),
        display_name: "Sample Extension".to_string(),
        summary: "Summary".to_string(),
        repository_url: Url::parse("https://github.com/example/demo").unwrap(),
        homepage_url: None,
        documentation_url: None,
        version: Version::parse("1.0.0").unwrap(),
        author: "Example".to_string(),
        license: Some("MIT".to_string()),
        categories: vec!["cli-tools".to_string()],
        tags: vec!["utility".to_string()],
        compatibility: vec!["cli>=1.0".to_string()],
        install_status: InstallStatus::NotInstalled,
        manifest_checksum: "abc123".to_string(),
        readme_excerpt: None,
        last_synced_at: Some(SystemTime::now()),
        cache_expires_at: None,
    }
}

#[test]
fn cache_store_persists_and_loads_entries() -> anyhow::Result<()> {
    let temp = tempdir()?;
    env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    let config = Config::new()?;
    init::ensure_layout(&config)?;

    let store = CacheStore::new(&config)?;
    let extensions = vec![sample_extension("athola", "marketplace")];

    store.save("athola", &extensions, Some("etag-123".into()), Duration::from_secs(3600))?;

    let snapshot = store.load("athola")?.expect("cache entry exists");
    assert_eq!(snapshot.entry.source_slug, "athola");
    assert!(snapshot.entry.expires_at > snapshot.entry.fetched_at);
    assert_eq!(snapshot.entry.etag.as_deref(), Some("etag-123"));
    assert_eq!(snapshot.entry.extension_ids.len(), 1);
    assert_eq!(snapshot.extensions.len(), 1);

    store.invalidate("athola")?;
    assert!(store.load("athola")?.is_none());

    env::remove_var("GEMINI_MARKETPLACE_HOME");
    Ok(())
}
