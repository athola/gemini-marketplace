//! Library for the marketplace MCP server.
//!
//! This library provides the protocol for communication between the marketplace MCP server
//! and clients, as well as functions for resolving the CLI path and invoking CLI commands.

use anyhow::{Context, Result};
use protocol::ServerResponse;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod protocol {
    use serde::{Deserialize, Serialize};

    /// A request sent from the client to the server.
    #[derive(Debug, Deserialize, Serialize)]
    pub struct ServerRequest {
        pub args: Vec<String>,
    }

    /// A response sent from the server to the client.
    #[derive(Debug, Deserialize, Serialize)]
    pub struct ServerResponse {
        pub status: i32,
        pub stdout: String,
        pub stderr: String,
    }
}

/// Determines the path to the legacy `gemini-marketplace` CLI binary.
pub fn resolve_cli_path(override_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(path);
    }

    if let Some(env_path) = env::var_os("MARKETPLACE_CLI_BIN") {
        let candidate = PathBuf::from(env_path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    if let Ok(current) = env::current_exe() {
        if let Some(parent) = current.parent() {
            let sibling = parent.join("gemini-marketplace");
            if sibling.exists() {
                return Ok(sibling);
            }
        }
    }

    if let Some(path) = lookup_in_path("gemini-marketplace") {
        return Ok(path);
    }

    Ok(PathBuf::from("gemini-marketplace"))
}

/// Executes the CLI with the provided arguments and captures stdout and stderr.
pub fn invoke_cli(cli_path: &Path, args: &[String]) -> Result<ServerResponse> {
    let output = Command::new(cli_path)
        .args(args)
        .output()
        .with_context(|| format!("failed to exec {:?} with args {:?}", cli_path, args))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    Ok(ServerResponse {
        status: output.status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}

fn lookup_in_path(bin: &str) -> Option<PathBuf> {
    let exe = format!("{}{}", bin, env::consts::EXE_SUFFIX);
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(&exe);
            if candidate.is_file() {
                Some(candidate)
            } else {
                None
            }
        })
    })
}
