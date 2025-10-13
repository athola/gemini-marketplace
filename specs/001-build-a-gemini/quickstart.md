# Quickstart — Gemini Marketplace Extension

## Prerequisites
- Rust 1.82.0 toolchain (`rustup show active-toolchain` should report 1.82).
- Gemini CLI installed and configured with `$GEMINI_CONFIG` directory writable.
- Network access to GitHub (optional when browsing cached data).

## Build & Test
```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## Configure Sources
```bash
# Add curated default (enabled by default)
gemini marketplace sources list

# Add an additional source
gemini marketplace sources add https://github.com/example/team-marketplace

# Remove a source
gemini marketplace sources remove team-marketplace
```

## Browse Extensions
```bash
# Paginated listing (table output)
gemini marketplace list

# Opt-in interactive pagination loop
gemini marketplace list --interactive

# Keyword search with category filter
gemini marketplace search observability --category analytics

# Show extension details (namespaced id)
gemini marketplace show curated/awesome-extension
```

## Cache Management
```bash
# Force refresh all sources (honors rate-limit queues)
gemini marketplace cache refresh --force

# Adjust TTL to 12 hours
gemini marketplace cache ttl set 12

# JSON output for scripting
gemini marketplace list --json
```

## Observability Tips
- Enable structured logging by setting `GEMINI_MARKETPLACE_LOG=json`.
- Inspect metrics counters (cache hits, rate-limit waits) via `gemini marketplace list --json`.
- When rate limited, the CLI displays a countdown sourced from the internal rate-limit window.
