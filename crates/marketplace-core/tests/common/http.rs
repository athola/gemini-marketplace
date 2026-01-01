#![allow(dead_code)]
//! An Axum-based fixture server for marketplace integration tests.
//!
//! This server serves static catalog and manifest JSON files from `tests/data/marketplace` while allowing tests to toggle GitHub-style rate limiting headers. The server binds to localhost on a random port and exposes helpers to retrieve the base URL and control rate-limit behavior.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use axum::body::Body;
use axum::extract::{Path as AxumPath, State};
use axum::http::{header, Response, StatusCode};
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;

#[derive(Clone)]
struct FixtureState {
    root: Arc<PathBuf>,
    control: FixtureControl,
}

/// Controls the rate limiting behavior of the fixture server.
#[derive(Clone, Default)]
struct FixtureControl {
    rate_limited: Arc<std::sync::atomic::AtomicBool>,
    reset_at: Arc<RwLock<Option<SystemTime>>>,
}

impl FixtureControl {
    /// Creates a new `FixtureControl` instance with rate limiting disabled.
    fn new() -> Self {
        Self {
            rate_limited: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            reset_at: Arc::new(RwLock::new(None)),
        }
    }

    /// Checks if the fixture server is currently rate limited.
    fn is_rate_limited(&self) -> bool {
        self.rate_limited.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Sets the rate limiting status and the time when the rate limit resets.
    async fn set_rate_limited(&self, value: bool, reset_at: Option<SystemTime>) {
        self.rate_limited
            .store(value, std::sync::atomic::Ordering::Relaxed);
        let mut guard = self.reset_at.write().await;
        *guard = reset_at;
    }

    /// Returns GitHub-style rate limit headers if rate limiting is active.
    ///
    /// The headers include `x-ratelimit-reset` (epoch seconds) and `x-ratelimit-remaining` (always "0").
    async fn rate_limit_headers(&self) -> Option<(String, String)> {
        if !self.is_rate_limited() {
            return None;
        }
        let guard = self.reset_at.read().await;
        guard
            .map(|ts| {
                let epoch = ts
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                (
                    epoch.to_string(),
                    "0".to_string(), // remaining requests
                )
            })
            .or_else(|| {
                Some((
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                    "0".to_string(),
                ))
            })
    }
}

/// An embedded Axum server that exposes marketplace fixtures.
pub struct FixtureServer {
    base_url: String,
    control: FixtureControl,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

impl FixtureServer {
    /// Launches a fixture server bound to localhost, serving files from the `tests/data/marketplace` directory.
    pub async fn start() -> anyhow::Result<Self> {
        let root = fixture_root()?;
        let control = FixtureControl::new();
        let state = FixtureState {
            root: Arc::new(root),
            control: control.clone(),
        };

        let router = Router::new()
            .route("/*path", get(serve_fixture))
            .with_state(state);

        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

        let handle = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await
                .expect("fixture server failed");
        });

        Ok(Self {
            base_url: format!("http://{addr}"),
            control,
            shutdown: Some(shutdown_tx),
            handle,
        })
    }

    /// Returns the base URL that tests can use to fetch fixtures.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Toggles rate-limiting behavior.
    ///
    /// When enabled, requests return HTTP 403 with GitHub-style rate limit headers.
    /// `reset_in` controls the reset time.
    pub async fn set_rate_limited(&self, enabled: bool, reset_in: Option<Duration>) {
        let reset_at = reset_in.map(|duration| SystemTime::now() + duration);
        self.control.set_rate_limited(enabled, reset_at).await;
    }

    /// Converts a logical fixture path (e.g., `curated/index.json`) into a full URL served by this fixture server.
    pub fn url_for(&self, fixture_path: &str) -> String {
        format!("{}/{fixture_path}", self.base_url.trim_end_matches('/'))
    }
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
        // Ensure the background task finishes to avoid cancellation warnings.
        // We do not block on the handle because tests might drop on panic.
        self.handle.abort();
    }
}

fn fixture_root() -> anyhow::Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests").join("data").join("marketplace");
    if !path.exists() {
        anyhow::bail!("fixture directory missing: {}", path.display());
    }
    Ok(path)
}

async fn serve_fixture(
    State(state): State<FixtureState>,
    AxumPath(request_path): AxumPath<String>,
) -> Response<Body> {
    if state.control.is_rate_limited() {
        let mut response = Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("rate limited"))
            .expect("failed to build response");
        if let Some((reset, remaining)) = state.control.rate_limit_headers().await {
            let headers = response.headers_mut();
            headers.insert(
                "x-ratelimit-reset",
                header::HeaderValue::from_str(&reset)
                    .unwrap_or_else(|_| header::HeaderValue::from_static("0")),
            );
            headers.insert(
                "x-ratelimit-remaining",
                header::HeaderValue::from_str(&remaining)
                    .unwrap_or_else(|_| header::HeaderValue::from_static("0")),
            );
        }
        return response;
    }

    let sanitized = request_path.trim_start_matches('/');
    let path = state.root.join(sanitized);
    let file_result = tokio::fs::read(path).await;
    match file_result {
        Ok(contents) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(contents))
            .expect("failed to build success response"),
        Err(err) => {
            let status = if err.kind() == std::io::ErrorKind::NotFound {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Response::builder()
                .status(status)
                .body(Body::from(format!("fixture error: {err}")))
                .expect("failed to build error response")
        }
    }
}
