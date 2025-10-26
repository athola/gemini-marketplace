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
*   **Extensible:** Add your own custom extension sources.

## Getting Started

### Installation

To install the Gemini Marketplace extension, you'll need to have the Gemini CLI installed. Then, you can install the marketplace extension from the official Gemini Marketplace catalog:

```bash
gemini marketplace install marketplace
```

### Build from Source

Alternatively, you can build the marketplace extension from source:

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

To install an extension:

```bash
gemini marketplace install <name>
```

## Contributing

We welcome contributions to the Gemini Marketplace! If you'd like to contribute, please read our [contributing guidelines](https://github.com/gemini-rs/marketplace/blob/main/CONTRIBUTING.md).

## License

This project is licensed under the [MIT License](https://github.com/gemini-rs/marketplace/blob/main/LICENSE).