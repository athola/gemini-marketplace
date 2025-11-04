# Quickstart: Gemini Marketplace CLI Extension

## Prerequisites

- Gemini CLI installed and authenticated.
- Rust 1.82.0 toolchain (for building from source) or access to prebuilt binary.
- Network access to GitHub or chosen marketplace sources (unless operating on cached data).

## Install the Marketplace Extension

```bash
git clone https://github.com/gemini-rs/marketplace.git
cd marketplace
cargo install --path . --force
gemini extensions link "$(pwd)"
gemini extensions list
```

Validate installation:

```bash
gemini marketplace --help
```

All runtime state lives under `$GEMINI_CONFIG/extensions/marketplace/` (defaults to `~/.gemini/extensions/marketplace/` when the environment variable is unset).

Inside the Gemini chat UI you can invoke the same functionality with the slash command:

```bash
/marketplace list --json
/marketplace cache refresh --force
```

For day-to-day development, the repo ships with a `Makefile`:

```bash
make help          # show available shortcuts
make fmt            # cargo fmt
make lint           # cargo clippy --all-targets --all-features -- -D warnings
make test           # cargo test
make local-publish  # rebuild and sync into ~/.gemini/extensions/gemini-marketplace
```

Use `make local-publish` to simulate a catalog release locally before pushing an official extension update.

## Seed Default Source

The extension ships with a curated source hosted at `https://github.com/athola/gemini-marketplace`. Confirm it is enabled:

```bash
gemini marketplace sources list
```

If disabled, re-enable it:

```bash
gemini marketplace sources add https://github.com/athola/gemini-marketplace
```

## Browse Extensions

List extensions (uses cached data when available):

```bash
gemini marketplace list
```

Navigate longer lists interactively:

```bash
gemini marketplace list --interactive
# then type: next / prev / quit
```

Inspect an extension’s details:

```bash
gemini marketplace show source-name/extension-name
```

## Search the Catalog

```bash
gemini marketplace search "analytics"
```

Apply filters:

```bash
gemini marketplace search --category data "analytics"
```

## Manage Sources

Add a custom source (git repository or local path):

```bash
gemini marketplace sources add https://github.com/example/private-marketplace
```

Remove a source:

```bash
gemini marketplace sources remove example
```

## Control Cache Freshness

Set the default TTL (hours):

```bash
gemini marketplace cache ttl set 12
```

Force a refresh immediately:

```bash
gemini marketplace cache refresh
```

Check marketplace status and diagnostics:

```bash
gemini marketplace status
gemini marketplace status --json
```

> Tip: When calling the extension via `gemini marketplace …`, ensure the
> CLI is launched with the `run_shell_command` tool enabled, e.g.:
> `gemini --allowed-tools run_shell_command marketplace status -- --json`.

## Troubleshooting & Status

Check background refresh and rate-limit state:

```bash
gemini marketplace status
```

Enable verbose logging for deeper diagnostics:

```bash
gemini marketplace list --verbose
```

When network issues occur, the CLI continues serving cached data and queues refresh jobs. Use `status` to monitor progress.

## Demo Workflow

This workflow demonstrates the extension's capabilities in an isolated environment.

1. **Bootstrap an isolated demo environment**

    ```bash
    export GEMINI_MARKETPLACE_HOME="$(mktemp -d)"
    export GEMINI_MARKETPLACE_LOG=info
    export GEMINI_MARKETPLACE_LOG_FORMAT=text
```

2. **Show current configuration**

    ```bash
    ls "$GEMINI_MARKETPLACE_HOME"
    cat "$GEMINI_MARKETPLACE_HOME/config/preferences.json"
    ```

3. **Demonstrate basic CLI commands**

    ```bash
    gemini marketplace --help
    gemini marketplace list --json
    gemini marketplace cache refresh
    ```

4. **Inspect the refresh queue**

    ```bash
    cat "$GEMINI_MARKETPLACE_HOME/config/refresh_queue.json"
    ```

5. **Adjust the cache TTL**

    ```bash
    gemini marketplace cache ttl set 12
    cat "$GEMINI_MARKETPLACE_HOME/config/preferences.json" | jq '.cache_ttl_hours'
    ```

6. **Review logs**

    ```bash
    cat "$GEMINI_MARKETPLACE_HOME/logs/marketplace.log" 2>/dev/null || echo "logs emitted to stderr"
    ```

7. **Tear down the demo environment**

    ```bash
    rm -rf "$GEMINI_MARKETPLACE_HOME"
    unset GEMINI_MARKETPLACE_HOME GEMINI_MARKETPLACE_LOG GEMINI_MARKETPLACE_LOG_FORMAT
    ```
