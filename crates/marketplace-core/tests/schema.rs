use chrono::{TimeZone, Utc};
use gemini_marketplace::marketplace::models::api::{
    ExtensionRecord, InstallState, ManifestCacheEntryRecord, MarketplaceSourceRecord,
};
use gemini_marketplace::marketplace::models::domain::{
    Extension, ExtensionId, InstallStatus, MarketplaceSource, SourceType, ValidationError,
    ValidationStatus, ValidationSummary,
};
use schemars::schema_for;
use semver::Version;
use url::Url;

fn sample_extension() -> Extension {
    let now = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(60);
    let mut ext = Extension::new(
        ExtensionId::new("curated", "awesome"),
        "Awesome Extension",
        "Test description",
        Url::parse("https://example.com/repo").unwrap(),
        Version::parse("1.2.3").unwrap(),
        "Example Dev",
        "curated",
        ["cli", "productivity"],
        ["Gemini CLI >=1.0"],
        InstallStatus::Installed {
            version: Version::parse("1.2.3").unwrap(),
        },
        Some(now),
        ValidationSummary::new(
            ValidationStatus::Passed,
            ValidationStatus::Warning,
            vec![ValidationError::new(
                "schema_warning",
                "Example warning",
                "/name",
            )],
            now,
        ),
        "extensions/awesome",
        Some("README excerpt".into()),
    );
    ext.cache_expires_at = Some(now + std::time::Duration::from_secs(60 * 60 * 6));
    ext
}

fn sample_source() -> MarketplaceSource {
    MarketplaceSource::new(
        "curated",
        "Curated",
        Url::parse("https://github.com/athola/gemini-marketplace").unwrap(),
        SourceType::GithubRepo,
        true,
        5,
    )
}

#[test]
fn extension_record_converts_domain_model() {
    let extension = sample_extension();
    let record = ExtensionRecord::from(&extension);

    assert_eq!(record.namespace, "curated/awesome");
    assert_eq!(record.name, "Awesome Extension");
    assert_eq!(record.install_status, InstallState::Installed);
    assert_eq!(record.source_alias, "curated");
    assert_eq!(record.warnings.len(), 1);
    assert_eq!(
        record.cache_freshness.unwrap(),
        Utc.timestamp_opt(60, 0).unwrap()
    );
}

#[test]
fn manifest_cache_entry_record_derives_ttl_hours() {
    let extension = sample_extension();
    let record = ManifestCacheEntryRecord::from_extension(&extension);

    assert_eq!(record.namespace, "curated/awesome");
    assert_eq!(record.ttl_hours, 6);
    assert!(record.schema_valid);
    assert_eq!(record.checksum, extension.manifest_checksum);
}

#[test]
fn marketplace_source_record_round_trips() {
    let source = sample_source();
    let record = MarketplaceSourceRecord::from(&source);

    assert_eq!(record.alias, "curated");
    assert_eq!(record.url, "https://github.com/athola/gemini-marketplace");
    assert!(record.enabled);
}

#[test]
fn extension_schema_exposes_expected_fields() {
    let schema = schema_for!(ExtensionRecord);
    let json = serde_json::to_value(schema).expect("serialize schema");
    let props = json["properties"].as_object().expect("object schema");
    assert!(props.contains_key("namespace"));
    assert!(props.contains_key("cache_freshness"));
    assert!(props.contains_key("warnings"));
}
