use assert_cmd::prelude::*;
use assert_cmd::Command;
use serde_json::Value;
use std::process::{ExitStatus, Output};

/// Helper for invoking the `gemini-marketplace` CLI binary inside tests.
pub struct MarketplaceCommand {
    inner: Command,
}

impl MarketplaceCommand {
    /// Start a new CLI command builder with default environment tweaks.
    pub fn new() -> Self {
        let mut cmd = Command::cargo_bin("gemini-marketplace")
            .expect("binary `gemini-marketplace` should be built by tests");

        // Default to human-readable logs unless tests request JSON explicitly.
        cmd.env("GEMINI_MARKETPLACE_LOG", "human");

        Self { inner: cmd }
    }

    /// Append a single CLI argument.
    pub fn arg<S: AsRef<str>>(mut self, arg: S) -> Self {
        self.inner.arg(arg.as_ref());
        self
    }

    /// Append multiple CLI arguments.
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

    /// Convenience helper to toggle JSON output mode.
    pub fn json(self) -> Self {
        self.arg("--json")
    }

    /// Execute the command and capture stdout/stderr/status.
    pub fn run(mut self) -> CliOutput {
        let output = self
            .inner
            .output()
            .expect("failed to run `gemini-marketplace`");
        CliOutput::from(output)
    }

    /// Execute the command and assert success, returning captured output.
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
    /// Assert the command exited successfully (status code 0).
    pub fn expect_success(&self) {
        assert!(
            self.status.success(),
            "expected success, got status {status:?}\nstdout:\n{stdout}\nstderr:\n{stderr}",
            status = self.status,
            stdout = self.stdout,
            stderr = self.stderr,
        );
    }

    /// Parse stdout as JSON, panicking if invalid.
    pub fn stdout_json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .expect("expected stdout to be valid JSON response")
    }

    /// Extract table rows from stdout, splitting by the vertical bar separators.
    pub fn stdout_table_rows(&self) -> Vec<Vec<String>> {
        parse_table_rows(&self.stdout)
    }

    /// Convenience to check stderr is empty (aside from whitespace).
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

/// Parse stdout formatted as a Unicode table into row/cell values.
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

/// Convenience constructor mirroring the legacy helper for quick calls.
pub fn run_marketplace(args: &[&str]) -> CliOutput {
    MarketplaceCommand::new().args(args.iter().copied()).run()
}
