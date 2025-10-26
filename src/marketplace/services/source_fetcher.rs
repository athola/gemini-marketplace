//! Marketplace source fetcher responsible for retrieving manifests, respecting
//! caching, and handling GitHub rate limits.

use std::borrow::Cow;
use std::time::{Duration, SystemTime};

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ETAG, USER_AGENT};
use reqwest::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio::time::sleep;

use crate::marketplace::cache::store::CacheStore;
use crate::marketplace::config::Config;
use crate::marketplace::error::{MarketplaceError, Result};
use crate::marketplace::models::domain::{
    Extension, ExtensionId, InstallStatus, MarketplaceSource, RateLimitWindow, ValidationSummary,
};
use crate::marketplace::models::manifest::ExtensionManifest;
use crate::marketplace::services::preferences::PreferencesService;

/// Fetches manifests from marketplace sources, respecting cache TTL and GitHub
/// rate limits.
pub struct SourceFetcher {
    client: Client,
    cache: CacheStore,
    prefs: PreferencesService,
}

impl SourceFetcher {
    pub fn new(config: &Config, prefs: PreferencesService) -> Result<Self> {
        let client = Client::builder()
            .user_agent("gemini-marketplace-extension/0.1.0")
            .build()
            .map_err(|err| MarketplaceError::network("client_build", err, None))?;
        let cache = CacheStore::new(config)?;
        Ok(Self {
            client,
            cache,
            prefs,
        })
    }

    pub async fn sync_source(&self, source: &MarketplaceSource) -> Result<Vec<Extension>> {
        let ttl = Duration::from_secs((self.prefs.cache_ttl_hours() as u64) * 3600);
        if let Some(snapshot) = self.cache.load(&source.slug)? {
            if snapshot.entry.expires_at > SystemTime::now() {
                return Ok(snapshot.extensions);
            }
        }

        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("gemini-marketplace-extension/0.1.0"),
        );

        let response = self
            .client
            .get(source.url.as_str())
            .headers(headers)
            .send()
            .await
            .map_err(|err| {
                MarketplaceError::network("catalog_fetch", err, Some(source.url.to_string()))
            })?;

