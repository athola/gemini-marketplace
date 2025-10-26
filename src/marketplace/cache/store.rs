//! JSON cache store for marketplace data.
//!
//! Persists per-source extension payloads and metadata to enable offline
//! browsing. Each cache file stores normalized extensions, expiration, and an
//! optional etag for conditional requests.

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};

use crate::marketplace::config::Config;
use crate::marketplace::error::{MarketplaceError, Result};
use crate::marketplace::models::domain::{CacheEntry, Extension};

#[derive(Debug)]
pub struct CacheStore {
    root: PathBuf,
    locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl CacheStore {
    /// Construct a cache store rooted at the provided directory (testing helper).
    pub fn for_root(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }
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
        let root = config.cache_dir().to_path_buf();
        fs::create_dir_all(&root).map_err(|err| MarketplaceError::io(root.to_path_buf(), err))?;
        Ok(Self {
            root,
            locks: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn cache_path(&self, source_slug: &str) -> PathBuf {
        self.root.join(format!("{source_slug}.json"))
    }

    pub fn batch_path(&self, source_slug: &str, batch_index: u32) -> PathBuf {
        let dir = self.root.join(source_slug);
        dir.join(format!("batch-{batch_index:03}.json"))
    }

    pub fn save_batch(
        &self,
        source_slug: &str,
        batch_index: u32,
        extensions: &[Extension],
        etag: Option<String>,
        ttl: Duration,
    ) -> Result<()> {
        self.with_slug_guard(source_slug, || {
            let path = self.batch_path(source_slug, batch_index);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| MarketplaceError::io(parent.to_path_buf(), err))?;
            }

            let fetched_at = SystemTime::now();
            let expires_at = fetched_at.checked_add(ttl).ok_or_else(|| {
                MarketplaceError::Configuration(format!(
                    "Cache TTL overflow for source '{}': TTL duration too large",
                    source_slug
                ))
            })?;

            let mut payload = extensions.to_vec();
            for extension in &mut payload {
                extension.cache_expires_at = Some(expires_at);
                extension.last_synced_at = Some(fetched_at);
            }

            let _ = ensure_manifest_checksums(&mut payload, source_slug)?;

            let cache_file = CacheFile {
                fetched_at,
                expires_at,
                etag,
                extensions: payload,
            };

            let file =
                File::create(&path).map_err(|err| MarketplaceError::io(path.to_path_buf(), err))?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &cache_file).map_err(|err| {
                MarketplaceError::InvalidManifest {
                    repository: format!("{source_slug}/batch-{batch_index:03}.json"),
                    reason: format!("unable to serialize cache: {err}"),
                }
            })
        })
    }

    pub fn load_batch(&self, source_slug: &str, batch_index: u32) -> Result<Option<CacheSnapshot>> {
        self.with_slug_guard(source_slug, || {
            let path = self.batch_path(source_slug, batch_index);
            if !path.exists() {
                return Ok(None);
            }

            let file =
                File::open(&path).map_err(|err| MarketplaceError::io(path.to_path_buf(), err))?;
            let reader = BufReader::new(file);
            let cache_file: CacheFile = serde_json::from_reader(reader).map_err(|err| {
                MarketplaceError::InvalidManifest {
                    repository: format!("{source_slug}/batch-{batch_index:03}.json"),
                    reason: format!("invalid cache file JSON: {err}"),
                }
            })?;

            let mut extensions = cache_file.extensions;
            let checksums = ensure_manifest_checksums(&mut extensions, source_slug)?;

            let entry = CacheEntry {
                source_slug: source_slug.to_string(),
                batch_index,
                manifest_checksum: checksums.join(";"),
                payload_path: path.to_string_lossy().into_owned(),
                fetched_at: cache_file.fetched_at,
                expires_at: cache_file.expires_at,
                extension_ids: extensions.iter().map(|ext| ext.id.clone()).collect(),
                etag: cache_file.etag.clone(),
            };

            Ok(Some(CacheSnapshot { entry, extensions }))
        })
    }

