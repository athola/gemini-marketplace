use assert_cmd::cargo::cargo_bin;
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn cli_help_roundtrip() {
    let cli_bin = cargo_bin("gemini-marketplace");
    let server_bin = cargo_bin("marketplace-mcp-server");

    Command::cargo_bin("marketplace-mcp-cli")
        .expect("binary built")
        .arg("list")
        .arg("--help")
        .env("MARKETPLACE_CLI_BIN", cli_bin)
        .env("MARKETPLACE_MCP_SERVER_BIN", server_bin)
        .assert()
        .success()
        .stdout(predicate::str::contains("gemini marketplace list"));
}
