//! REST routes that back the extensions CLI commands.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;

use crate::marketplace::error::MarketplaceError;
use crate::marketplace::services::catalog::{CatalogService, ListRequest};
use humantime;

/// Shared application state for the extensions API
#[derive(Clone)]
pub struct ExtensionsState {
    pub catalog: Arc<CatalogService>,
}

/// Query parameters for GET /marketplace/extensions
#[derive(Debug, Deserialize)]
pub struct ListQueryParams {
    /// Search term to filter extensions by name or description
    pub search: Option<String>,
    /// Filter by category
    pub category: Option<String>,
    /// Filter by source slug
    pub source: Option<String>,
    /// Show only installed extensions
    #[serde(default)]
    pub installed_only: bool,
    /// Use pre-fetch filtering to reduce API calls
    #[serde(default)]
    pub prefetch_filter: bool,
}

/// JSON response for GET /marketplace/extensions
#[derive(Debug, Serialize)]
pub struct ExtensionListResponse {
    pub extensions: Vec<ExtensionSummary>,
    pub warnings: Vec<String>,
}

/// Structured error response for API endpoints
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub request_id: Option<String>,
    pub timestamp: String,
}

/// Detailed error information
#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Option<ErrorDetails>,
}

/// Additional error details based on error type
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ErrorDetails {
    /// Rate limiting information
    RateLimit {
        source_slug: String,
        reset_at: Option<String>,
        retry_after: Option<u64>,
    },
    /// Network error information
    Network {
        operation: Option<String>,
        url: Option<String>,
    },
    /// I/O error information
    Io {
        path: Option<String>,
        operation: Option<String>,
    },
    /// Configuration error information
    Configuration {
        parameter: Option<String>,
        value: Option<String>,
    },
    /// Generic error with additional context
    Generic {
        context: std::collections::HashMap<String, serde_json::Value>,
    },
}

