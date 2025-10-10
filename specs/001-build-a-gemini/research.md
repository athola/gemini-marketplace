# Phase 0 Research — Gemini CLI Extension Marketplace

## Minimum Supported Rust Version
Decision: Target Rust 1.82.0 as the minimum supported version (MSRV) for the marketplace extension.  
Rationale: Rust 1.82.0 is currently required by transitive ICU/idna dependencies (via `reqwest`) and remains in the stable channel, ensuring compatibility with crate ecosystem expectations while keeping tooling consistent. Basing the extension on 1.82.0 prevents incompatible dependency updates and aligns with upstream requirements.  
Alternatives considered: Holding at Rust 1.80.0 was rejected because modern ICU crates no longer support it; nightly toolchains were rejected because they violate stability expectations for end users.

## Core Crate Dependencies
Decision: Use `reqwest` + `tokio` for async HTTP, `serde`/`serde_json` for manifest parsing, `directories` for cross-platform config paths, `thiserror` + `anyhow` for error handling, `indicatif` for progress feedback, and `assert_cmd` + `insta` (dev-dependencies) for testing.  
Rationale: This stack mirrors idiomatic Rust CLI patterns, provides proven GitHub API support, and satisfies spec requirements for rate-limit handling, caching, and UX feedback without reinventing primitives. Each crate is well-supported and compatible with Rust 1.82.  
Alternatives considered: `ureq` (blocking) was rejected because it complicates concurrent downloads and rate-limit queuing; custom path resolution was rejected because `directories` already encapsulates OS differences; bespoke progress rendering was rejected in favor of `indicatif` for maintainability.

## Cache Storage Format & Eviction
Decision: Persist cached marketplace payloads as JSON files under `$GEMINI_CONFIG/extensions/marketplace/cache/*.json`, with per-source files keyed by slug and metadata file tracking TTL + etags. TTL defaults to 24h with manual refresh support; background cleanup removes entries whose TTL has expired during refresh.  
Rationale: JSON aligns with manifest schema (`gemini-extension.json`), keeps troubleshooting simple, and allows incremental per-source updates without rewriting a monolithic cache. Leveraging the Gemini config directory honors user expectations and avoids pollution of project workspaces.  
Alternatives considered: SQLite was rejected as overkill for the expected dataset size; in-memory-only caches were rejected because offline browsing is a requirement; a single cache file was rejected to simplify partial invalidation.

## Testing Strategy
Decision: Adopt layered testing with `cargo test` for unit coverage, `assert_cmd`-powered CLI contract tests for marketplace commands, and `insta` snapshots for deterministic render output. Network interactions use axum-backed test servers so we can simulate rate limits and catalog shifts without pulling binaries.  
Rationale: The spec emphasizes testable acceptance criteria and accurate display formatting; combining unit + contract + snapshot testing enforces behavior without relying on live APIs. Axum lets us script failure modes (rate limits, malformed manifests) while staying in-process.  
Alternatives considered: Live GitHub integration tests were rejected due to rate-limit unpredictability; relying solely on unit tests was rejected because CLI UX regressions would be missed; external mocking frameworks were avoided to reduce MSRV and dependency churn.

## Target Platform Support
Decision: Officially support Linux and macOS hosts (matching Gemini CLI’s primary targets) and treat Windows 11+ as best-effort by leveraging the `directories` crate, avoiding symlinks, and documenting PowerShell-specific notes in quickstart.  
Rationale: The Gemini CLI primarily advertises Linux/macOS compatibility today, yet Windows developers benefit from inclusion; using cross-platform crates minimizes platform-specific branches while keeping expectations transparent.  
Alternatives considered: Declaring Linux-only support was rejected because many developers run the CLI on macOS; promising full Windows parity without constraints was rejected because Git credential helper behavior differs and needs explicit documentation.
