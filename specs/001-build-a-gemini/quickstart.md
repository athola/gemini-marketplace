# Quickstart – Gemini CLI Extension Marketplace

## Prerequisites
- Rust 1.82.0 toolchain installed (the repo's `rust-toolchain.toml` pins 1.82.0; run `rustup toolchain install 1.82.0` once and `rustup show active-toolchain` inside the repo should report `1.82.0` before hacking).
- Gemini CLI available in PATH with extensions enabled.
- GitHub access token (if hitting private repos) configured via credential helpers.

## Environment Setup

Use the following `make` targets to keep formatting, linting, and testing consistent across the workspace.

```bash
make fmt          # cargo fmt --all
make lint         # cargo clippy --workspace --all-targets --all-features -D warnings
make test         # cargo test --workspace
```

Run `~/.codex/commands/session-auto-skill` at the start of a session to auto-load `token-conservation` and `cpu-gpu-performance`.

Capture a CPU/GPU baseline (`uptime`, `ps -Ao pcpu,comm | head`, `nvidia-smi dmon`) before long test runs, per Constitution Principle VI.

## Workspace Targets

All commands run from the workspace root.

*   `cargo build -p marketplace-core`: Builds the shared library and legacy CLI entrypoints (`gemini-marketplace` binary).
*   `cargo build -p marketplace-mcp-server`: Compiles the MCP stdio server binary that Gemini launches during extension installs/runtime.
*   `cargo build -p marketplace-mcp-cli`: Builds the developer-only MCP client harness. Use this binary for local testing; Gemini never invokes it.
*   `cargo run -p marketplace-mcp-cli -- list`: Automatically spawns the MCP server (if not already running), issues the same MCP request Gemini would send, and prints the response for quick validation.

### OpenAPI Contract

The CLI mirrors the HTTP API described in `specs/001-build-a-gemini/contracts/marketplace-openapi.yaml`. Lint the contract with:

```bash
make contract-lint
```

`make contract-lint` uses `scripts/lint-openapi.sh` (Redocly CLI via `npx`).

## Common Workflows

### 1. List extensions

```bash
gemini marketplace list
gemini marketplace list --interactive
```

### 2. Search/filter

```bash
gemini marketplace search caching
gemini marketplace search --category observability
```

### 3. View details

```bash
gemini marketplace show curated/cache-inspector
```

### 4. Manage sources

```bash
gemini marketplace sources add https://github.com/org/extensions
# The CLI prompts for an alias, which defaults to a sanitized slug.
gemini marketplace sources list
gemini marketplace sources remove org-extensions
```

### 5. Cache control

```bash
gemini marketplace cache refresh --sources org-extensions curated
gemini marketplace cache ttl set 24
```

### 6. MCP server (Gemini runtime)

Gemini launches the MCP server via stdio. Developers can mimic this manually:

```bash
cargo run -p marketplace-mcp-server -- --stdio
```

### 7. Local MCP harness

Auto-spawn the server and issue the same MCP calls Gemini sends:

```bash
cargo run -p marketplace-mcp-cli -- list --json
cargo run -p marketplace-mcp-cli -- search caching
```

## Testing & Verification

*   Write failing tests (unit/integration/snapshot) that capture new CLI surfaces.
*   For selective reruns, prefer `cargo test cli::list_command::tests::shows_pagination` and escalate to `cargo test` only before a pull request.
*   Record token usage and CPU/GPU minutes in the pull request description as evidence for Principle VI.
*   Run `make contract-lint` when touching REST/CLI contracts to ensure the OpenAPI file stays valid.

## Observability Hooks

*   Export human-readable logs with `RUST_LOG=info gemini marketplace list`.
*   Enable JSON telemetry via `GEMINI_MARKETPLACE_LOG=json` for automated ingestion. The same environment variable applies to the MCP server (`cargo run -p marketplace-mcp-server`), so Gemini and the test CLI share identical log formats.
*   Metrics include cache hits/misses, rate-limit wait durations, and aggregated search keywords. The MCP test CLI forwards server responses but does not implement its own logging pipeline.

## Marketplace Filesystem Layout

Runtime state is stored in `$GEMINI_CONFIG/extensions/marketplace/` (or `GEMINI_MARKETPLACE_HOME` when set) with these directories:

*   `cache/`: Normalized manifests, batches, and TTL metadata.
*   `config/`: Preferences (`preferences.json`), sources registry (`sources.json`), and the refresh queue.
*   `logs/`: JSON/human-readable telemetry exports.

The cache initializer (`crate::marketplace::cache::init::ensure_layout`) creates this layout before commands run so developers can rely on consistent paths when adding new features or tests.

### Permissions & Local Publish

`make local-publish` copies the extension into `$GEMINI_CONFIG/extensions/{gemini-marketplace,marketplace}`. Ensure the target path is writable (either run `chown` once or set `GEMINI_CONFIG` to a directory you own, e.g. `export GEMINI_CONFIG=$HOME/.gemini-dev`). See README's Environment Variables section for additional overrides.
