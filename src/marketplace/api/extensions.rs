//! REST routes that back the extensions CLI commands.

use axum::{
    http::StatusCode,
    routing::{get, post},
    Router,
};

pub fn router() -> Router {
    Router::new()
        .route("/marketplace/extensions", get(not_implemented))
        .route("/marketplace/extensions/:id", get(not_implemented))
        .route("/marketplace/cache/refresh", post(not_implemented))
}

async fn not_implemented() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_IMPLEMENTED,
        "extensions API not yet implemented",
    )
}
