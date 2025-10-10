use std::net::SocketAddr;
use std::sync::Arc;

use assert_cmd::Command;
use axum::{Router, routing::get, Json};
use predicates::str::contains;
use tempfile::tempdir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;
use url::Url;

async fn spawn(app: Router) -> anyhow::Result<(SocketAddr, JoinHandle<()>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let handle = tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app.into_make_service())
            .await
            .unwrap();
    });
    Ok((addr, handle))
}

#[tokio::test]
async fn cli_list_outputs_json() -> anyhow::Result<()> {
    let manifests = Arc::new(vec![
        "http://placeholder/manifests/tool.json".to_string(),
    ]);
    let manifest_body = Arc::new(serde_json::json!({
        "name": "tool",
        "displayName": "Tool",
        "description": "Helpful tool",
        "repository": "https://github.com/example/tool",
        "version": "1.0.0",
        "author": "Example",
        "categories": ["Utility"],
        "tags": ["tool"],
        "compatibility": ["cli>=1.0"]
    }));

    let manifests_clone = manifests.clone();
    let manifest_clone = manifest_body.clone();

    let app = Router::new()
        .route(
            "/index.json",
            get(move || {
                let data = manifests_clone.clone();
                async move { Json((*data).clone()) }
            }),
        )
        .route(
            "/manifests/tool.json",
            get(move || {
                let data = manifest_clone.clone();
                async move { Json((*data).clone()) }
            }),
        );

    let (addr, handle) = spawn(app).await?;
    let base = format!("http://{addr}");

    let temp = tempdir()?;
    std::env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    std::env::set_var("GEMINI_MARKETPLACE_SOURCE_URL", format!("{base}/index.json"));

    Command::cargo_bin("gemini-marketplace")?
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(contains("\"namespace\": \"athola/tool\""));

    handle.abort();
    std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    std::env::remove_var("GEMINI_MARKETPLACE_SOURCE_URL");
    Ok(())
}
