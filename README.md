# Gemini Marketplace

A CLI extension for discovering and managing Gemini extensions.

The Gemini Marketplace is a command-line tool to find, install, and manage extensions for the Gemini CLI. It provides a centralized place to discover new functionality and customize your Gemini experience.

[![CI](https://github.com/gemini-rs/marketplace/actions/workflows/ci.yml/badge.svg)](https://github.com/gemini-rs/marketplace/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/gemini-marketplace.svg)](https://crates.io/crates/gemini-marketplace)
[![docs.rs](https://docs.rs/gemini-marketplace/badge.svg)](https://docs.rs/gemini-marketplace)

## Features

*   **Discover:** Find new extensions in the official Gemini Marketplace catalog.
*   **Search:** Search for extensions by name, keyword, or author.
*   **Install:** Install extensions from the marketplace with a single command.
*   **Manage:** View and manage your installed extensions.

## Getting Started

### Installation

1. **Install the binary**

   ```bash
   cargo install --path crates/marketplace-core --force
   ```

   This puts the `gemini-marketplace` executable in `~/.cargo/bin`. Make sure that directory is on your `PATH` so the Gemini CLI can invoke the binary.

2. **Install the extension into Gemini**

   - Once the extension is published to the public catalog:

     ```bash
     gemini extensions install marketplace
     ```

   - Until then, you can install directly from this repository:

     ```bash
     gemini extensions install https://github.com/gemini-rs/marketplace.git
     ```

   - For local development you can link the working tree:

     ```bash
     gemini extensions link /absolute/path/to/gemini-marketplace
     ```

   Validate the installation:

   ```bash
   gemini extensions list
   ```

   Remove the extension with `gemini extensions uninstall gemini-marketplace` when needed.

All marketplace data is stored in `$GEMINI_CONFIG/extensions/marketplace/` (defaults to `~/.gemini/extensions/marketplace/` when the environment variable is not set), keeping it alongside the rest of your Gemini CLI state.

### Build from Source

To build from source without a global install:

```bash
git clone https://github.com/gemini-rs/marketplace.git
cd marketplace
   cargo build -p marketplace-core
```

## Usage

Once installed, use the `marketplace` command to manage your extensions.

```bash
gemini marketplace --help
```

To list all available extensions:

```bash
gemini marketplace list
```

To search for an extension:

```bash
gemini marketplace search <query>
```

To manage sources:

```bash
gemini marketplace sources add https://github.com/example/repo
gemini marketplace sources list
```

To refresh the cache manually:

```bash
gemini marketplace cache refresh
```

To inspect marketplace health and configuration:

```bash
gemini marketplace status
# or
gemini marketplace status --json
```

### Supported subcommands

| Command | Purpose |
| --- | --- |
| `gemini marketplace list [--json] [--interactive]` | Show paginated catalog results with cache freshness and warnings. |
| `gemini marketplace show <source/extension>` | Display detailed manifest metadata, README excerpts, and validation diagnostics. |
| `gemini marketplace search [--category <tag>] <keyword?>` | Filter catalog locally or via pre-filtered fetches. |
| `gemini marketplace sources add/list/remove …` | Manage curated and custom sources, including recursion depth and alias prompts. |
| `gemini marketplace cache refresh [--force]` | Trigger background refresh queues and report rate-limit warnings. |
| `gemini marketplace cache ttl set <hours>` | Adjust cache TTL (default 24h). |
| `gemini marketplace status [--json]` | View cache/refresh/telemetry health. |

### Contracts & API Reference

The CLI mirrors the internal HTTP API defined in [`specs/001-build-a-gemini/contracts/marketplace-openapi.yaml`](specs/001-build-a-gemini/contracts/marketplace-openapi.yaml). To lint the OpenAPI contract locally, install [Redocly CLI](https://redocly.com/docs/cli/) (or any `npx`-resolvable implementation) and run:

```bash
make contract-lint
```

CI should run the same command to gate contract regressions.

### Environment Variables

| Variable | Description | Default/Stability |
| --- | --- | --- |
| `GEMINI_MARKETPLACE_HOME` | Overrides the runtime config/cache root (mirrors `$GEMINI_CONFIG/extensions/marketplace`). Useful for demos/tests. | Unset ⇒ `$GEMINI_CONFIG/extensions/marketplace` or `~/.gemini/extensions/marketplace`. Stable. |
| `GEMINI_MARKETPLACE_SOURCE_URL` | Points the catalog to a custom `index.json`. Handy for testing private sources or demo servers. | Unset ⇒ curated Athola catalog. Experimental but widely used in tests. |
| `GEMINI_MARKETPLACE_LOG` | Controls log format (`text` or `json`). Applies to both CLI and MCP server. | `text`. Stable. |
| `MARKETPLACE_CLI_BIN` | Path override the MCP server uses to find the `gemini-marketplace` binary. | Auto-detected (sibling binary / `$PATH`). Advanced/unstable. |
| `MARKETPLACE_MCP_SERVER_BIN` | Path override the developer harness (`marketplace-mcp-cli`) uses to spawn the MCP server. | Auto-detected. Advanced/unstable. |

### Versioning & Changelog

The crate follows Semantic Versioning. Backwards-compatible additions bump MINOR, breaking changes bump MAJOR, and critical fixes bump PATCH. See [CHANGELOG.md](CHANGELOG.md) for release notes; every PR touching a public surface should update the changelog entry for the upcoming release.
| `gemini marketplace status [--json]` | Surface cache health, refresh queue depth, and a telemetry summary. |

The same contract applies when Gemini launches the MCP server binary. The developer-only `marketplace-mcp-cli` harness issues these subcommands over MCP for local testing without Gemini.

> Running `gemini marketplace …` through the Gemini CLI requires the
> `run_shell_command` tool. If you see an error like “Tool
> `run_shell_command` not found”, relaunch Gemini with the flag (for
> example: `gemini --allowed-tools run_shell_command marketplace status -- --json`).

### Slash commands inside the Gemini CLI

Installing the extension registers a `/marketplace` slash command that proxies to the Rust binary:

```bash
/marketplace                     # show CLI help
/marketplace list --json
/marketplace search analytics
/marketplace cache refresh --force
```

Everything after `/marketplace` is passed verbatim to `gemini-marketplace`, so the slash command behaves exactly like the standalone CLI.

### Makefile shortcuts

A few common workflows are available in the `Makefile`:

```bash
make help          # show available shortcuts
make fmt           # cargo fmt --all
make lint          # cargo clippy --workspace --all-targets --all-features -D warnings
make test          # cargo test --workspace
make local-publish  # rebuild binary and copy the extension into ~/.gemini/extensions/gemini-marketplace
make extension-archive  # build dist/gemini-marketplace-extension.tar.gz for remote installs
```

`make local-publish` is useful for testing a release locally without involving the public catalog. It rebuilds the binary, reinstalls it, and recreates the extension directories.

For remote distribution, run `make extension-archive`. This creates `dist/gemini-marketplace-extension.tar.gz`, which contains the manifest and command definitions.

## Contributing

We welcome contributions to the Gemini Marketplace! If you'd like to contribute, please read our [contributing guidelines](https://github.com/gemini-rs/marketplace/blob/main/CONTRIBUTING.md).

## Speckit + Superpowers Workflow (AI agents)

Agents working inside `/home/alext/gemini-marketplace` must follow this project-scoped ritual when running any `/speckit:*` command:

1. **When you need Speckit, start with `/prompts:specify-startup`.** This prompt runs `specify --help` (the uv-installed CLI from [github/spec-kit](https://github.com/github/spec-kit)) so Speckit hooks load for the current shell, then loads `speckit-meta` to attach superpowers skills. Skip it during generic repo onboarding to keep the context window lean.
2. **Run the Speckit command normally.** `/speckit.clarify`, `/speckit.plan`, `/speckit.tasks`, `/speckit.implement`, `/speckit.analyze`, and `/speckit.checklist` all assume `speckit-meta` is active; each prompt now reminds you to verify this before continuing.
3. **Log dual-governance variances.** If Speckit’s constitution checks and the superpowers skills disagree, record a variance note (conflict, owner, and resolution timeline) instead of halting. Work continues, but you must resolve or explicitly waive each variance before final verification.
4. **Verify before completion.** After implementing changes, run `make test --quiet` (or relevant Make targets) and finish with `/speckit.checklist` to confirm both Speckit and superpowers are satisfied.

These instructions are scoped to this repository so other projects can maintain their own workflows.

### Catchup Prompt Workflow

- Run `/prompts:catchup` whenever you need a branch summary. The prompt immediately tells you to execute `~/.codex/superpowers/.codex/superpowers-codex use-skill catchup`.
- The `catchup` skill enforces four TodoWrite items (repo confirmed, git status captured, diffs summarized, follow-ups recorded) so every review captures consistent evidence while keeping tokens low.
- The workflow relies on lightweight git commands (`git status -sb`, `git diff --stat origin/main...HEAD`, `git diff --name-only`) and asks you to open only meaningful files. Escalate deeper investigations to `superpowers:systematic-debugging` after the catchup pass.
- If the skill cannot load (e.g., sandboxed shell), the prompt documents the fallback commands so `/prompts:catchup` still produces a useful summary.

## License

This project is licensed under the [MIT License](https://github.com/gemini-rs/marketplace/blob/main/LICENSE).
