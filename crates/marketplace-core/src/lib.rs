//! Core library for the Gemini CLI marketplace extension.
//!
//! ## Modules
//!
//! -   [`cli`]: Thin re-exports that provide a stable public CLI surface. This layer mirrors the `gemini-marketplace` binary and is **not** a stable SDK.
//! -   [`marketplace`]: Internal services, config, and models used by both the CLI and the MCP server. Breaking changes here follow SemVer.
//! -   [`telemetry`]: Shared logging and metrics helpers. Treat this as experimental; names may change between minor releases.
//!
//! See the top-level `README.md` and `CHANGELOG.md` for stability guarantees across releases.

/// CLI wrappers exported for integration tests and downstream tooling.
pub mod cli;
/// Core services, config, and domain models for the marketplace.
pub mod marketplace;
/// Shared telemetry helpers (logging and metrics). APIs may evolve rapidly.
pub mod telemetry;