        if response.status().as_u16() == 403 {
            if let Some(window) = Self::extract_rate_limit(&source.slug, &response) {
                return Err(MarketplaceError::RateLimited {
                    source_slug: source.slug.clone(),
                    reset_at: window
                        .reset_at
                        .map(|instant| humantime::format_rfc3339_seconds(instant).to_string()),
                });
            }
        }

        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());

        let body = response.text().await.map_err(|err| {
            MarketplaceError::network("catalog_read_body", err, Some(source.url.to_string()))
        })?;

        let manifest_urls = CatalogBody::parse(&body).map_err(|err| {
            MarketplaceError::Configuration(format!("Invalid catalog for {}: {err}", source.slug))
        })?;
        let mut extensions = Vec::new();
        for manifest_url in manifest_urls {
            let manifest_url_owned = manifest_url.clone();
            let manifest_response = self
                .client
                .get(manifest_url.as_str())
                .send()
                .await
                .map_err(|err| {
                    MarketplaceError::network(
                        "manifest_fetch",
                        err,
                        Some(manifest_url_owned.clone()),
                    )
                })?;
            if Self::check_rate_limited(&manifest_response) {
                let window = Self::extract_rate_limit(&source.slug, &manifest_response)
                    .unwrap_or_else(|| RateLimitWindow::new(source.slug.as_str(), None, Some(0)));
                sleep(Duration::from_secs(5)).await;
                return Err(MarketplaceError::RateLimited {
                    source_slug: source.slug.clone(),
                    reset_at: window
                        .reset_at
                        .map(|instant| humantime::format_rfc3339_seconds(instant).to_string()),
                });
            }
            let manifest_body = manifest_response.text().await.map_err(|err| {
                MarketplaceError::network(
                    "manifest_read_body",
                    err,
                    Some(manifest_url_owned.clone()),
                )
            })?;
            let (manifest, validation) =
                ExtensionManifest::from_str(&manifest_body, &manifest_url_owned)?;
            if !validation.is_clean() {
                for warning in &validation.warnings {
                    tracing::warn!(
                        target: "marketplace::manifest",
                        source = %source.slug,
                        field = %warning.field,
                        message = %warning.message,
                        "Manifest warning [{}]: {} ({})",
                        source.slug,
                        warning.field,
                        warning.message
                    );
                }
            }
            let checksum = format!("{:x}", Sha256::digest(manifest_body.as_bytes()));
            extensions.push(Self::to_extension(source, &manifest, ttl, checksum)?);
        }

        self.cache.save(&source.slug, &extensions, etag, ttl)?;
        Ok(extensions)
    }

    pub fn cached_extensions(&self, source_slug: &str) -> Result<Option<Vec<Extension>>> {
        Ok(self
            .cache
            .load(source_slug)?
            .map(|snapshot| snapshot.extensions))
    }

    fn to_extension(
        source: &MarketplaceSource,
        manifest: &ExtensionManifest,
        ttl: Duration,
        checksum: String,
    ) -> Result<Extension> {
        let now = SystemTime::now();
        let expires_at = now.checked_add(ttl).ok_or_else(|| {
            MarketplaceError::Configuration(format!(
                "Cache TTL overflow for source '{}': TTL duration too large",
                source.slug
            ))
        })?;
        Ok(Extension {
            id: ExtensionId::new(&source.slug, &manifest.name),
            source_slug: source.slug.clone(),
            extension_slug: manifest.name.clone(),
            display_name: match &manifest.display_name {
                Some(display_name) => Cow::Owned(display_name.clone()),
                None => Cow::Owned(manifest.name.clone()),
            },
            summary: Cow::Owned(manifest.description.clone()),
            repository_url: manifest.repository.clone(),
            homepage_url: manifest.homepage.clone(),
            documentation_url: manifest.documentation.clone(),
            version: manifest.version.clone(),
            author: match &manifest.author {
                Some(author) => Cow::Owned(author.clone()),
                None => Cow::Owned(String::new()),
            },
            license: manifest
                .license
                .as_ref()
                .map(|license| Cow::Owned(license.clone())),
            categories: manifest.categories.clone(),
            tags: manifest.tags.clone(),
            compatibility: manifest.compatibility.clone(),
            install_status: InstallStatus::Unknown,
            manifest_checksum: checksum,
            readme_excerpt: manifest
                .readme
                .as_ref()
                .map(|readme| Cow::Owned(readme.clone())),
            last_synced_at: Some(now),
            cache_expires_at: Some(expires_at),
            validation_summary: ValidationSummary::new_pending(now),
            manifest_path: Some(".".to_string()),
        })
    }

    fn extract_rate_limit(
        source_slug: &str,
        response: &reqwest::Response,
    ) -> Option<RateLimitWindow> {
        let headers = response.headers();
        let reset_at = headers
            .get("x-ratelimit-reset")
            .and_then(|value| value.to_str().ok())
            .and_then(|epoch| epoch.parse::<u64>().ok())
            .map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds));
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u32>().ok());
        let _limit = headers
            .get("x-ratelimit-limit")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u32>().ok());

        Some(RateLimitWindow::new(source_slug, reset_at, remaining))
    }

    fn check_rate_limited(response: &reqwest::Response) -> bool {
        response.status().as_u16() == 403
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CatalogBody {
    Array(Vec<String>),
    Object { manifests: Vec<String> },
}

impl CatalogBody {
    fn parse(body: &str) -> Result<Vec<String>> {
        let parsed: CatalogBody = serde_json::from_str(body).map_err(|err| {
            MarketplaceError::Configuration(format!("invalid catalog payload: {err}"))
        })?;
        Ok(match parsed {
            CatalogBody::Array(urls) => urls,
            CatalogBody::Object { manifests } => manifests,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_array_catalog() {
        let urls = CatalogBody::parse("[\"a.json\", \"b.json\"]").unwrap();
        assert_eq!(urls.len(), 2);
    }

    #[test]
    fn parses_object_catalog() {
        let urls = CatalogBody::parse("{\"manifests\":[\"a.json\"]}").unwrap();
        assert_eq!(urls, vec!["a.json".to_string()]);
    }

    #[test]
    fn invalid_payload_returns_error() {
        let err = CatalogBody::parse("not json").unwrap_err();
        assert!(err.to_string().contains("invalid catalog payload"));
    }
}
