//! JSON cache store for marketplace data.
//!
//! Persists per-source extension payloads and metadata to enable offline
//! browsing. Each cache file stores normalized extensions, expiration, and an
//! optional etag for conditional requests.

use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::marketplace::config::Config;
use crate::marketplace::error::{MarketplaceError, Result};
use crate::marketplace::models::domain::{CacheEntry, Extension};

#[derive(Debug)]
pub struct CacheStore {
    root: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheFile {
    #[serde(with = "humantime_serde")]
    pub fetched_at: SystemTime,
    #[serde(with = "humantime_serde")]
    pub expires_at: SystemTime,
    pub etag: Option<String>,
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone)]
pub struct CacheSnapshot {
    pub entry: CacheEntry,
    pub extensions: Vec<Extension>,
}

impl CacheStore {
    pub fn new(config: &Config) -> Result<Self> {
        let root = config.cache_dir();
        fs::create_dir_all(&root).map_err(|err| MarketplaceError::io(root.clone(), err))?;
        Ok(Self { root })
    }

    pub fn cache_path(&self, source_slug: &str) -> PathBuf {
        self.root.join(format!("{source_slug}.json"))
    }

    pub fn load(&self, source_slug: &str) -> Result<Option<CacheSnapshot>> {
        let path = self.cache_path(source_slug);
        if !path.exists() {
            return Ok(None);
        }
        let file = File::open(&path).map_err(|err| MarketplaceError::io(path.clone(), err))?;
        let reader = BufReader::new(file);
        let cache_file: CacheFile =
            serde_json::from_reader(reader).map_err(|err| MarketplaceError::InvalidManifest {
                repository: source_slug.to_string(),
                reason: format!("invalid cache file JSON: {err}"),
            })?;

        let extension_ids = cache_file
            .extensions
            .iter()
            .map(|ext| ext.id.clone())
            .collect();

        let entry = CacheEntry {
            source_slug: source_slug.to_string(),
            manifest_checksum: cache_file
                .extensions
                .iter()
                .map(|ext| ext.manifest_checksum.clone())
                .collect::<Vec<_>>()
                .join(";"),
            payload_path: path.to_string_lossy().into_owned(),
            fetched_at: cache_file.fetched_at,
            expires_at: cache_file.expires_at,
            extension_ids,
            etag: cache_file.etag.clone(),
        };

        Ok(Some(CacheSnapshot {
            entry,
            extensions: cache_file.extensions,
        }))
    }

    pub fn save(
        &self,
        source_slug: &str,
        extensions: &[Extension],
        etag: Option<String>,
        ttl: Duration,
    ) -> Result<()> {
        let path = self.cache_path(source_slug);
        let fetched_at = SystemTime::now();
        let expires_at = fetched_at
            .checked_add(ttl)
            .ok_or_else(|| MarketplaceError::Configuration("Cache TTL overflow".into()))?;

        let mut payload = extensions.to_vec();
        for extension in &mut payload {
            extension.cache_expires_at = Some(expires_at);
            extension.last_synced_at = Some(fetched_at);
        }

        let cache_file = CacheFile {
            fetched_at,
            expires_at,
            etag,
            extensions: payload,
        };

        let file = File::create(&path).map_err(|err| MarketplaceError::io(path.clone(), err))?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &cache_file).map_err(|err| {
            MarketplaceError::InvalidManifest {
                repository: source_slug.to_string(),
                reason: format!("unable to serialize cache: {err}"),
            }
        })
    }

    pub fn invalidate(&self, source_slug: &str) -> Result<()> {
        let path = self.cache_path(source_slug);
        if path.exists() {
            fs::remove_file(&path).map_err(|err| MarketplaceError::io(path, err))?;
        }
        Ok(())
    }
}
