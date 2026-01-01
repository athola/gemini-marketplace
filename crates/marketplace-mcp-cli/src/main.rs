//! Main entry point for the Gemini Marketplace MCP CLI harness.
//!
//! This CLI acts as a developer tool to interact with the marketplace MCP server,
//! mirroring how the Gemini CLI would communicate with it.

use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use clap::Parser;
use marketplace_mcp_server::protocol::{ServerRequest, ServerResponse};

/// Options for the Gemini Marketplace MCP CLI harness.
#[derive(Debug, Parser)]
#[command(
    name = "marketplace-mcp-cli",
    about = "Developer harness for the Gemini Marketplace MCP server"
)]
struct CliOptions {
    /// Path to the `marketplace-mcp-server` binary (defaults to a sibling binary or PATH).
    #[arg(long = "server-bin")]
    server_bin: Option<PathBuf>,

    /// Arguments to forward to the `gemini marketplace` CLI (e.g., `list --json`).
    #[arg(required = true, trailing_var_arg = true)]
    args: Vec<String>,
}

fn main() -> Result<()> {
    let opts = CliOptions::parse();
    let server_path = resolve_server_path(opts.server_bin)?;
    let mut child = Command::new(server_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .context("failed to spawn marketplace-mcp-server")?;

    {
        let mut stdin = child
            .stdin
            .take()
            .context("failed to acquire server stdin")?;
        serde_json::to_writer(&mut stdin, &ServerRequest { args: opts.args })?;
        stdin.write_all(b"\n")?;
    }

    let stdout = child
        .stdout
        .take()
        .context("failed to acquire server stdout")?;
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .context("failed to read server response")?;

    if line.trim().is_empty() {
        bail!("server exited without response");
    }

    let response: ServerResponse = serde_json::from_str(line.trim())?;
    print!("{}", response.stdout);
    if !response.stderr.is_empty() {
        eprint!("{}", response.stderr);
    }

    let status = child.wait()?;
    let exit_code = if status.success() {
        response.status
    } else {
        status.code().unwrap_or(1)
    };

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}

fn resolve_server_path(override_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        return Ok(path);
    }

    if let Some(env_path) = env::var_os("MARKETPLACE_MCP_SERVER_BIN") {
        let candidate = PathBuf::from(env_path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    if let Ok(current) = env::current_exe() {
        if let Some(parent) = current.parent() {
            let sibling = parent.join("marketplace-mcp-server");
            if sibling.exists() {
                return Ok(sibling);
            }
        }
    }

    if let Some(path) = lookup_in_path("marketplace-mcp-server") {
        return Ok(path);
    }

    Ok(PathBuf::from("marketplace-mcp-server"))
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
