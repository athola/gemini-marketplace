use semver::Version;

use url::Url;

use gemini_marketplace::marketplace::models::domain::{
    ExtensionId, InstallStatus, MarketplaceSource, SourceType, SyncStatus,
};

#[test]
fn extension_id_formats_namespaced_identifier() {
    let id = ExtensionId::new("athola", "marketplace");
    assert_eq!(id.0, "athola/marketplace");
}

#[test]
fn default_curated_source_uses_expected_defaults() {
    let url = Url::parse("https://example.com/index.json").unwrap();
    let source = MarketplaceSource::default_curated(url.clone());

    assert_eq!(source.slug, "athola");
    assert_eq!(source.display_name, "Athola Default");
    assert_eq!(source.url, url);
    assert!(source.default);
    assert!(source.enabled);
    assert!(!source.requires_auth);
    matches!(source.source_type, SourceType::GithubRepo);
    matches!(source.last_sync_status, SyncStatus::Idle);
}

#[test]
fn install_status_transitions_allow_updates() {
    let installed = InstallStatus::Installed {
        version: Version::parse("1.0.0").unwrap(),
    };
    match installed {
        InstallStatus::Installed { version } => assert_eq!(version, Version::parse("1.0.0").unwrap()),
        other => panic!("expected Installed, got {:?}", other),
    }

    let update_available = InstallStatus::UpdateAvailable {
        installed_version: Version::parse("1.0.0").unwrap(),
        latest_version: Version::parse("1.1.0").unwrap(),
    };
    match update_available {
        InstallStatus::UpdateAvailable {
            installed_version,
            latest_version,
        } => {
            assert_eq!(installed_version, Version::parse("1.0.0").unwrap());
            assert_eq!(latest_version, Version::parse("1.1.0").unwrap());
        }
        other => panic!("expected UpdateAvailable, got {:?}", other),
    }
}