    pub fn load(&self, source_slug: &str) -> Result<Option<CacheSnapshot>> {
        self.with_slug_guard(source_slug, || {
            let path = self.cache_path(source_slug);
            if !path.exists() {
                return Ok(None);
            }
            let file =
                File::open(&path).map_err(|err| MarketplaceError::io(path.to_path_buf(), err))?;
            let reader = BufReader::new(file);
            let cache_file: CacheFile = serde_json::from_reader(reader).map_err(|err| {
                MarketplaceError::InvalidManifest {
                    repository: source_slug.to_string(),
                    reason: format!("invalid cache file JSON: {err}"),
                }
            })?;

            let extension_ids = cache_file
                .extensions
                .iter()
                .map(|ext| ext.id.clone())
                .collect();

            let mut extensions = cache_file.extensions;
            let checksums = ensure_manifest_checksums(&mut extensions, source_slug)?;

            let entry = CacheEntry {
                source_slug: source_slug.to_string(),
                batch_index: 0,
                manifest_checksum: checksums.join(";"),
                payload_path: path.to_string_lossy().into_owned(),
                fetched_at: cache_file.fetched_at,
                expires_at: cache_file.expires_at,
                extension_ids,
                etag: cache_file.etag.clone(),
            };

            Ok(Some(CacheSnapshot { entry, extensions }))
        })
    }

    pub fn save(
        &self,
        source_slug: &str,
        extensions: &[Extension],
        etag: Option<String>,
        ttl: Duration,
    ) -> Result<()> {
        self.with_slug_guard(source_slug, || {
            let path = self.cache_path(source_slug);
            let fetched_at = SystemTime::now();
            let expires_at = fetched_at.checked_add(ttl).ok_or_else(|| {
                MarketplaceError::Configuration(format!(
                    "Cache TTL overflow for source '{}': TTL duration too large",
                    source_slug
                ))
            })?;

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|err| MarketplaceError::io(parent.to_path_buf(), err))?;
            }

            let mut payload = extensions.to_vec();
            for extension in &mut payload {
                extension.cache_expires_at = Some(expires_at);
                extension.last_synced_at = Some(fetched_at);
            }

            let _ = ensure_manifest_checksums(&mut payload, source_slug)?;

            let cache_file = CacheFile {
                fetched_at,
                expires_at,
                etag,
                extensions: payload,
            };

            let file =
                File::create(&path).map_err(|err| MarketplaceError::io(path.to_path_buf(), err))?;
            let writer = BufWriter::new(file);
            serde_json::to_writer_pretty(writer, &cache_file).map_err(|err| {
                MarketplaceError::InvalidManifest {
                    repository: source_slug.to_string(),
                    reason: format!("unable to serialize cache: {err}"),
                }
            })
        })
    }

    pub fn invalidate(&self, source_slug: &str) -> Result<()> {
        self.with_slug_guard(source_slug, || {
            let path = self.cache_path(source_slug);
            if path.exists() {
                fs::remove_file(&path).map_err(|err| MarketplaceError::io(path, err))?;
            }
            Ok(())
        })
    }
}

