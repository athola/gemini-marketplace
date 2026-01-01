# Changelog

All notable changes to this project will be documented in this file. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- Converted the repo into a Cargo workspace (`marketplace-core`, `marketplace-mcp-server`, `marketplace-mcp-cli`).
- Documented SemVer policy, OpenAPI contract, and environment variables in README/Quickstart.
- Added demo targets (`make demo`, `make demo-mcp`) plus OpenAPI linting (`make contract-lint`).

### Fixed
- Telemetry warnings and redundant closures flagged by `cargo clippy`.
- Duplicate getrandom dependencies: Switched from rustls-tls to native-tls in reqwest, eliminating the getrandom version conflict between ring (v0.2.16) and tempfile (v0.3.4).
