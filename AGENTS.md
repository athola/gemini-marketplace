# gemini-marketplace Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-09

## Active Technologies
- Rust 1.82.0 with reqwest + tokio (HTTP), serde/serde_json (manifests), directories (config paths), thiserror/anyhow (errors), indicatif (UX); dev stack: assert_cmd, insta, predicates, humantime, axum for test servers (001-build-a-gemini)
- Per-source JSON cache under `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata and manual refresh (001-build-a-gemini)
- Rust 1.82.0 (MSRV locked in constitution) + `reqwest`, `tokio`, `serde`/`serde_json`, `directories`, `thiserror`/`anyhow`, `indicatif` (001-build-a-gemini)
- Local filesystem cache under `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata (001-build-a-gemini)
- Rust 1.82.0 (MSRV locked by constitution) + `reqwest`, `tokio`, `serde`/`serde_json`, `directories`, `thiserror`, `anyhow`, `indicatif`, `semver`, `clap`, `axum`, `sha2` (001-build-a-gemini)
- Local filesystem cache per source under `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata (001-build-a-gemini)

## Project Structure
```
src/
tests/
```

## Commands
- `cargo test` (run locally once rustup can create temp files)
- `cargo clippy --all-targets -- -D warnings`

## Code Style
Rust 1.82.0 (MSRV aligned with Gemini CLI extension tooling): Follow standard conventions

## Recent Changes
- 001-build-a-gemini: Added Rust 1.82.0 (MSRV locked by constitution) + `reqwest`, `tokio`, `serde`/`serde_json`, `directories`, `thiserror`, `anyhow`, `indicatif`, `semver`, `clap`, `axum`, `sha2`
- 001-build-a-gemini: Added Rust 1.82.0 (MSRV locked in constitution) + `reqwest`, `tokio`, `serde`/`serde_json`, `directories`, `thiserror`/`anyhow`, `indicatif`
- 001-build-a-gemini: Locked MSRV to 1.82.0 (ICU/idna dependency requirement) and documented the test stack (assert_cmd, predicates, humantime, axum)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
