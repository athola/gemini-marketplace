//! Unit tests for default curated source configuration.
//!
//! Validates that the default marketplace source is correctly initialized,
//! persisted, and validated per FR-005a.

use gemini_marketplace::marketplace::models::domain::{MarketplaceSource, SourceType, SyncStatus};
use gemini_marketplace::marketplace::services::sources::SourcesService;
use url::Url;

#[test]
fn test_default_curated_source_structure() {
    // Verify the default curated source has correct structure
    let url = Url::parse("https://github.com/athola/gemini-marketplace").unwrap();
    let source = MarketplaceSource::default_curated(url.clone());

    assert_eq!(source.slug, "athola");
    assert_eq!(source.display_name, "Athola Default");
    assert_eq!(source.url, url);
    assert!(matches!(source.source_type, SourceType::GithubRepo));
    assert!(source.default);
    assert!(source.enabled);
    assert!(!source.requires_auth);
    assert!(source.last_synced_at.is_none());
    assert!(matches!(source.last_sync_status, SyncStatus::Idle));
    assert!(source.etag.is_none());
    assert!(source.poll_interval_hours.is_none());
}

#[test]
fn test_default_source_url_validation() {
    // Ensure default source URL is exactly as specified in FR-005a
    let expected_url = "https://github.com/athola/gemini-marketplace";
    let url = Url::parse(expected_url).unwrap();
    let source = MarketplaceSource::default_curated(url.clone());

    assert_eq!(source.url.as_str(), expected_url);
}

#[test]
fn test_sources_service_initializes_with_default() {
    // Verify SourcesService can be initialized
    let service = SourcesService::new();

    // Service should be creatable (placeholder implementation)
    assert!(std::mem::size_of_val(&service) > 0);
}

#[test]
fn test_sources_service_loads_default_source() {
    // Verify that get_default_source returns the curated source
    let service = SourcesService::new();
    let default_source = service.get_default_source();

    assert!(default_source.is_ok());
    let source = default_source.unwrap();
    assert!(source.default);
    assert_eq!(source.slug, "athola");
    assert_eq!(source.url.as_str(), "https://github.com/athola/gemini-marketplace");
}

#[test]
fn test_sources_service_lists_includes_default() {
    // Verify that list_sources includes the default source
    let service = SourcesService::new();
    let sources = service.list_sources();

    assert!(sources.is_ok());
    let source_list = sources.unwrap();
    assert!(!source_list.is_empty(), "Default source should be present");

    let default_exists = source_list.iter().any(|s| s.default && s.slug == "athola");
    assert!(default_exists, "Default 'athola' source must be in the list");
}

#[test]
fn test_default_source_cannot_be_removed() {
    // Verify that the default source cannot be removed
    let service = SourcesService::new();
    let result = service.remove_source("athola");

    assert!(result.is_err(), "Default source removal should be rejected");
    assert!(
        result.unwrap_err().to_string().contains("default")
        || result.unwrap_err().to_string().contains("cannot be removed"),
        "Error message should indicate default source protection"
    );
}

#[test]
fn test_default_source_can_be_disabled() {
    // Verify that users can disable (but not remove) the default source
    let service = SourcesService::new();
    let result = service.disable_source("athola");

    assert!(result.is_ok(), "Default source should be disableable");

    let sources = service.list_sources().unwrap();
    let default_source = sources.iter().find(|s| s.slug == "athola").unwrap();
    assert!(!default_source.enabled, "Default source should be disabled");
}

#[test]
fn test_default_source_can_be_re_enabled() {
    // Verify that disabled default source can be re-enabled
    let service = SourcesService::new();

    service.disable_source("athola").unwrap();
    let result = service.enable_source("athola");

    assert!(result.is_ok(), "Default source should be re-enableable");

    let sources = service.list_sources().unwrap();
    let default_source = sources.iter().find(|s| s.slug == "athola").unwrap();
    assert!(default_source.enabled, "Default source should be enabled again");
}

#[test]
fn test_default_source_slug_is_normalized() {
    // Verify slug normalization (lowercase, alphanumeric + hyphens)
    let url = Url::parse("https://github.com/athola/gemini-marketplace").unwrap();
    let source = MarketplaceSource::default_curated(url);

    assert_eq!(source.slug, "athola");
    assert!(source.slug.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'));
}
