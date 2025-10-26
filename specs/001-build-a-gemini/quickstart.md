# Quickstart – Gemini CLI Extension Marketplace

## Prerequisites
- Rust 1.82.0 toolchain (`rustup toolchain install 1.82.0`)
- Gemini CLI installed and accessible on `PATH`
- GitHub credentials (token or SSH agent) configured via OS credential helpers
- Network access for initial catalog sync (offline mode supported afterwards)

## Environment Setup
```bash
git checkout 001-build-a-gemini
rustup override set 1.82.0
cargo install just --locked # optional command runner
```

Set `GEMINI_CONFIG` if you use a non-default config directory:
```bash
export GEMINI_CONFIG="$HOME/.config/gemini"
```

## Running the Extension Locally
```bash
cargo run --bin marketplace -- list --help
cargo run --bin marketplace -- sources add https://github.com/athola/gemini-marketplace
```

The CLI spins up a local API server on demand; commands remain single-shot unless you pass
`--interactive`.

## Test-First Workflow
1. Create failing tests before implementing a story:
   ```bash
   cargo test tests::integration::marketplace_list::shows_paginated_results -- --nocapture
   ```
2. Implement the code to satisfy the failing tests.
3. Run the full suite to ensure regressions are caught:
   ```bash
   cargo fmt
   cargo clippy --all-targets -- -D warnings
   cargo test --all-targets
   ```

## Working with Cached Data
- Cache lives under `$GEMINI_CONFIG/extensions/marketplace/`.
- Refresh the cache and monitor countdowns:
  ```bash
  cargo run --bin marketplace -- cache refresh
  ```
- Adjust TTL:
  ```bash
  cargo run --bin marketplace -- cache ttl set 12
  ```

## Observability Hooks
- Use `--json` to emit structured CLI output suitable for piping to jq.
- Enable verbose logs:
  ```bash
  RUST_LOG=marketplace=debug cargo run --bin marketplace -- list
  ```
- Metrics snapshot:
  ```bash
  curl http://127.0.0.1:8910/metrics
  ```

## Troubleshooting
- **Credential errors**: Confirm `git credential` helpers are configured; the extension never stores secrets.
- **Rate limit wait**: CLI surfaces countdown; rerun `cache refresh` once countdown reaches zero.
- **Offline usage**: Listings fall back to cached manifests; warnings indicate stale data.
