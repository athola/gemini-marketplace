//! Shared error definitions for the marketplace extension.
//!
//! Centralizing errors simplifies mapping between service failures, CLI output,
//! and API responses. Concrete variants align with spec clarifications such as
//! skipped manifests, rate limiting, and credential requirements.

use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, MarketplaceError>;

#[derive(Debug, Error)]
pub enum MarketplaceError {
    #[error("configuration error: {0}")]
    Configuration(String),

    #[error("I/O error at {path:?}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("network request failed: {0}")]
    Network(String),

    #[error("rate limit active for source {source}: resets at {reset_at}")]
    RateLimited { source: String, reset_at: String },

    #[error("manifest invalid for repository {repository}: {reason}")]
    InvalidManifest { repository: String, reason: String },

    #[error("extension not found: {id}")]
    ExtensionNotFound { id: String },

    #[error("source not found: {slug}")]
    SourceNotFound { slug: String },

    #[error("authentication required for source {slug}, credentials missing")]
    AuthenticationRequired { slug: String },

    #[error("operation not yet implemented")]
    Todo,
}

impl MarketplaceError {
    pub fn io(path: impl Into<PathBuf>, source: io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }
}
