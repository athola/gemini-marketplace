#![allow(dead_code)]

use assert_cmd::Command;
use serde_json::Value;
use std::env;
use std::process::{ExitStatus, Output};

/// A helper for invoking the `gemini-marketplace` CLI binary inside tests.
pub struct MarketplaceCommand {
    inner: Command,
}

impl MarketplaceCommand {
    /// Starts a new CLI command builder with default environment tweaks.
    pub fn new() -> Self {
        let mut cmd = Command::cargo_bin("gemini-marketplace")
            .expect("binary `gemini-marketplace` should be built by tests");

        // Default to human-readable logs unless tests explicitly request another format.
        if env::var("GEMINI_MARKETPLACE_LOG").is_err() {
            cmd.env("GEMINI_MARKETPLACE_LOG", "human");
        }

        Self { inner: cmd }
    }

    /// Appends a single CLI argument.
    pub fn arg<S: AsRef<str>>(mut self, arg: S) -> Self {
        self.inner.arg(arg.as_ref());
        self
    }

    /// Appends multiple CLI arguments.
    pub fn args<S, I>(mut self, args: I) -> Self
    where
        S: AsRef<str>,
        I: IntoIterator<Item = S>,
    {
        for arg in args {
            self.inner.arg(arg.as_ref());
        }
        self
    }

    /// Toggles JSON output mode.
    pub fn json(self) -> Self {
        self.arg("--json")
    }

    /// Executes the command and captures stdout, stderr, and status.
    pub fn run(mut self) -> CliOutput {
        let output = self
            .inner
            .output()
            .expect("failed to run `gemini-marketplace`");
        CliOutput::from(output)
    }

    /// Executes the command and asserts success, returning captured output.
    pub fn run_assert_success(self) -> CliOutput {
        let output = self.run();
        output.expect_success();
        output
    }
}

/// Captured CLI output with convenience helpers for assertions.
#[derive(Debug)]
pub struct CliOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl CliOutput {
    /// Asserts that the command exited successfully (status code 0).
    pub fn expect_success(&self) {
        assert!(
            self.status.success(),
            "expected success, got status {status:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            status = self.status,
            stdout = self.stdout,
            stderr = self.stderr,
        );
    }

    /// Parses stdout as JSON, panicking if invalid.
    pub fn stdout_json(&self) -> Value {
        serde_json::from_str(&self.stdout).expect("expected stdout to be a valid JSON response")
    }

    /// Extracts table rows from stdout, splitting by the vertical bar separators.
    pub fn stdout_table_rows(&self) -> Vec<Vec<String>> {
        parse_table_rows(&self.stdout)
    }

    /// Checks that stderr is empty (aside from whitespace).
    pub fn expect_no_stderr(&self) {
        assert!(
            self.stderr.trim().is_empty(),
            "expected no stderr output, got:\n{}",
            self.stderr
        );
    }
}

impl From<Output> for CliOutput {
    fn from(output: Output) -> Self {
        CliOutput {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        }
    }
}

/// Parses stdout that is formatted as a Unicode table into row and cell values.
pub fn parse_table_rows(stdout: &str) -> Vec<Vec<String>> {
    stdout
        .lines()
        .filter(|line| line.contains('│'))
        .map(|line| {
            line.split('│')
                .map(str::trim)
                .filter(|cell| !cell.is_empty())
                .map(|cell| cell.to_string())
                .collect::<Vec<_>>()
        })
        .filter(|row| !row.is_empty())
        .collect()
}

/// A convenience constructor that mirrors the legacy helper for quick calls.
pub fn run_marketplace(args: &[&str]) -> CliOutput {
    MarketplaceCommand::new().args(args.iter().copied()).run()
}
