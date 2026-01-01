use std::borrow::Cow;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use semver::Version;
use serde_json::{json, Value};
use tempfile::TempDir;
use url::Url;

use gemini_marketplace::marketplace::cache::{init, store::CacheStore};
use gemini_marketplace::marketplace::config::Config;
use gemini_marketplace::marketplace::models::domain::{
    Extension, ExtensionId, InstallStatus, ValidationSummary,
};

#[path = "../../common/mod.rs"]
mod common;
use common::MarketplaceCommand;

struct TestEnv {
    _home: TempDir,
}

impl TestEnv {
    async fn new(manifests: Vec<Value>) -> Result<Self> {
        let home = TempDir::new()?;
        std::env::set_var("GEMINI_MARKETPLACE_HOME", home.path());
        // Provide a valid source URL even though cache hits avoid network calls.
        std::env::set_var(
            "GEMINI_MARKETPLACE_SOURCE_URL",
            "http://example.com/index.json",
        );
        std::env::set_var("GEMINI_MARKETPLACE_LOG", "error");

        let config = Config::new()?;
        init::ensure_layout(&config)?;
        let cache = CacheStore::new(&config)?;

        for manifest in manifests {
            seed_cache(&cache, &manifest)?;
        }

        Ok(Self { _home: home })
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        std::env::remove_var("GEMINI_MARKETPLACE_SOURCE_URL");
        std::env::remove_var("GEMINI_MARKETPLACE_HOME");
        std::env::remove_var("GEMINI_MARKETPLACE_LOG");
    }
}

fn seed_cache(cache: &CacheStore, manifest: &Value) -> Result<()> {
    let name = manifest["name"]
        .as_str()
        .expect("manifest name should be present");
    let display = manifest["displayName"]
        .as_str()
        .expect("manifest displayName");
    let description = manifest["description"]
        .as_str()
        .expect("manifest description");
    let repository = manifest["repository"]
        .as_str()
        .expect("manifest repository");
    let version = manifest["version"].as_str().expect("manifest version");
    let author = manifest["author"].as_str().expect("manifest author");

    let categories = manifest["categories"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let tags = manifest["tags"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let compatibility = manifest["compatibility"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .filter_map(|value| value.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let now = SystemTime::now();
    let extension = Extension {
        id: ExtensionId::new("athola", name),
        source_slug: "athola".to_string(),
        extension_slug: name.to_string(),
        display_name: Cow::Owned(display.to_string()),
        summary: Cow::Owned(description.to_string()),
        repository_url: Url::parse(repository)?,
        homepage_url: None,
        documentation_url: None,
        version: Version::parse(version)?,
        author: Cow::Owned(author.to_string()),
        license: None,
        categories,
        tags,
        compatibility,
        install_status: InstallStatus::NotInstalled,
        manifest_checksum: "checksum".to_string(),
        readme_excerpt: manifest["readme"]
            .as_str()
            .map(|s| Cow::Owned(s.to_string())),
        last_synced_at: Some(now),
        cache_expires_at: Some(now + Duration::from_secs(3600)),
        validation_summary: ValidationSummary::new_pending(now),
        manifest_path: Some(".".to_string()),
    };

    cache.save(
        "athola",
        &[extension],
        Some("etag".to_string()),
        Duration::from_secs(3600),
    )?;
    Ok(())
}

fn make_manifest(name: &str) -> Value {
    json!({
        "name": name,
        "displayName": format!("{name} Extension"),
        "description": format!("{name} description"),
        "repository": format!("https://github.com/example/{name}"),
        "version": "1.0.0",
        "author": "Example Dev",
        "categories": ["analytics"],
        "tags": ["demo"],
        "compatibility": ["cli>=1.0"],
        "readme": "README",
    })
}

#[tokio::test]
async fn show_displays_extension_details() -> Result<()> {
    let _env = TestEnv::new(vec![make_manifest("alpha")]).await?;

    let output = MarketplaceCommand::new()
        .args(["show", "athola/alpha"])
        .run();

    output.expect_success();
    assert!(
        output.stdout.contains("alpha Extension"),
        "expected extension name in output, got:\n{}",
        output.stdout
    );

    Ok(())
}

#[tokio::test]
async fn show_supports_json_output() -> Result<()> {
    let _env = TestEnv::new(vec![make_manifest("alpha")]).await?;

    let output = MarketplaceCommand::new()
        .args(["show", "athola/alpha", "--json"])
        .run();

    output.expect_success();
    assert!(
        !output.stdout.trim().is_empty(),
        "expected JSON output, got empty stdout; stderr:\n{}",
        output.stderr
    );
    let json: Value = serde_json::from_str(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "expected stdout to be JSON: {err}\nstdout:\n{}\nstderr:\n{}",
            output.stdout, output.stderr
        )
    });
    assert_eq!(
        json["extension"]["namespace"], "athola/alpha",
        "expected namespace in JSON payload"
    );

    Ok(())
}

#[tokio::test]
async fn show_returns_not_found_for_unknown_extension() -> Result<()> {
    let _env = TestEnv::new(vec![make_manifest("alpha")]).await?;

    let output = MarketplaceCommand::new()
        .args(["show", "athola/missing"])
        .run();

    assert!(
        !output.status.success(),
        "expected failure status for missing extension"
    );
    assert!(
        output
            .stderr
            .contains("Extension 'athola/missing' not found"),
        "expected not-found message, got stderr:\n{}",
        output.stderr
    );

    Ok(())
}
