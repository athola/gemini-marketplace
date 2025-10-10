# Quickstart — Gemini CLI Marketplace Extension

## Prerequisites
- Rust toolchain `1.82.0` installed via `rustup` (`rustup default 1.82.0`)
- Gemini CLI installed and configured (ensure `gemini --version` succeeds)
- Access to Git credential helper or environment tokens for private sources
- GitHub personal access token (PAT) stored in your credential helper for higher rate limits (optional but recommended)

## Project Setup
1. Clone the repository and checkout the feature branch:
   ```bash
   git checkout 001-build-a-gemini
   ```
2. Verify toolchain and install components:
   ```bash
   rustup override set 1.82.0
   rustup component add clippy rustfmt
   cargo fetch
   ```
3. Configure the Gemini CLI extension path (once):
   ```bash
   gemini extensions path add ./target/debug/gemini-marketplace
   ```

## Building
```bash
cargo build
```
- Debug binary output: `target/debug/gemini-marketplace`
- Release build for distribution:
  ```bash
  cargo build --release
  ```

## Running the Extension
- List marketplace extensions:
  ```bash
  gemini marketplace list --search "<keyword>"
  ```
- View details:
  ```bash
  gemini marketplace show source-slug/extension-slug
  ```
- Manage sources:
  ```bash
  gemini marketplace sources add https://github.com/example/extensions
  gemini marketplace sources ls
  gemini marketplace sources rm example
  ```

## Testing
1. Run fast unit tests:
   ```bash
   cargo test
   ```
2. Execute integration suite (uses axum-based test servers to simulate GitHub):
   ```bash
   cargo test --test integration
   ```
3. Lint + fmt:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   ```

## Debugging Rate Limits
- Check status:
  ```bash
  gemini marketplace status
  ```
- Force refresh after countdown completes:
  ```bash
  gemini marketplace refresh --force
  ```

## Windows Notes
- Run commands from PowerShell; ensure `~\AppData\Local\gemini\config` is writable.
- Confirm Git credential manager is installed (`git credential-manager-core`).

## Next Steps
- Review `research.md` and `data-model.md` for architectural context.
- Align implementation tasks via `/speckit.tasks` once Phase 2 is initiated.
