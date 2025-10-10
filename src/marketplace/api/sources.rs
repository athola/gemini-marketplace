//! REST routes for source management.

use axum::{
    http::StatusCode,
    routing::{delete, get},
    Router,
};

pub fn router() -> Router {
    Router::new()
        .route(
            "/marketplace/sources",
            get(not_implemented).post(not_implemented),
        )
        .route("/marketplace/sources/:slug", delete(not_implemented))
}

async fn not_implemented() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_IMPLEMENTED,
        "sources API not yet implemented",
    )
}