/// Extension summary for list view
#[derive(Debug, Serialize)]
pub struct ExtensionSummary {
    pub namespace: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub source: String,
    pub installed: bool,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

pub fn router() -> Router {
    Router::new()
        .route("/marketplace/extensions", get(not_implemented_list))
        .route("/marketplace/extensions/:id", get(not_implemented))
        .route("/marketplace/cache/refresh", post(not_implemented))
}

/// Router with injected state for full functionality
pub fn router_with_state(catalog: Arc<CatalogService>) -> Router {
    let state = ExtensionsState { catalog };
    Router::new()
        .route("/marketplace/extensions", get(list_extensions))
        .route("/marketplace/extensions/:id", get(not_implemented))
        .route("/marketplace/cache/refresh", post(not_implemented))
        .with_state(state)
}

/// Maps anyhow::Error to appropriate HTTP status code and structured error response
fn map_error_to_response(
    error: anyhow::Error,
    request_id: Option<String>,
) -> (StatusCode, Json<ErrorResponse>) {
    use humantime::format_rfc3339_seconds;
    let timestamp = format_rfc3339_seconds(std::time::SystemTime::now());

    // Try to extract MarketplaceError from anyhow::Error, otherwise handle as generic error
    let (status_code, error_detail) =
        if let Some(marketplace_err) = error.downcast_ref::<MarketplaceError>() {
            map_marketplace_error_to_response(marketplace_err)
        } else {
            map_generic_error_to_response(&error)
        };

    let error_response = ErrorResponse {
        error: error_detail,
        request_id,
        timestamp: timestamp.to_string(),
    };

    (status_code, Json(error_response))
}

/// Maps MarketplaceError to appropriate HTTP status code and error detail
fn map_marketplace_error_to_response(error: &MarketplaceError) -> (StatusCode, ErrorDetail) {
    match error {
        MarketplaceError::ExtensionNotFound { id } => (
            StatusCode::NOT_FOUND,
            ErrorDetail {
                code: "EXTENSION_NOT_FOUND".to_string(),
                message: format!("Extension '{id}' not found"),
                details: None,
            },
        ),

        MarketplaceError::SourceNotFound { slug } => (
            StatusCode::NOT_FOUND,
            ErrorDetail {
                code: "SOURCE_NOT_FOUND".to_string(),
                message: format!("Source '{slug}' not found"),
                details: None,
            },
        ),

        MarketplaceError::RateLimited {
            source_slug,
            reset_at,
        } => {
            let retry_after = reset_at
                .as_ref()
                .and_then(|s| humantime::parse_rfc3339(s).ok())
                .and_then(|dt| {
                    let now = std::time::SystemTime::now();
                    if dt > now {
                        dt.duration_since(now).ok().map(|d| d.as_secs())
                    } else {
                        Some(0)
                    }
                });

            (
                StatusCode::TOO_MANY_REQUESTS,
                ErrorDetail {
                    code: "RATE_LIMITED".to_string(),
                    message: format!("Rate limit exceeded for source '{}'", source_slug),
                    details: Some(ErrorDetails::RateLimit {
                        source_slug: source_slug.clone(),
                        reset_at: reset_at.clone(),
                        retry_after,
                    }),
                },
            )
        }

        MarketplaceError::AuthenticationRequired { slug } => (
            StatusCode::UNAUTHORIZED,
            ErrorDetail {
                code: "AUTHENTICATION_REQUIRED".to_string(),
                message: format!("Authentication required for source '{}'", slug),
                details: None,
            },
        ),

        MarketplaceError::InvalidManifest { repository, reason } => (
            StatusCode::UNPROCESSABLE_ENTITY,
            ErrorDetail {
                code: "INVALID_MANIFEST".to_string(),
                message: format!(
                    "Invalid manifest for repository '{}': {}",
                    repository, reason
                ),
                details: Some(ErrorDetails::Generic {
                    context: {
                        let mut map = HashMap::new();
                        map.insert(
                            "repository".to_string(),
                            serde_json::Value::String(repository.clone()),
                        );
                        map.insert(
                            "reason".to_string(),
                            serde_json::Value::String(reason.clone()),
                        );
                        map
                    },
                }),
            },
        ),

        MarketplaceError::Network {
            operation,
            source,
            url,
        } => (
            StatusCode::BAD_GATEWAY,
            ErrorDetail {
                code: "NETWORK_ERROR".to_string(),
                message: source.to_string(),
                details: Some(ErrorDetails::Network {
                    operation: Some(operation.clone()),
                    url: url.clone(),
                }),
            },
        ),

        MarketplaceError::Io { path, source } => (
            StatusCode::INTERNAL_SERVER_ERROR,
            ErrorDetail {
                code: "IO_ERROR".to_string(),
                message: format!("I/O error: {}", source),
                details: Some(ErrorDetails::Io {
                    path: Some(path.to_string_lossy().to_string()),
                    operation: Some("file operation".to_string()),
                }),
            },
        ),

        MarketplaceError::Configuration(message) => {
            let (parameter, value) = extract_config_context(message);
            (
                StatusCode::BAD_REQUEST,
                ErrorDetail {
                    code: "CONFIGURATION_ERROR".to_string(),
                    message: "Configuration error".to_string(),
                    details: Some(ErrorDetails::Configuration { parameter, value }),
                },
            )
        }

        MarketplaceError::Todo => (
            StatusCode::NOT_IMPLEMENTED,
            ErrorDetail {
                code: "NOT_IMPLEMENTED".to_string(),
                message: "Operation not yet implemented".to_string(),
                details: None,
            },
        ),
    }
}

/// Maps generic anyhow::Error to appropriate HTTP status code and error detail
fn map_generic_error_to_response(error: &anyhow::Error) -> (StatusCode, ErrorDetail) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        ErrorDetail {
            code: "INTERNAL_ERROR".to_string(),
            message: "An internal error occurred".to_string(),
            details: Some(ErrorDetails::Generic {
                context: {
                    let mut map = HashMap::new();
                    map.insert(
                        "error".to_string(),
                        serde_json::Value::String(error.to_string()),
                    );
                    map
                },
            }),
        },
    )
}

/// Extract configuration parameter and value from error message
fn extract_config_context(message: &str) -> (Option<String>, Option<String>) {
    // Try to parse common configuration error patterns
    if message.contains("Cache TTL overflow") {
        (
            Some("cache_ttl_hours".to_string()),
            Some("overflow".to_string()),
        )
    } else if message.contains("invalid catalog") {
        (Some("catalog_url".to_string()), Some("invalid".to_string()))
    } else {
        (None, None)
    }
}

