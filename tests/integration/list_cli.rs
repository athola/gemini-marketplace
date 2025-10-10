use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cli_list_outputs_json() -> anyhow::Result<()> {
    let server = MockServer::start().await;
    let manifests = vec![format!("{}/manifests/tool.json", server.uri())];
    Mock::given(method("GET")).and(path("/index.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifests))
        .mount(&server)
        .await;

    let manifest = serde_json::json!({
        "name": "tool",
        "displayName": "Tool",
        "description": "Helpful tool",
        "repository": "https://github.com/example/tool",
        "version": "1.0.0",
        "author": "Example",
        "categories": ["utility"],
        "tags": ["tool"],
        "compatibility": ["cli>=1.0"]
    });
    Mock::given(method("GET")).and(path("/manifests/tool.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifest))
        .mount(&server)
        .await;

    let temp = tempdir()?;
    std::env::set_var("GEMINI_MARKETPLACE_HOME", temp.path());
    std::env::set_var(
        "GEMINI_MARKETPLACE_SOURCE_URL",
        format!("{}/index.json", server.uri()),
    );

    Command::cargo_bin("gemini-marketplace")?
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(contains("\"namespace\": \"athola/tool\""));

    server.stop().await;
    std::env::remove_var("GEMINI_MARKETPLACE_HOME");
    std::env::remove_var("GEMINI_MARKETPLACE_SOURCE_URL");
    Ok(())
}
