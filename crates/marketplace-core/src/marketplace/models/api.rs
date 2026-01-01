//! Serializable marketplace view models and JSON schemas.
//!
//! These records map the internal domain entities to the spec-defined shapes that power the CLI's JSON output and the OpenAPI contracts. They intentionally avoid implementation details so that they can derive `JsonSchema` for downstream tooling.

use std::time::Duration;

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::domain::{
    Extension, InstallStatus, MarketplaceSource, SourceType, SyncStatus, ValidationStatus,
    ValidationSummary,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallState {
    Installed,
    NotInstalled,
    UpdateAvailable,
    Unknown,
}

impl From<&InstallStatus> for InstallState {
    fn from(status: &InstallStatus) -> Self {
        match status {
            InstallStatus::Installed { .. } => InstallState::Installed,
            InstallStatus::NotInstalled => InstallState::NotInstalled,
            InstallStatus::UpdateAvailable { .. } => InstallState::UpdateAvailable,
            InstallStatus::Unknown => InstallState::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ExtensionRecord {
    pub namespace: String,
    pub name: String,
    #[schemars(length(max = 512))]
    pub description: String,
    pub version: String,
    pub author: String,
    pub categories: Vec<String>,
    pub repository_url: String,
    pub readme: Option<String>,
    pub install_status: InstallState,
    pub source_alias: String,
    pub warnings: Vec<String>,
    #[schemars(with = "Option<String>")]
    pub cache_freshness: Option<DateTime<Utc>>,
}

impl From<&Extension> for ExtensionRecord {
    fn from(extension: &Extension) -> Self {
        Self {
            namespace: extension.id.0.clone(),
            name: extension.display_name.to_string(),
            description: extension.summary.to_string(),
            version: extension.version.to_string(),
            author: extension.author.to_string(),
            categories: extension.categories.clone(),
            repository_url: extension.repository_url.to_string(),
            readme: extension.readme_excerpt.as_ref().map(|c| c.to_string()),
            install_status: InstallState::from(&extension.install_status),
            source_alias: extension.source_slug.clone(),
            warnings: validation_messages(&extension.validation_summary),
            cache_freshness: extension.last_synced_at.map(DateTime::<Utc>::from),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct ManifestCacheEntryRecord {
    pub namespace: String,
    #[schemars(with = "Option<String>")]
    pub stored_at: Option<DateTime<Utc>>,
    pub ttl_hours: u64,
    pub checksum: String,
    pub schema_valid: bool,
    pub semantic_valid: bool,
    pub metadata: Option<serde_json::Value>,
}

impl ManifestCacheEntryRecord {
    pub fn from_extension(extension: &Extension) -> Self {
        let stored_at = extension.last_synced_at.map(DateTime::<Utc>::from);
        let ttl_hours = match (extension.last_synced_at, extension.cache_expires_at) {
            (Some(start), Some(expiry)) => {
                expiry
                    .duration_since(start)
                    .unwrap_or_else(|_| Duration::from_secs(0))
                    .as_secs()
                    / 3600
            }
            _ => 0,
        };
        Self {
            namespace: extension.id.0.clone(),
            stored_at,
            ttl_hours,
            checksum: extension.manifest_checksum.clone(),
            schema_valid: matches!(
                extension.validation_summary.schema_status,
                ValidationStatus::Passed
            ),
            semantic_valid: matches!(
                extension.validation_summary.semantic_status,
                ValidationStatus::Passed | ValidationStatus::Warning
            ),
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct MarketplaceSourceRecord {
    pub alias: String,
    pub display_name: String,
    pub url: String,
    pub source_type: SourceTypeRecord,
    pub recursion_depth: u8,
    pub enabled: bool,
    pub default: bool,
    pub requires_auth: bool,
    #[schemars(with = "Option<String>")]
    pub last_sync_at: Option<DateTime<Utc>>,
    pub error_state: Option<SourceErrorRecord>,
}

impl From<&MarketplaceSource> for MarketplaceSourceRecord {
    fn from(source: &MarketplaceSource) -> Self {
        Self {
            alias: source.slug.clone(),
            display_name: source.display_name.clone(),
            url: source.url.to_string(),
            source_type: SourceTypeRecord::from(&source.source_type),
            recursion_depth: source.recursion_depth,
            enabled: source.enabled,
            default: source.default,
            requires_auth: source.requires_auth,
            last_sync_at: source.last_synced_at.map(DateTime::<Utc>::from),
            error_state: SourceErrorRecord::from_sync_status(&source.last_sync_status),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceTypeRecord {
    GithubRepo,
    GitUrl,
    LocalPath,
}

impl From<&SourceType> for SourceTypeRecord {
    fn from(source_type: &SourceType) -> Self {
        match source_type {
            SourceType::GithubRepo => SourceTypeRecord::GithubRepo,
            SourceType::GitUrl => SourceTypeRecord::GitUrl,
            SourceType::LocalPath => SourceTypeRecord::LocalPath,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct SourceErrorRecord {
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub warnings: Vec<String>,
}

impl SourceErrorRecord {
    fn from_sync_status(status: &SyncStatus) -> Option<Self> {
        match status {
            SyncStatus::Warning { warnings } => Some(Self {
                message: "source reported warnings".to_string(),
                warnings: warnings.clone(),
            }),
            SyncStatus::Error { message } => Some(Self {
                message: message.clone(),
                warnings: Vec::new(),
            }),
            _ => None,
        }
    }
}

fn validation_messages(summary: &ValidationSummary) -> Vec<String> {
    summary
        .errors
        .iter()
        .map(|error| format!("{}: {}", error.path, error.message))
        .collect()
}
