//! Domain-level structs representing marketplace entities.
//!
//! These types mirror the data model documented in
//! `specs/001-build-a-gemini/data-model.md` while preserving the legacy field
//! names already used throughout the services layer.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Extension {
    pub id: ExtensionId,
    pub source_slug: String,
    pub extension_slug: String,
    pub display_name: Cow<'static, str>,
    pub summary: Cow<'static, str>,
    pub repository_url: Url,
    pub homepage_url: Option<Url>,
    pub documentation_url: Option<Url>,
    pub version: Version,
    pub author: Cow<'static, str>,
    pub license: Option<Cow<'static, str>>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub compatibility: Vec<String>,
    pub install_status: InstallStatus,
    pub manifest_checksum: String,
    pub readme_excerpt: Option<Cow<'static, str>>,
    #[serde(with = "humantime_serde::option")]
    pub last_synced_at: Option<SystemTime>,
    #[serde(with = "humantime_serde::option")]
    pub cache_expires_at: Option<SystemTime>,
    #[serde(default)]
    pub validation_summary: ValidationSummary,
    #[serde(default)]
    pub manifest_path: Option<String>,
}

impl Extension {
    #[allow(clippy::too_many_arguments)]
    pub fn new<I, J>(
        id: ExtensionId,
        display_name: impl Into<String>,
        summary: impl Into<String>,
        repository_url: Url,
        version: Version,
        author: impl Into<String>,
        source_slug: impl Into<String>,
        categories: I,
        compatibility: J,
        install_status: InstallStatus,
        last_seen: Option<SystemTime>,
        validation_summary: ValidationSummary,
        manifest_path: impl Into<String>,
        readme_excerpt: Option<String>,
    ) -> Self
    where
        I: IntoIterator,
        I::Item: Into<String>,
        J: IntoIterator,
        J::Item: Into<String>,
    {
        let source_slug_str = source_slug.into();
        let extension_slug = id.0.split('/').nth(1).unwrap_or_default().to_string();
        Self {
            id,
            source_slug: source_slug_str,
            extension_slug,
            display_name: Cow::Owned(display_name.into()),
            summary: Cow::Owned(summary.into()),
            repository_url,
            homepage_url: None,
            documentation_url: None,
            version,
            author: Cow::Owned(author.into()),
            license: None,
            categories: dedup_case_insensitive(categories),
            tags: Vec::new(),
            compatibility: dedup_case_insensitive(compatibility),
            install_status,
            manifest_checksum: String::new(),
            readme_excerpt: readme_excerpt.map(Cow::Owned),
            last_synced_at: last_seen,
            cache_expires_at: None,
            validation_summary,
            manifest_path: Some(manifest_path.into()),
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarketplaceSource {
    pub slug: String,
    pub display_name: String,
    pub url: Url,
    pub source_type: SourceType,
    pub default: bool,
    pub enabled: bool,
    pub requires_auth: bool,
    #[serde(with = "humantime_serde::option")]
    pub last_synced_at: Option<SystemTime>,
    pub last_sync_status: SyncStatus,
    pub etag: Option<String>,
    pub poll_interval_hours: Option<u16>,
    pub recursion_depth: u8,
}

impl MarketplaceSource {
    pub fn default_curated(url: Url) -> Self {
        Self {
            slug: "athola".to_string(),
            display_name: "Athola Curated".to_string(),
            url,
            source_type: SourceType::GithubRepo,
            default: true,
            enabled: true,
            requires_auth: false,
            last_synced_at: None,
            last_sync_status: SyncStatus::Idle,
            etag: None,
            poll_interval_hours: None,
            recursion_depth: 5,
        }
    }

    pub fn new(
        slug: impl Into<String>,
        display_name: impl Into<String>,
        url: Url,
        source_type: SourceType,
        enabled: bool,
        recursion_depth: u8,
    ) -> Self {
        Self {
            slug: slug.into(),
            display_name: display_name.into(),
            url,
            source_type,
            default: false,
            enabled,
            requires_auth: false,
            last_synced_at: None,
            last_sync_status: SyncStatus::Idle,
            etag: None,
            poll_interval_hours: None,
            recursion_depth,
        }
    }

    pub fn with_last_synced_at(mut self, value: Option<SystemTime>) -> Self {
        self.last_synced_at = value;
        self
    }

    pub fn with_etag(mut self, value: Option<String>) -> Self {
        self.etag = value;
        self
    }

    pub fn with_sync_status(mut self, status: SyncStatus) -> Self {
        self.last_sync_status = status;
        self
    }

    pub fn with_requires_auth(mut self, requires_auth: bool) -> Self {
        self.requires_auth = requires_auth;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceType {
    GithubRepo,
    GitUrl,
    LocalPath,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Healthy,
    Warning { warnings: Vec<String> },
    Error { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheEntry {
    pub source_slug: String,
    pub batch_index: u32,
    pub manifest_checksum: String,
    pub payload_path: PathBuf,
    #[serde(with = "humantime_serde")]
    pub fetched_at: SystemTime,
    #[serde(with = "humantime_serde")]
    pub expires_at: SystemTime,
    pub extension_ids: Vec<ExtensionId>,
    pub etag: Option<String>,
}

impl CacheEntry {
    pub fn new(
        source_slug: impl Into<String>,
        batch_index: u32,
        fetched_at: SystemTime,
        expires_at: SystemTime,
        extension_ids: Vec<ExtensionId>,
        checksum: Option<String>,
    ) -> Self {
        Self {
            source_slug: source_slug.into(),
            batch_index,
            manifest_checksum: checksum.unwrap_or_else(|| format!("batch-{batch_index}")),
            payload_path: PathBuf::new(),
            fetched_at,
            expires_at,
            extension_ids,
            etag: None,
        }
    }

    pub fn is_stale(&self, now: SystemTime) -> bool {
        now >= self.expires_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryJob {
    pub job_id: String,
    pub source_slug: String,
    #[serde(with = "humantime_serde")]
    pub scheduled_for: SystemTime,
    pub attempt: u16,
    pub reason: String,
}

impl RetryJob {
    pub fn new(
        source_slug: impl Into<String>,
        job_id: impl Into<String>,
        scheduled_for: SystemTime,
        attempt: u16,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            job_id: job_id.into(),
            source_slug: source_slug.into(),
            scheduled_for,
            attempt,
            reason: reason.into(),
        }
    }

    pub fn next_attempt(&self, backoff: Duration) -> Self {
        Self {
            job_id: self.job_id.clone(),
            source_slug: self.source_slug.clone(),
            scheduled_for: self.scheduled_for + backoff,
            attempt: self.attempt.saturating_add(1),
            reason: self.reason.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimitWindow {
    pub source_slug: String,
    #[serde(with = "humantime_serde::option")]
    pub reset_at: Option<SystemTime>,
    pub remaining_requests: Option<u32>,
    pub limit: Option<u32>,
}

impl RateLimitWindow {
    pub fn new(
        source_slug: impl Into<String>,
        reset_at: Option<SystemTime>,
        remaining_requests: Option<u32>,
    ) -> Self {
        Self {
            source_slug: source_slug.into(),
            reset_at,
            remaining_requests,
            limit: None,
        }
    }

    pub fn time_until_reset(&self, now: SystemTime) -> Option<Duration> {
        let reset = self.reset_at?;
        reset.duration_since(now).ok()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub cache_ttl_hours: u16,
    pub auto_refresh_on_launch: bool,
    pub search_mode: SearchMode,
    pub output_format: OutputFormat,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            cache_ttl_hours: 24,
            auto_refresh_on_launch: false,
            search_mode: SearchMode::default(),
            output_format: OutputFormat::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchMode {
    LocalFilter,
    PreFetchFilter,
}

impl Default for SearchMode {
    fn default() -> Self {
        SearchMode::LocalFilter
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputFormat {
    Table,
    Json,
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Table
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationSummary {
    pub schema_status: ValidationStatus,
    pub semantic_status: ValidationStatus,
    pub errors: Vec<ValidationError>,
    #[serde(with = "humantime_serde")]
    pub last_checked: SystemTime,
}

impl ValidationSummary {
    pub fn new(
        schema_status: ValidationStatus,
        semantic_status: ValidationStatus,
        errors: Vec<ValidationError>,
        last_checked: SystemTime,
    ) -> Self {
        Self {
            schema_status,
            semantic_status,
            errors,
            last_checked,
        }
    }

    pub fn new_pending(now: SystemTime) -> Self {
        Self::new(
            ValidationStatus::Passed,
            ValidationStatus::Pending,
            Vec::new(),
            now,
        )
    }
}

impl Default for ValidationSummary {
    fn default() -> Self {
        Self::new_pending(SystemTime::UNIX_EPOCH)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Pending,
    Passed,
    Warning,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
}

impl ValidationError {
    pub fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            path: path.into(),
        }
    }
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

fn dedup_case_insensitive<I>(items: I) -> Vec<String>
where
    I: IntoIterator,
    I::Item: Into<String>,
{
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        let value: String = item.into();
        let key = value.to_lowercase();
        if seen.insert(key) {
            result.push(value);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use humantime::format_rfc3339_seconds;
    use semver::Version;
    use serde_json::json;
    use std::time::{Duration, SystemTime};
    use url::Url;

    #[test]
    fn extension_round_trips_and_deduplicates_categories() {
        let mut extension = Extension::new(
            ExtensionId::new("curated", "awesome-extension"),
            "Awesome Extension",
            "Great things happen here.",
            Url::parse("https://github.com/example/awesome").unwrap(),
            Version::parse("1.2.3").unwrap(),
            "Example Dev",
            "curated",
            ["Analytics", "analytics", "CLI"],
            ["Gemini CLI >=1.0"],
            InstallStatus::Installed {
                version: Version::parse("1.2.3").unwrap(),
            },
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(60)),
            ValidationSummary::new_pending(SystemTime::UNIX_EPOCH),
            "extensions/awesome",
            Some("Readme excerpt".into()),
        );
        extension.manifest_checksum = "checksum".into();

        assert_eq!(
            extension.categories,
            vec!["Analytics".to_string(), "CLI".to_string()],
            "duplicate categories should collapse case-insensitively while preserving first casing"
        );

        let as_json = serde_json::to_value(&extension).expect("serialize extension");
        let round_trip: Extension = serde_json::from_value(as_json).expect("deserialize extension");
        assert_eq!(round_trip, extension);
    }

    #[test]
    fn validation_summary_serializes_errors() {
        let summary = ValidationSummary::new(
            ValidationStatus::Passed,
            ValidationStatus::Failed,
            vec![ValidationError::new(
                "invalid_semver",
                "Version must follow SemVer.",
                "/version",
            )],
            SystemTime::UNIX_EPOCH + Duration::from_secs(120),
        );

        let value = serde_json::to_value(&summary).expect("serialize validation summary");
        assert_eq!(
            value,
            json!({
                "schema_status": "passed",
                "semantic_status": "failed",
                "errors": [{
                    "code": "invalid_semver",
                    "message": "Version must follow SemVer.",
                    "path": "/version"
                }],
                "last_checked": format_rfc3339_seconds(
                    SystemTime::UNIX_EPOCH + Duration::from_secs(120)
                )
                .to_string()
            })
        );

        let back: ValidationSummary =
            serde_json::from_value(value).expect("round-trip validation summary");
        assert_eq!(back, summary);
    }

    #[test]
    fn cache_entry_marks_stale_when_expired() {
        let entry = CacheEntry::new(
            "curated",
            0,
            SystemTime::UNIX_EPOCH,
            SystemTime::UNIX_EPOCH + Duration::from_secs(30),
            vec![ExtensionId::new("curated", "awesome-extension")],
            Some("checksum".into()),
        );

        assert!(
            entry.is_stale(SystemTime::UNIX_EPOCH + Duration::from_secs(31)),
            "entry should be stale after expiry"
        );
        assert!(
            !entry.is_stale(SystemTime::UNIX_EPOCH + Duration::from_secs(5)),
            "entry should be fresh before expiry"
        );
    }

    #[test]
    fn retry_job_computes_next_schedule_with_backoff() {
        let job = RetryJob::new(
            "curated",
            "curated::batch-0",
            SystemTime::UNIX_EPOCH,
            1,
            "timeout",
        );

        let next = job.next_attempt(Duration::from_secs(30));
        assert_eq!(next.attempt, 2);
        assert_eq!(
            next.scheduled_for,
            SystemTime::UNIX_EPOCH + Duration::from_secs(30)
        );
    }

    #[test]
    fn rate_limit_window_reports_remaining_time() {
        let window = RateLimitWindow::new(
            "curated",
            Some(SystemTime::UNIX_EPOCH + Duration::from_secs(90)),
            Some(10),
        );

        assert_eq!(
            window.time_until_reset(SystemTime::UNIX_EPOCH + Duration::from_secs(50)),
            Some(Duration::from_secs(40))
        );

        assert_eq!(
            window.time_until_reset(SystemTime::UNIX_EPOCH + Duration::from_secs(100)),
            None
        );
    }

    #[test]
    fn marketplace_source_round_trip() {
        let source = MarketplaceSource::new(
            "curated",
            "Curated",
            Url::parse("https://github.com/athola/gemini-marketplace").unwrap(),
            SourceType::GithubRepo,
            true,
            5,
        )
        .with_sync_status(SyncStatus::Syncing)
        .with_etag(Some("etag".into()))
        .with_last_synced_at(Some(SystemTime::UNIX_EPOCH + Duration::from_secs(45)));

        let value = serde_json::to_value(&source).expect("serialize source");
        let round_trip: MarketplaceSource =
            serde_json::from_value(value).expect("deserialize source");
        assert_eq!(round_trip, source);
    }
}
