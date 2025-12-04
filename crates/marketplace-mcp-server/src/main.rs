//! Main entry point for the Gemini Marketplace MCP stdio server.
//!
//! This server listens for requests on stdin, invokes the `gemini-marketplace` CLI,
//! and returns responses on stdout.

use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use marketplace_mcp_server::protocol::{ServerRequest, ServerResponse};
use marketplace_mcp_server::{invoke_cli, resolve_cli_path};
use serde_json::Deserializer;

/// Options for the Gemini Marketplace MCP stdio server.
#[derive(Debug, Parser)]
#[command(
    name = "marketplace-mcp-server",
    about = "Gemini Marketplace MCP stdio server"
)]
struct ServerOptions {
    /// Path to the `gemini-marketplace` CLI binary (defaults to a sibling binary or PATH).
    #[arg(long = "cli-bin")]
    cli_bin: Option<PathBuf>,
}

fn main() -> Result<()> {
    let opts = ServerOptions::parse();
    let cli_path = resolve_cli_path(opts.cli_bin)?;
    run_stdio_loop(&cli_path)
}

/// Runs the standard I/O loop, listening for requests and sending responses.
fn run_stdio_loop(cli_path: &Path) -> Result<()> {
    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();

    for result in Deserializer::from_reader(reader).into_iter::<ServerRequest>() {
        match result {
            Ok(request) => match invoke_cli(cli_path, &request.args) {
                Ok(response) => write_response(&mut writer, &response)?,
                Err(err) => {
                    let response = ServerResponse {
                        status: -1,
                        stdout: String::new(),
                        stderr: err.to_string(),
                    };
                    write_response(&mut writer, &response)?;
                }
            },
            Err(err) => {
                let response = ServerResponse {
                    status: -1,
                    stdout: String::new(),
                    stderr: format!("failed to parse request: {err}"),
                };
                write_response(&mut writer, &response)?;
            }
        }
    }

    Ok(())
}

fn write_response(writer: &mut impl Write, response: &ServerResponse) -> Result<()> {
    serde_json::to_writer(&mut *writer, response)?;
    writer
        .write_all(b"\n")
        .context("failed to flush response")?;
    writer.flush().context("failed to flush stdout")?;
    Ok(())
}
