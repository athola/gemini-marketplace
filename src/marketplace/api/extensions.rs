//! REST routes that back the extensions CLI commands.

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::marketplace::services::catalog::{CatalogService, ListRequest};

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

/// Handler for GET /marketplace/extensions
async fn list_extensions(
    State(state): State<ExtensionsState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<ExtensionListResponse>, (StatusCode, String)> {
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
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

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

async fn not_implemented_list() -> (StatusCode, Json<ExtensionListResponse>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(ExtensionListResponse {
            extensions: vec![],
            warnings: vec![
                "Extensions API not yet fully initialized. Use router_with_state() to enable."
                    .to_string(),
            ],
        }),
    )
}

async fn not_implemented() -> (StatusCode, &'static str) {
    (StatusCode::NOT_IMPLEMENTED, "endpoint not yet implemented")
}
