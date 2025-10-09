# Gemini Marketplace Extension

A Rust-based Gemini CLI extension that surfaces third-party extensions from multiple sources, with caching, search, and source management.

## Project Layout

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

## Toolchain

- Rust 1.82.0
- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`

## Publishing with GitHub CLI

```
gh auth login
gh repo create <org-or-user>/gemini-marketplace --private --source . --remote origin
git push -u origin main
```
