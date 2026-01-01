//! Configuration helpers for marketplace directories.
//!
//! By default, the marketplace persists data beneath the Gemini CLI config
//! directory (`$GEMINI_CONFIG/extensions/marketplace`). This keeps caches and
//! preferences co-located with other Gemini extensions, which is critical when
//! the extension is installed via `gemini extensions`. A `GEMINI_MARKETPLACE_HOME`
//! override remains available for isolated testing scenarios.

use std::env;
use std::path::{Path, PathBuf};

use directories::{BaseDirs, UserDirs};

use crate::marketplace::error::{MarketplaceError, Result};

const ENV_OVERRIDE: &str = "GEMINI_MARKETPLACE_HOME";
const GEMINI_CONFIG_ENV: &str = "GEMINI_CONFIG";
const DEFAULT_GEMINI_DIR: &str = ".gemini";
const EXTENSIONS_DIR: &str = "extensions";
const MARKETPLACE_DIR: &str = "marketplace";
const CACHE_SUBDIR: &str = "cache";
const CONFIG_SUBDIR: &str = "config";
const LOG_SUBDIR: &str = "logs";

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
                cache_dir: base.join(CACHE_SUBDIR),
                config_dir: base.join(CONFIG_SUBDIR),
                log_dir: base.join(LOG_SUBDIR),
            });
        }

        let base_config_dir = env::var(GEMINI_CONFIG_ENV)
            .map(PathBuf::from)
            .or_else(|_| {
                BaseDirs::new()
                    .map(|dirs| dirs.home_dir().join(DEFAULT_GEMINI_DIR))
                    .ok_or_else(|| {
                        MarketplaceError::Configuration(
                            "Unable to resolve home directory for Gemini config".into(),
                        )
                    })
            })?;

        let base = base_config_dir.join(EXTENSIONS_DIR).join(MARKETPLACE_DIR);
        Ok(Self {
            cache_dir: base.join(CACHE_SUBDIR),
            config_dir: base.join(CONFIG_SUBDIR),
            log_dir: base.join(LOG_SUBDIR),
        })
    }

    /// Construct configuration paths anchored to an explicit base directory (primarily for tests).
    #[cfg(test)]
    pub(crate) fn from_base_path(base: PathBuf) -> Self {
        Self {
            cache_dir: base.join(CACHE_SUBDIR),
            config_dir: base.join(CONFIG_SUBDIR),
            log_dir: base.join(LOG_SUBDIR),
        }
    }

    /// Directory containing cache files (per-source JSON payloads, TTL metadata).
    pub fn cache_dir(&self) -> &PathBuf {
        &self.cache_dir
    }

    /// Directory containing configuration files (user preferences, sources list).
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    /// Directory for logs or diagnostics produced by the extension.
    pub fn log_dir(&self) -> &PathBuf {
        &self.log_dir
    }

    /// User downloads directory, used as a default location when exporting manifests or reports.
    pub fn downloads_dir(&self) -> Option<PathBuf> {
        UserDirs::new().and_then(|ud| ud.download_dir().map(PathBuf::from))
    }

    /// Ensure required directories exist, creating them as needed.
    pub fn ensure_dirs(&self) -> Result<()> {
        std::fs::create_dir_all(&self.cache_dir)
            .map_err(|err| MarketplaceError::io(self.cache_dir.to_path_buf(), err))?;
        std::fs::create_dir_all(&self.config_dir)
            .map_err(|err| MarketplaceError::io(self.config_dir.to_path_buf(), err))?;
        std::fs::create_dir_all(&self.log_dir)
            .map_err(|err| MarketplaceError::io(self.log_dir.to_path_buf(), err))?;
        Ok(())
    }

    /// Root directory that Gemini CLI uses to store extensions.
    pub fn extensions_root(&self) -> Option<PathBuf> {
        self.config_dir
            .parent()
            .and_then(|marketplace_root| marketplace_root.parent())
            .map(PathBuf::from)
    }
}
