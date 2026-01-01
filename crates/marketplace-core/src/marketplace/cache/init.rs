//! Filesystem layout initializer for the marketplace cache and config dirs.
//!
//! All runtime data for the marketplace extension is stored beneath
//! `$GEMINI_CONFIG/extensions/marketplace/` (or the override provided by
//! `GEMINI_MARKETPLACE_HOME`). This module centralizes the directory creation
//! logic so commands and services can rely on a consistent structure without
//! duplicating `std::fs::create_dir_all` calls.

use std::path::{Path, PathBuf};

use crate::marketplace::config::Config;
use crate::marketplace::error::Result;

/// Describes the on-disk layout for marketplace state.
#[derive(Debug, Clone)]
pub struct CacheLayout {
    root: PathBuf,
    cache_dir: PathBuf,
    config_dir: PathBuf,
    log_dir: PathBuf,
}

impl CacheLayout {
    /// Ensure directories exist for the provided configuration and return the layout.
    pub fn ensure(config: &Config) -> Result<Self> {
        config.ensure_dirs()?;
        let root = config
            .config_dir()
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| config.config_dir().clone());
        Ok(Self {
            root,
            cache_dir: config.cache_dir().clone(),
            config_dir: config.config_dir().clone(),
            log_dir: config.log_dir().clone(),
        })
    }

    /// Root directory beneath `$GEMINI_CONFIG/extensions/marketplace/`.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Directory storing cached manifests and metadata.
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Directory storing preferences, sources, and refresh queue files.
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Directory storing logs/telemetry exports.
    pub fn log_dir(&self) -> &Path {
        &self.log_dir
    }
}

/// Convenience wrapper used by most call sites that only need to ensure the
/// layout is present. The returned layout can be ignored when only the side
/// effect matters.
pub fn ensure_layout(config: &Config) -> Result<CacheLayout> {
    CacheLayout::ensure(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn ensure_layout_creates_directories() {
        let temp = TempDir::new().expect("tempdir");
        let base = temp.path().join("marketplace");
        std::env::set_var("GEMINI_MARKETPLACE_HOME", &base);

        let config = Config::new().expect("config");
        let layout = ensure_layout(&config).expect("layout");

        assert!(layout.cache_dir().exists());
        assert!(layout.config_dir().exists());
        assert!(layout.log_dir().exists());
        assert!(layout.root().ends_with("marketplace"));

        std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    }
}