impl CacheStore {
    fn lock_for_slug(&self, slug: &str) -> Arc<Mutex<()>> {
        let mut map = self.locks.lock().unwrap();
        map.entry(slug.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    fn with_slug_guard<T, F>(&self, slug: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let slug_lock = self.lock_for_slug(slug);
        let _guard = slug_lock.lock().unwrap();
        f()
    }
}

fn ensure_manifest_checksums(extensions: &mut [Extension], context: &str) -> Result<Vec<String>> {
    let mut checksums = Vec::with_capacity(extensions.len());
    for extension in extensions.iter_mut() {
        if extension.manifest_checksum.is_empty() {
            // Serialize with empty checksum (since it's already empty)
            let serialized = serde_json::to_vec(&extension).map_err(|err| {
                MarketplaceError::InvalidManifest {
                    repository: context.to_string(),
                    reason: format!("unable to serialize extension for checksum: {err}"),
                }
            })?;
            let mut hasher = Sha256::new();
            hasher.update(serialized);
            extension.manifest_checksum = format!("{:x}", hasher.finalize());
        }
        checksums.push(extension.manifest_checksum.clone());
    }
    Ok(checksums)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marketplace::models::{Extension, ExtensionId, InstallStatus, ValidationSummary};
    use semver::Version;
    use std::time::{Duration, SystemTime};
    use tempfile::TempDir;
    use url::Url;

    fn sample_extension(source: &str, slug: &str) -> Extension {
        Extension::new(
            ExtensionId::new(source, slug),
            format!("{slug} extension"),
            "Great extension for testing",
            Url::parse("https://example.com/foo").unwrap(),
            Version::parse("1.0.0").unwrap(),
            "Example Author",
            source,
            ["analytics"],
            ["Gemini CLI >=1.0"],
            InstallStatus::NotInstalled,
            Some(SystemTime::UNIX_EPOCH),
            ValidationSummary::new_pending(SystemTime::UNIX_EPOCH),
            format!("repos/{slug}"),
            Some("README".into()),
        )
    }

    fn store_with_temp_root() -> (TempDir, CacheStore) {
        let temp = TempDir::new().expect("temp dir");
        let store = CacheStore::for_root(temp.path().to_path_buf());
        (temp, store)
    }

    #[test]
    fn save_batch_persists_ttl_and_metadata() {
        let (_tmp, store) = store_with_temp_root();
        let extension = sample_extension("curated", "awesome");
        let ttl = Duration::from_secs(120);

        store
            .save_batch(
                "curated",
                0,
                &[extension.clone()],
                Some("etag-123".into()),
                ttl,
            )
            .expect("save batch");

        let snapshot = store
            .load_batch("curated", 0)
            .expect("load batch")
            .expect("snapshot should exist");

        assert_eq!(snapshot.entry.source_slug, "curated");
        assert_eq!(snapshot.entry.batch_index, 0);
        let ttl_diff = snapshot
            .entry
            .expires_at
            .duration_since(snapshot.entry.fetched_at)
            .expect("expires after fetched");
        assert!((ttl_diff.as_secs_f64() - ttl.as_secs_f64()).abs() < 1.0);
        assert_eq!(snapshot.entry.etag.as_deref(), Some("etag-123"));
        assert!(snapshot
            .entry
            .payload_path
            .ends_with("curated/batch-000.json"));
        assert_eq!(snapshot.extensions.len(), 1);
        let loaded = &snapshot.extensions[0];
        assert_eq!(
            loaded.readme_excerpt.as_ref().map(|cow| cow.as_ref()),
            Some("README")
        );
        assert_eq!(loaded.cache_expires_at, Some(snapshot.entry.expires_at));
        assert_eq!(loaded.last_synced_at, Some(snapshot.entry.fetched_at));
    }

    #[test]
    fn load_batch_returns_none_when_missing() {
        let (_tmp, store) = store_with_temp_root();
        let snapshot = store
            .load_batch("curated", 42)
            .expect("load should succeed even when missing");
        assert!(snapshot.is_none());
    }

    #[test]
    fn save_multiple_batches_produces_independent_cache_files() {
        let (_tmp, store) = store_with_temp_root();
        let ext_a = sample_extension("curated", "awesome");
        let ext_b = sample_extension("curated", "beta");

        store
            .save_batch("curated", 0, &[ext_a], None, Duration::from_secs(60))
            .expect("save batch 0");
        store
            .save_batch("curated", 1, &[ext_b], None, Duration::from_secs(60))
            .expect("save batch 1");

        let first = store
            .load_batch("curated", 0)
            .expect("load batch 0")
            .expect("snapshot");
        let second = store
            .load_batch("curated", 1)
            .expect("load batch 1")
            .expect("snapshot");

        assert_ne!(first.entry.payload_path, second.entry.payload_path);
        assert_eq!(first.entry.batch_index, 0);
        assert_eq!(second.entry.batch_index, 1);
        assert_eq!(first.extensions[0].extension_slug, "awesome");
        assert_eq!(second.extensions[0].extension_slug, "beta");
    }
}
