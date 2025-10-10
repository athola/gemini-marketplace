//! Domain-level structs representing marketplace entities.
//!
//! These types mirror the data model documented in `data-model.md` and act as
//! the canonical representation inside the extension.

use std::collections::BTreeMap;
use std::time::SystemTime;

use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;

/// Unique identifier format: `source-slug/extension-slug`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExtensionId(pub String);

impl ExtensionId {
    pub fn new(source_slug: &str, extension_slug: &str) -> Self {
        Self(format!("{source_slug}/{extension_slug}"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Extension {
    pub id: ExtensionId,
    pub source_slug: String,
    pub extension_slug: String,
    pub display_name: String,
    pub summary: String,
    pub repository_url: Url,
    pub homepage_url: Option<Url>,
    pub documentation_url: Option<Url>,
    pub version: Version,
    pub author: String,
    pub license: Option<String>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub compatibility: Vec<String>,
    pub install_status: InstallStatus,
    pub manifest_checksum: String,
    pub readme_excerpt: Option<String>,
    #[serde(with = "humantime_serde::option")]
    pub last_synced_at: Option<SystemTime>,
    #[serde(with = "humantime_serde::option")]
    pub cache_expires_at: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallStatus {
    NotInstalled,
    Installed {
        version: Version,
    },
    UpdateAvailable {
        installed_version: Version,
        latest_version: Version,
    },
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSource {
    pub slug: String,
    pub display_name: String,
    pub url: Url,
    pub source_type: SourceType,
    pub default: bool,
    pub enabled: bool,
    pub requires_auth: bool,
    pub last_synced_at: Option<SystemTime>,
    pub last_sync_status: SyncStatus,
    pub etag: Option<String>,
    pub poll_interval_hours: Option<u64>,
}

impl MarketplaceSource {
    pub fn default_curated(url: Url) -> Self {
        Self {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceType {
    GithubRepo,
    GitUrl,
    LocalPath,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Healthy,
    Warning { warnings: Vec<String> },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub source_slug: String,
    pub manifest_checksum: String,
    pub payload_path: String,
    #[serde(with = "humantime_serde")]
    pub fetched_at: SystemTime,
    #[serde(with = "humantime_serde")]
    pub expires_at: SystemTime,
    pub extension_ids: Vec<ExtensionId>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitWindow {
    pub source_slug: String,
    #[serde(with = "humantime_serde::option")]
    pub reset_at: Option<SystemTime>,
    pub remaining_requests: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub cache_ttl_hours: u16,
    pub auto_refresh_on_launch: bool,
    pub search_mode: SearchMode,
    pub output_format: OutputFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchMode {
    LocalFilter,
    PreFetchFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionListView {
    pub extensions: Vec<Extension>,
    pub rate_limit_windows: BTreeMap<String, RateLimitWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStatusView {
    pub sources: Vec<MarketplaceSource>,
    pub pending_refresh_jobs: Vec<String>,
}
