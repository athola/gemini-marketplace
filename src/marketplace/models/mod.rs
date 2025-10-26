//! Marketplace data models.

pub mod domain;
pub mod manifest;

pub use domain::{
    CacheEntry, Extension, ExtensionId, InstallStatus, MarketplaceSource, RateLimitWindow,
    RetryJob, SourceType, SyncStatus, ValidationError, ValidationStatus, ValidationSummary,
};
