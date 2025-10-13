//! Source registry service for managing marketplace sources.
//!
//! Handles persistence, validation, and lifecycle of marketplace sources including
//! the default curated source per FR-005a.

use std::fs;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::marketplace::config::Config;
use crate::marketplace::models::domain::MarketplaceSource;

const SOURCES_FILE: &str = "sources.json";
const DEFAULT_SOURCE_URL: &str = "https://github.com/athola/gemini-marketplace";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourcesRegistry {
    sources: Vec<MarketplaceSource>,
}

#[derive(Clone)]
pub struct SourcesService {
    #[allow(dead_code)]
    config: Config,
    sources_path: PathBuf,
}

impl SourcesService {
    /// Create a new SourcesService with the given configuration.
    pub fn new() -> Result<Self> {
        let config = Config::new()?;
        config.ensure_dirs()?;
        let sources_path = config.config_dir().join(SOURCES_FILE);

        let service = Self {
            config,
            sources_path,
        };

        // Initialize with default source if sources file doesn't exist
        if !service.sources_path.exists() {
            service.initialize_default()?;
        }

        Ok(service)
    }

    /// Initialize the sources registry with the default curated source.
    fn initialize_default(&self) -> Result<()> {
        let default_url =
            Url::parse(DEFAULT_SOURCE_URL).context("Failed to parse default source URL")?;
        let default_source = MarketplaceSource::default_curated(default_url);

        let registry = SourcesRegistry {
            sources: vec![default_source],
        };

        self.save_registry(&registry)?;
        Ok(())
    }

    /// Load the sources registry from disk.
    fn load_registry(&self) -> Result<SourcesRegistry> {
        if !self.sources_path.exists() {
            // Return empty registry if file doesn't exist
            return Ok(SourcesRegistry {
                sources: Vec::new(),
            });
        }

        let contents =
            fs::read_to_string(&self.sources_path).context("Failed to read sources file")?;

        serde_json::from_str(&contents).context("Failed to parse sources file")
    }

    /// Save the sources registry to disk.
    fn save_registry(&self, registry: &SourcesRegistry) -> Result<()> {
        let contents =
            serde_json::to_string_pretty(registry).context("Failed to serialize sources")?;

        fs::write(&self.sources_path, contents).context("Failed to write sources file")?;

        Ok(())
    }

    /// Get the default curated source.
    pub fn get_default_source(&self) -> Result<MarketplaceSource> {
        let registry = self.load_registry()?;
        registry
            .sources
            .into_iter()
            .find(|s| s.default)
            .ok_or_else(|| anyhow::anyhow!("Default source not found in registry"))
    }

    /// List all configured sources.
    pub fn list_sources(&self) -> Result<Vec<MarketplaceSource>> {
        let registry = self.load_registry()?;
        Ok(registry.sources)
    }

    /// Add a new marketplace source.
    pub fn add_source(&self, source: MarketplaceSource) -> Result<()> {
        let mut registry = self.load_registry()?;

        // Check for duplicate slug
        if registry.sources.iter().any(|s| s.slug == source.slug) {
            bail!("Source with slug '{}' already exists", source.slug);
        }

        registry.sources.push(source);
        self.save_registry(&registry)?;
        Ok(())
    }

    /// Remove a marketplace source by slug.
    ///
    /// The default source cannot be removed, only disabled.
    pub fn remove_source(&self, slug: &str) -> Result<()> {
        let mut registry = self.load_registry()?;

        // Check if this is the default source
        if let Some(source) = registry.sources.iter().find(|s| s.slug == slug) {
            if source.default {
                bail!("Cannot remove default source; use disable_source instead");
            }
        }

        let original_len = registry.sources.len();
        registry.sources.retain(|s| s.slug != slug);

        if registry.sources.len() == original_len {
            bail!("Source '{}' not found", slug);
        }

        self.save_registry(&registry)?;
        Ok(())
    }

    /// Disable a marketplace source.
    pub fn disable_source(&self, slug: &str) -> Result<()> {
        let mut registry = self.load_registry()?;

        let source = registry
            .sources
            .iter_mut()
            .find(|s| s.slug == slug)
            .ok_or_else(|| anyhow::anyhow!("Source '{}' not found", slug))?;

        source.enabled = false;
        self.save_registry(&registry)?;
        Ok(())
    }

    /// Enable a marketplace source.
    pub fn enable_source(&self, slug: &str) -> Result<()> {
        let mut registry = self.load_registry()?;

        let source = registry
            .sources
            .iter_mut()
            .find(|s| s.slug == slug)
            .ok_or_else(|| anyhow::anyhow!("Source '{}' not found", slug))?;

        source.enabled = true;
        self.save_registry(&registry)?;
        Ok(())
    }
}

impl Default for SourcesService {
    fn default() -> Self {
        Self::new().expect("Failed to initialize SourcesService")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marketplace::models::domain::{SourceType, SyncStatus};
    use std::env;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_temp_home() -> TempDir {
        let temp = TempDir::new().expect("temp dir");
        let path = temp
            .path()
            .to_str()
            .expect("temp path utf8 for env override");
        env::set_var("GEMINI_MARKETPLACE_HOME", path);
        temp
    }

    fn clear_temp_home() {
        env::remove_var("GEMINI_MARKETPLACE_HOME");
    }

    #[test]
    fn new_service_bootstraps_default_source() {
        let _guard = env_lock().lock().unwrap();
        let temp = with_temp_home();
        let service = SourcesService::new().expect("service init");

        let sources = service.list_sources().expect("list sources");
        assert!(
            sources
                .iter()
                .any(|s| s.default && s.slug == "athola" && s.enabled),
            "expected default curated source present"
        );

        drop(service);
        clear_temp_home();
        temp.close().unwrap();
    }

    #[test]
    fn add_source_persists_across_service_instances() {
        let _guard = env_lock().lock().unwrap();
        let temp = with_temp_home();
        let service = SourcesService::new().expect("service init");

        let new_source = MarketplaceSource::new(
            "custom",
            "Custom",
            Url::parse("https://example.com/org/repo").unwrap(),
            SourceType::GitUrl,
            true,
            5,
        )
        .with_sync_status(SyncStatus::Idle);

        service.add_source(new_source.clone()).expect("add source");

        drop(service);

        // Re-initialize service to ensure persistence
        let service_again = SourcesService::new().expect("service re-init");
        let sources = service_again.list_sources().expect("list sources");

        assert!(
            sources.iter().any(|s| s.slug == new_source.slug),
            "expected custom source persisted"
        );

        drop(service_again);
        clear_temp_home();
        temp.close().unwrap();
    }
}
