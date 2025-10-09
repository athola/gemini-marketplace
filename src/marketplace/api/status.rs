//! REST routes for marketplace status reporting.

use axum::{http::StatusCode, routing::get, Router};

pub fn router() -> Router {
    Router::new().route("/marketplace/status", get(not_implemented))
}

async fn not_implemented() -> (StatusCode, &'static str) {
    (
        StatusCode::NOT_IMPLEMENTED,
        "status API not yet implemented",
    )
}
