# Gemini Marketplace

A CLI extension for discovering and managing Gemini extensions.

Gemini Marketplace is a command-line tool that makes it easy to find, install, and manage extensions for the Gemini CLI. It provides a centralized place to discover new functionality and customize your Gemini experience.

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
   cargo install --path . --force
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

All marketplace data now lives under `$GEMINI_CONFIG/extensions/marketplace/` (defaults to `~/.gemini/extensions/marketplace/` when the environment variable is not set), keeping it alongside the rest of your Gemini CLI state.

### Build from Source

If you prefer a local build without installing the binary globally, you can still compile the project directly:

```bash
git clone https://github.com/gemini-rs/marketplace.git
cd marketplace
cargo build
```

## Usage

Once installed, you can use the `marketplace` command to manage your extensions.

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

Inspect marketplace health and configuration:

```bash
gemini marketplace status
# or
gemini marketplace status --json
```

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

A few common workflows are wired up in the `Makefile`:

```bash
make help          # show available shortcuts
make fmt            # cargo fmt
make lint           # cargo clippy --all-targets --all-features -- -D warnings
make test           # cargo test
make local-publish  # rebuild binary and copy the extension into ~/.gemini/extensions/gemini-marketplace
```

`make local-publish` is useful for testing a release locally without involving the public catalog. It recreates the installed extension directory (respecting `$GEMINI_CONFIG` if set) and drops a `.gemini-extension-install.json` metadata file so `gemini extensions list` reflects the update.

## Contributing

We welcome contributions to the Gemini Marketplace! If you'd like to contribute, please read our [contributing guidelines](https://github.com/gemini-rs/marketplace/blob/main/CONTRIBUTING.md).

## License

This project is licensed under the [MIT License](https://github.com/gemini-rs/marketplace/blob/main/LICENSE).