/// Handler for GET /marketplace/extensions
async fn list_extensions(
    State(state): State<ExtensionsState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<ExtensionListResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Generate a simple request ID (in production, you might want a proper UUID)
    let request_id = Some(format!(
        "req-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    ));

    // Build the list request from query parameters
    let request = ListRequest {
        search: params.search.as_deref(),
        category: params.category.as_deref(),
        source: params.source.as_deref(),
        installed_only: params.installed_only,
        prefetch_filter: params.prefetch_filter,
    };

    // Call the catalog service
    let response = state
        .catalog
        .list(&request)
        .await
        .map_err(|e| map_error_to_response(e, request_id.clone()))?;

    // Transform the response to API format
    let extensions = response
        .entries
        .into_iter()
        .map(|entry| ExtensionSummary {
            namespace: entry.namespace,
            display_name: entry.display_name,
            description: entry.description,
            version: entry.version,
            source: entry.source,
            installed: entry.installed,
            categories: entry.categories,
            tags: entry.tags,
        })
        .collect();

    Ok(Json(ExtensionListResponse {
        extensions,
        warnings: response.warnings,
    }))
}

async fn not_implemented_list() -> (StatusCode, Json<ErrorResponse>) {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "NOT_IMPLEMENTED".to_string(),
            message: "Extensions API not yet fully initialized. Use router_with_state() to enable."
                .to_string(),
            details: None,
        },
        request_id: None,
        timestamp: humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string(),
    };
    (StatusCode::NOT_IMPLEMENTED, Json(error_response))
}

async fn not_implemented() -> (StatusCode, Json<ErrorResponse>) {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "NOT_IMPLEMENTED".to_string(),
            message: "endpoint not yet implemented".to_string(),
            details: None,
        },
        request_id: None,
        timestamp: humantime::format_rfc3339_seconds(std::time::SystemTime::now()).to_string(),
    };
    (StatusCode::NOT_IMPLEMENTED, Json(error_response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{routing::get, Router as TestRouter};
    use reqwest::Client;
    use serde_json::Value;
    use std::sync::{Arc, Mutex, OnceLock};
    use tempfile::TempDir;
    use tokio::net::TcpListener;

    use crate::marketplace::config::Config;
    use crate::marketplace::models::domain::{
        MarketplaceSource, OutputFormat, SearchMode, SourceType, SyncStatus, UserPreferences,
    };
    use crate::marketplace::services::catalog::CatalogService;
    use crate::marketplace::services::preferences::PreferencesService;
    use crate::marketplace::services::source_fetcher::SourceFetcher;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[tokio::test]
    async fn list_extensions_returns_structured_error_on_fatal_failure() {
        let _guard = env_lock().lock().unwrap();
        let temp = TempDir::new().expect("temp dir");
        std::env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());

        let app = TestRouter::new().route("/catalog.json", get(|| async { "not json" }));
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(async move {
            axum::serve(listener, app.into_make_service())
                .await
                .expect("server run");
        });

        let config = Config::new().expect("config");
        config.ensure_dirs().expect("ensure dirs");
        let prefs = PreferencesService::new(UserPreferences {
            cache_ttl_hours: 1,
            auto_refresh_on_launch: false,
            search_mode: SearchMode::LocalFilter,
            output_format: OutputFormat::Table,
        });
        let fetcher = SourceFetcher::new(&config, prefs.clone()).expect("fetcher");
        let source_url = format!("http://{}/catalog.json", addr);
        let source = MarketplaceSource::new(
            "fatal",
            "Fatal",
            source_url.parse().expect("url"),
            SourceType::GithubRepo,
            true,
            1,
        )
        .with_sync_status(SyncStatus::Idle);
        let catalog = CatalogService::new(fetcher, prefs, vec![source]);
        let router = router_with_state(Arc::new(catalog));
        let api_listener = TcpListener::bind("127.0.0.1:0").await.expect("api bind");
        let api_addr = api_listener.local_addr().expect("api addr");
        tokio::spawn(async move {
            axum::serve(api_listener, router.into_make_service())
                .await
                .expect("api server run");
        });

        let client = Client::new();
        let response = client
            .get(format!("http://{}/marketplace/extensions", api_addr))
            .send()
            .await
            .expect("request");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let payload: Value = response.json().await.expect("json");
        assert_eq!(payload["error"]["code"], "CONFIGURATION_ERROR");

        std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    }
}
