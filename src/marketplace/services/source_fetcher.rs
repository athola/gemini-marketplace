//! Marketplace source fetcher responsible for retrieving manifests, respecting
//! caching, and handling GitHub rate limits.

use std::time::{Duration, SystemTime};

use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, USER_AGENT};
use reqwest::Client;
use tokio::time::sleep;

use crate::marketplace::cache::store::CacheStore;
use crate::marketplace::config::Config;
use crate::marketplace::error::{MarketplaceError, Result};
use crate::marketplace::models::domain::{
    Extension, ExtensionId, InstallStatus, MarketplaceSource, RateLimitWindow,
};
use crate::marketplace::models::manifest::ExtensionManifest;
use crate::marketplace::services::preferences::PreferencesService;

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
            .map_err(|err| MarketplaceError::Network(format!("client build failed: {err}")))?;
        let cache = CacheStore::new(config)?;
        Ok(Self {
            client,
            cache,
            prefs,
        })
    }

    pub async fn sync_source(&self, source: &MarketplaceSource) -> Result<Vec<Extension>> {
        let ttl = Duration::from_secs((self.prefs.cache_ttl_hours() as u64) * 3600);
        if let Some(cache_entry) = self.cache.load(&source.slug)? {
            if cache_entry.expires_at > SystemTime::now() {
                // TODO: map cache_entry to denormalized extension list
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
            .get(source.url.clone())
            .headers(headers)
            .send()
            .await
            .map_err(|err| MarketplaceError::Network(format!("fetch failed: {err}")))?;

        if response.status().as_u16() == 403 {
            if let Some(window) = Self::extract_rate_limit(&source.slug, &response) {
                return Err(MarketplaceError::RateLimited {
                    source: source.slug.clone(),
                    reset_at: window
                        .reset_at
                        .map(|instant| humantime::format_rfc3339_seconds(instant).to_string())
                        .unwrap_or_else(|| "unknown".into()),
                });
            }
        }

        let body = response
            .text()
            .await
            .map_err(|err| MarketplaceError::Network(format!("read body failed: {err}")))?;
        // TODO: parse repository listing format into manifest URLs
        let manifest_urls: Vec<String> = vec![];
        let mut extensions = Vec::new();
        for manifest_url in manifest_urls {
            let manifest_response =
                self.client
                    .get(manifest_url.clone())
                    .send()
                    .await
                    .map_err(|err| {
                        MarketplaceError::Network(format!("manifest fetch failed: {err}"))
                    })?;
            if Self::check_rate_limited(&manifest_response) {
                let window = Self::extract_rate_limit(&source.slug, &manifest_response)
                    .unwrap_or_else(|| RateLimitWindow {
                        source_slug: source.slug.clone(),
                        reset_at: None,
                        remaining_requests: Some(0),
                        limit: None,
                    });
                sleep(Duration::from_secs(5)).await;
                return Err(MarketplaceError::RateLimited {
                    source: source.slug.clone(),
                    reset_at: window
                        .reset_at
                        .map(|instant| humantime::format_rfc3339_seconds(instant).to_string())
                        .unwrap_or_else(|| "unknown".into()),
                });
            }
            let manifest_body = manifest_response.text().await.map_err(|err| {
                MarketplaceError::Network(format!("manifest body read failed: {err}"))
            })?;
            let (manifest, validation) =
                ExtensionManifest::from_str(&manifest_body, &manifest_url)?;
            if !validation.is_clean() {
                return Err(MarketplaceError::InvalidManifest {
                    repository: manifest_url,
                    reason: format!(
                        "warnings detected: {}",
                        validation
                            .warnings
                            .iter()
                            .map(|w| format!("{}: {}", w.field, w.message))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                });
            }
            extensions.push(Self::to_extension(source, &manifest));
        }

        self.cache.save(&source.slug, &extensions, None, ttl)?;
        Ok(extensions)
    }

    fn to_extension(source: &MarketplaceSource, manifest: &ExtensionManifest) -> Extension {
        Extension {
            id: ExtensionId::new(&source.slug, &manifest.name),
            source_slug: source.slug.clone(),
            extension_slug: manifest.name.clone(),
            display_name: manifest
                .display_name
                .clone()
                .unwrap_or_else(|| manifest.name.clone()),
            summary: manifest.description.clone(),
            repository_url: manifest.repository.clone(),
            homepage_url: manifest.homepage.clone(),
            documentation_url: manifest.documentation.clone(),
            version: manifest.version.clone(),
            author: manifest.author.clone().unwrap_or_default(),
            license: manifest.license.clone(),
            categories: manifest.categories.clone(),
            tags: manifest.tags.clone(),
            compatibility: manifest.compatibility.clone(),
            install_status: InstallStatus::Unknown,
            manifest_checksum: String::new(),
            readme_excerpt: manifest.readme.clone(),
            last_synced_at: Some(SystemTime::now()),
            cache_expires_at: None,
        }
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
        let limit = headers
            .get("x-ratelimit-limit")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u32>().ok());

        Some(RateLimitWindow {
            source_slug: source_slug.to_string(),
            reset_at,
            remaining_requests: remaining,
            limit,
        })
    }

    fn check_rate_limited(response: &reqwest::Response) -> bool {
        response.status().as_u16() == 403
    }
}
