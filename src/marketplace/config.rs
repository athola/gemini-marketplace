//! Configuration helpers for marketplace directories.
//!
//! Resolves platform-specific paths for cache, preferences, and logs using the
//! `directories` crate so the extension remains portable across Linux, macOS,
//! and Windows targets.

use std::path::PathBuf;

use directories::{ProjectDirs, UserDirs};

const QUALIFIER: &str = "com";
const ORGANIZATION: &str = "gemini";
const APPLICATION: &str = "marketplace-extension";

#[derive(Debug, Clone)]
pub struct Config {
    project_dirs: ProjectDirs,
}

impl Config {
    /// Construct configuration paths, returning an error when system user dirs
    /// cannot be resolved (rare but indicates a misconfigured environment).
    pub fn new() -> anyhow::Result<Self> {
        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION)
            .ok_or_else(|| anyhow::anyhow!("Unable to resolve project directories"))?;
        Ok(Self { project_dirs })
    }

    /// Directory containing cache files (per-source JSON payloads, TTL metadata).
    pub fn cache_dir(&self) -> PathBuf {
        self.project_dirs.cache_dir().to_path_buf()
    }

    /// Directory containing configuration files (user preferences, sources list).
    pub fn config_dir(&self) -> PathBuf {
        self.project_dirs.config_dir().to_path_buf()
    }

    /// Directory for logs or diagnostics produced by the extension.
    pub fn log_dir(&self) -> PathBuf {
        self.project_dirs.data_local_dir().join("logs")
    }

    /// User downloads directory, used as fallback when exporting manifests or reports.
    pub fn downloads_dir(&self) -> Option<PathBuf> {
        UserDirs::new().and_then(|ud| ud.download_dir().map(PathBuf::from))
    }

    /// Ensure required directories exist, creating them as needed.
    pub fn ensure_dirs(&self) -> anyhow::Result<()> {
        std::fs::create_dir_all(self.cache_dir())?;
        std::fs::create_dir_all(self.config_dir())?;
        std::fs::create_dir_all(self.log_dir())?;
        Ok(())
    }
}
