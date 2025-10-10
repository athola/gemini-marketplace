# Gemini Marketplace Extension

This repo holds a Gemini CLI extension written in Rust that discovers third-party extensions from GitHub-based catalogs. This project is built in Rust because the Gemini CLI already ships with a Rust toolchain. Additionally, async HTTP + filesystem work both benefit from Rust’s safety guarantees.

## Project Layout (Why it looks like this)

The source tree mirrors how Gemini CLI extensions load crates: a single binary entrypoint under `src/bin/` plus feature modules under `src/marketplace/`. Tests are split so unit tests stay fast and integration tests can spin up lightweight axum servers instead of hitting GitHub.

```
.
├── Cargo.toml
├── src/
│   ├── bin/marketplace.rs         # Clap-powered CLI entrypoint
│   └── marketplace/
│       ├── api/                   # Axum HTTP surfaces (placeholders)
│       ├── cache/                 # JSON cache store
│       ├── commands/              # CLI command handlers (stubs)
│       ├── config.rs              # Platform path helpers
│       ├── error.rs               # Shared error enum
│       ├── models/                # Domain + manifest types
│       └── services/              # Fetcher, preferences, etc.
├── tests/
│   ├── integration/               # Integration harness (WIP)
│   ├── unit/                      # Unit tests
│   └── common/                    # Shared fixtures
└── specs/001-build-a-gemini/      # Specification, plan, research, tasks
```

## Building

```bash
rustup override set 1.82.0
cargo build
```

## Run CLI Skeleton

```bash
cargo run -- list --help
```

## Testing Strategy

The plan leans on axum-backed harnesses for HTTP playback and `assert_cmd` for end-to-end CLI assertions. The `GEMINI_MARKETPLACE_HOME` environment override isolates test cache directories and does not rely upon modifying the Gemini config. Run `cargo test` once rustup can write temp files; the sandbox blocks it here.

## Toolchain

- Rust 1.82.0 (matches upstream ICU/idna requirements)
- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`
