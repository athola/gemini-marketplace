//! Lightweight HTTP server harness exposing marketplace endpoints.
//!
//! Uses `axum` to provide REST routes that mirror the CLI commands, enabling
//! integration with other tools or remote control of the marketplace extension.

use std::net::SocketAddr;

use axum::Router;
use tokio::{net::TcpListener, task::JoinHandle};

use crate::marketplace::api::{extensions, sources, status};
use crate::marketplace::error::{MarketplaceError, Result};

pub struct ApiServer {
    router: Router,
}

impl ApiServer {
    pub fn new() -> Self {
        Self {
            router: build_router(),
        }
    }

    pub fn with_router(router: Router) -> Self {
        Self { router }
    }

    pub fn router(&self) -> Router {
        self.router.clone()
    }

    pub async fn run(self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(|err| MarketplaceError::network("api_bind", err, Some(addr.to_string())))?;
        axum::serve(listener, self.router.into_make_service())
            .await
            .map_err(|err| MarketplaceError::network("api_serve", err, None))
    }

    pub fn spawn(self, addr: SocketAddr) -> JoinHandle<Result<()>> {
        tokio::spawn(async move { self.run(addr).await })
    }
}

impl Default for ApiServer {
    fn default() -> Self {
        Self::new()
    }
}

fn build_router() -> Router {
    Router::new()
        .merge(extensions::router())
        .merge(sources::router())
        .merge(status::router())
}
