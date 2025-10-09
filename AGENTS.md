# gemini-marketplace Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-09

## Active Technologies
- Rust 1.82.0 (MSRV aligned with Gemini CLI extension tooling) + reqwest + tokio (HTTP), serde/serde_json (manifests), directories (config paths), thiserror/anyhow (errors), indicatif (UX); dev: assert_cmd, insta, wiremock (001-build-a-gemini)

## Project Structure
```
src/
tests/
```

## Commands
cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style
Rust 1.82.0 (MSRV aligned with Gemini CLI extension tooling): Follow standard conventions

## Recent Changes
- 001-build-a-gemini: Added Rust 1.82.0 (MSRV aligned with Gemini CLI extension tooling) + reqwest + tokio (HTTP), serde/serde_json (manifests), directories (config paths), thiserror/anyhow (errors), indicatif (UX); dev: assert_cmd, insta, wiremock

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
