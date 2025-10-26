use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use assert_cmd::prelude::*;
use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

#[path = "../../common/mod.rs"]
mod common;
use common::MarketplaceCommand;

struct TestEnv {
    _home: TempDir,
    _server: JoinHandle<()>,
}

impl TestEnv {
    async fn new(manifests: Vec<Value>) -> Result<Self> {
        let home = TempDir::new()?;
        std::env::set_var("GEMINI_MARKETPLACE_HOME", home.path());

        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let base = format!("http://{addr}");

        let manifests = Arc::new(manifests);
        let index_body = json!({
            "manifests": manifests
                .iter()
                .map(|entry| format!("{base}/manifests/{}.json", entry["name"].as_str().unwrap()))
                .collect::<Vec<_>>(),
        });

        let manifests_clone = manifests.clone();
        let app = Router::new()
            .route(
                "/index.json",
                get({
                    let body = index_body.clone();
                    move || async move { Json(body.clone()) }
                }),
            )
            .route(
                "/manifests/:name.json",
                get(move |axum::extract::Path(name): axum::extract::Path<String>| {
                    let manifests = manifests_clone.clone();
                    async move {
                        let entry = manifests
                            .iter()
                            .find(|manifest| manifest["name"].as_str() == Some(&name))
                            .expect("manifest for requested name");
                        Json(entry.clone())
                    }
                }),
            );

        let handle = tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .await
                .unwrap();
        });

        std::env::set_var(
            "GEMINI_MARKETPLACE_SOURCE_URL",
            format!("{base}/index.json"),
        );

        Ok(Self {
            _home: home,
            _server: handle,
        })
    }
}

impl Drop for TestEnv {
    fn drop(&mut self) {
        let _ = self._server.abort();
        std::env::remove_var("GEMINI_MARKETPLACE_SOURCE_URL");
        std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    }
}

fn make_manifest(name: &str) -> Value {
    json!({
        "name": name,
        "displayName": format!("{name} Extension"),
        "description": format!("{name} description"),
        "repository": format!("https://github.com/example/{name}"),
        "version": "1.0.0",
        "author": "Example Dev",
        "categories": ["analytics"],
        "tags": ["demo"],
        "compatibility": ["cli>=1.0"],
        "readme": "README",
    })
}

#[tokio::test]
async fn list_outputs_paginated_table() -> Result<()> {
    let _env = TestEnv::new(vec![
        make_manifest("alpha"),
        make_manifest("beta"),
        make_manifest("gamma"),
    ])
    .await?;

    let output = MarketplaceCommand::new()
        .arg("list")
        .run()
        .expect("command runs");

    output.expect_success();
    assert!(
        output.stdout.contains("Page 1 of"),
        "expected paginated banner in output, got:\n{}",
        output.stdout
    );

    Ok(())
}

#[tokio::test]
async fn list_supports_json_output_with_pagination_fields() -> Result<()> {
    let _env = TestEnv::new(vec![make_manifest("alpha")]).await?;

    let output = MarketplaceCommand::new()
        .args(["list", "--json"])
        .run()
        .expect("command runs");

    output.expect_success();
    let json: Value = serde_json::from_str(&output.stdout)?;
    assert!(json.get("page").is_some(), "expected `page` field in JSON payload");
    assert!(
        json.get("page_size").is_some(),
        "expected `page_size` field in JSON payload"
    );
    Ok(())
}

#[tokio::test]
async fn list_uses_cached_data_when_refresh_fails() -> Result<()> {
    let env = TestEnv::new(vec![make_manifest("alpha")]).await?;

    MarketplaceCommand::new()
        .arg("list")
        .run_assert_success();

    env._server.abort();
    std::env::remove_var("GEMINI_MARKETPLACE_SOURCE_URL");

    let output = MarketplaceCommand::new()
        .arg("list")
        .run()
        .expect("command runs");

    output.expect_success();
    assert!(
        output.stdout.contains("using cached data"),
        "expected cached data warning when refresh fails"
    );

    Ok(())
}
