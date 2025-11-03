# Implementation Plan: Gemini CLI Extension Marketplace

**Branch**: `001-build-a-gemini` | **Date**: 2025-10-30 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-build-a-gemini/spec.md`

## Summary

This plan details the implementation of a `gemini marketplace` command. This command will enable discovery, inspection, and management of Gemini CLI extensions from curated or user-defined sources. Key features include progressive manifest validation with checksum enforcement, namespaced identifiers, offline caching with configurable TTLs, observability, and background refresh.

## Technical Context

**Language/Version**: Rust 1.82.0 (pinned via `rust-toolchain.toml`)  
**Primary Dependencies**: `tokio`, `reqwest` (rustls), `serde`/`serde_json`, `schemars`, `directories`, `clap`, `semver`, `thiserror`, `tracing`, optional `indicatif` for progress UI  
**Storage**: Filesystem cache beneath `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata and user preferences  
**Testing**: `cargo test`, `cargo clippy --all-targets --all-features -D warnings`, integration coverage via `assert_cmd`, `insta` snapshots, targeted unit tests per module  
**Target Platform**: Cross-platform Gemini CLI environments (Linux, macOS, Windows) running the extension in terminal contexts  
**Project Type**: Single Rust CLI crate with reusable library modules in `src/marketplace`  
**Performance Goals**: Cached list rendering ≤2s; remote catalog fetch in 500-item batches with perceived response ≤5s per batch; detail view metadata parsing ≤30s worst case  
**Constraints**: Must support offline browsing, avoid storing credentials, respect rate limits with background retries, keep async code non-blocking, and expose default warnings without crashing commands  
**Security**: All functionality runs in-process with the CLI; no daemon or separate HTTP surface is exposed.  
**Validation**: Manifest processing must perform schema + semantic checks and verify `sha2` checksums and `semver` metadata before caching results  
**Scale/Scope**: Designed for thousands of extensions across multiple sources, each namespaced; supports user-managed source catalog plus default curated repo; concurrent refresh queue limited to maintainable size

## Constitution Check

*Initial review before Phase 0 research. Re-evaluate after Phase 1 design.*

- [x] CLI contract changes identified, help text updates planned, and automation impact noted — `gemini marketplace` subcommands documented and slated for snapshot tests.
- [x] Marketplace data validation strategy defined, including schema/hash handling — progressive manifest validation with schema + semantic phases and checksum/version enforcement.
- [x] Cache TTL behavior and refresh triggers documented for this feature — 24h default TTL, user-configurable overrides, background refresh + manual `cache refresh`.
- [x] Observability plan covers tracing spans and error propagation — INFO spans around fetch/refresh/install checks, structured metrics for cache hits, rate-limit waits, and trace IDs in verbose mode.
- [x] Test strategy lists the unit, integration, and snapshot coverage to be added — unit coverage per service, `assert_cmd` CLI flows, `insta` snapshots for help/list output, cache expiry integration tests.

*After design review (2025-10-30): All checks remain satisfied after data model, contracts, and quickstart deliverables.*

## Project Structure

### Documentation (this feature)

```text
specs/001-build-a-gemini/
├── plan.md              # This file (/speckit.plan output)
├── research.md          # Phase 0 research synthesis
├── data-model.md        # Phase 1 entity and validation design
├── quickstart.md        # Phase 1 runbook for extension usage
└── tasks.md             # Phase 2 task breakdown (generated later)
```

### Source Code (repository root)

```text
src/
├── bin/
│   └── marketplace.rs        # CLI entrypoint (clap definitions)
└── marketplace/
    ├── cache/
    │   ├── mod.rs
    │   └── store.rs
    ├── commands/
    │   ├── cache.rs
    │   ├── list.rs
    │   ├── refresh.rs
    │   ├── search.rs
    │   ├── show.rs
    │   └── sources.rs
    ├── models/
    │   ├── domain.rs
    │   ├── manifest.rs
    │   └── mod.rs
    ├── services/
    │   ├── catalog.rs
    │   ├── preferences.rs
    │   ├── refresh.rs
    │   ├── source_fetcher.rs
    │   └── sources.rs
    └── config.rs

tests/
├── integration/
│   ├── cache_store.rs
│   ├── list_cli.rs
│   ├── list_extensions.rs
│   ├── source_fetcher.rs
│   └── bootstrap.rs
├── unit/
│   ├── config.rs
│   ├── default_source.rs
│   ├── domain.rs
│   ├── error.rs
│   └── preferences.rs
└── marketplace_help.rs
```

**Structure Decision**: Retain single-crate organization with library modules under `src/marketplace` and the clap-based binary in `src/bin/marketplace.rs`. Feature work concentrates on services, commands, cache, and models, with mirrored integration tests under `tests/integration` and snapshot harnesses in `tests/marketplace_help.rs`.

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|

No constitution violations identified.

## Delivery Checkpoints

- **Pre-PR handoff**: Complete all tasks in the forthcoming `/speckit.tasks` checklist. Then, re-run `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, and `cargo test` to ensure compliance.
- Review updated CLI snapshots (`insta` output) and docs, verify constitution gates remain satisfied, then create a final commit encapsulating the marketplace feature changes.
- Push the branch and open a PR only once the above validation evidence is captured in the PR description (including references to CLI contract updates and observability instrumentation per Principles I & IV).
