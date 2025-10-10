//! Configuration helpers for marketplace directories.
//!
//! Resolves platform-specific paths for cache, preferences, and logs using the
//! `directories` crate so the extension remains portable across Linux, macOS,
//! and Windows targets. A `GEMINI_MARKETPLACE_HOME` override is available to
//! support isolated testing.

use std::env;
use std::path::{Path, PathBuf};

use directories::{ProjectDirs, UserDirs};

use crate::marketplace::error::{MarketplaceError, Result};

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "gemini";
const APPLICATION: &str = "marketplace-extension";
const ENV_OVERRIDE: &str = "GEMINI_MARKETPLACE_HOME";

#[derive(Debug, Clone)]
pub struct Config {
    cache_dir: PathBuf,
    config_dir: PathBuf,
    log_dir: PathBuf,
}

impl Config {
    /// Construct configuration paths, honoring the optional override env var for testing.
    pub fn new() -> Result<Self> {
        if let Ok(path) = env::var(ENV_OVERRIDE) {
            let base = Path::new(&path).to_path_buf();
            return Ok(Self {
                cache_dir: base.join("cache"),
                config_dir: base.join("config"),
                log_dir: base.join("logs"),
            });
        }

        let project_dirs =
            ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or_else(|| {
                MarketplaceError::Configuration("Unable to resolve project directories".into())
            })?;

        Ok(Self {
            cache_dir: project_dirs.cache_dir().to_path_buf(),
            config_dir: project_dirs.config_dir().to_path_buf(),
            log_dir: project_dirs.data_local_dir().join("logs"),
        })
    }

    /// Directory containing cache files (per-source JSON payloads, TTL metadata).
    pub fn cache_dir(&self) -> PathBuf {
        self.cache_dir.clone()
    }

    /// Directory containing configuration files (user preferences, sources list).
    pub fn config_dir(&self) -> PathBuf {
        self.config_dir.clone()
    }

    /// Directory for logs or diagnostics produced by the extension.
    pub fn log_dir(&self) -> PathBuf {
        self.log_dir.clone()
    }

    /// User downloads directory, used as fallback when exporting manifests or reports.
    pub fn downloads_dir(&self) -> Option<PathBuf> {
        UserDirs::new().and_then(|ud| ud.download_dir().map(PathBuf::from))
    }

    /// Ensure required directories exist, creating them as needed.
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|err| MarketplaceError::io(self.cache_dir.clone(), err))?;
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|err| MarketplaceError::io(self.config_dir.clone(), err))?;
        std::fs::create_dir_all(&self.log_dir)
            .map_err(|err| MarketplaceError::io(self.log_dir.clone(), err))?;
        Ok(())
    }
}
