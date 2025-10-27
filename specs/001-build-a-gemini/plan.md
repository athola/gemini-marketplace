# Implementation Plan: Gemini CLI Extension Marketplace

**Branch**: `001-build-a-gemini` | **Date**: 2025-10-26 | **Spec**: `/specs/001-build-a-gemini/spec.md`
**Input**: Feature specification from `/specs/001-build-a-gemini/spec.md`

**Note**: This plan aligns with the Gemini Marketplace constitution and guides `/speckit.tasks`.

## Summary

Build a Rust-based Gemini CLI extension that exposes `gemini marketplace` commands for browsing,
inspecting, searching, and managing Gemini CLI extensions. The marketplace aggregates multiple
sources, namespaces extensions, caches manifests under `$GEMINI_CONFIG/extensions/marketplace/`, and
supports dual-format (table/JSON) output. It must remain usable offline through cached data with
background retries, surface rate-limit countdowns, and validate manifests progressively (schema on
fetch, semantic on detail).

## Technical Context

**Language/Version**: Rust 1.82.0 (MSRV enforced by constitution)  
**Primary Dependencies**: `reqwest`, `tokio`, `serde`, `serde_json`, `schemars`, `directories`,
`thiserror`, `anyhow`, `indicatif`, `semver`, `clap`, `axum`, `tracing`, `sha2`  
**Storage**: Local filesystem cache beneath `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata  
**Testing**: `cargo test` leveraging `assert_cmd`, `insta`, `predicates`, `humantime`, and Axum fixtures  
**Target Platform**: Cross-platform Gemini CLI environments (macOS, Linux, Windows terminals)  
**Project Type**: Single Rust crate with CLI entrypoint and modular marketplace services  
**Performance Goals**: Cached listings render within 2 seconds; remote catalogs fetched lazily in
500-extension batches; interactive navigation remains responsive; detail view completes within 30 s  
**Constraints**: Offline-first behaviour, dual-format CLI/API parity, structured logging + metrics,
credential helper reliance (no secret storage), configurable cache TTL and recursion depth, telemetry
tracking of top search terms for SC-005  
**Scale/Scope**: Thousands of extensions across multiple sources; telemetry-driven keyword list per
release; organizations adding private sources alongside curated catalog

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Description | Status | Notes |
|-----------|-------------|--------|-------|
| I. Test-First CLI Delivery | Tests are written before implementation; red-green-refactor enforced | PASS | Plan sequences failing tests before code per story; fixtures ready in Setup. |
| II. Consistent Dual-Format UX | CLI commands expose human-readable + `--json` parity | PASS | Each story’s CLI work includes table + JSON parity and mirrored API endpoints. |
| III. Offline-First Resilience | Use of cached data, deterministic TTL, and background retries are in place | PASS | Foundational phase establishes cache store, preferences, and refresh queue before stories. |
| IV. Observability & Diagnostics | Structured logs, metrics, and countdown UX planned | PASS | Polish tasks add tracing metrics; earlier stories emit countdown + warnings. |
| V. Dependency & Version Stewardship | Rust MSRV locked, new crates justified, no credential storage | PASS | Dependencies limited to approved stack; curated source bootstrap enforces helper usage. |

Additional constraints satisfied: cross-platform CLI ensured via pure stdout/stderr rendering; cache
location fixed to `$GEMINI_CONFIG/extensions/marketplace/`; authentication flows rely solely on OS
credential helpers; telemetry for top search terms captured via preferences/metrics for SC-005.

## Project Structure

### Documentation (this feature)

```text
specs/001-build-a-gemini/
├── plan.md              # This file (/speckit.plan output)
├── research.md          # Phase 0 research decisions
├── data-model.md        # Phase 1 entity definitions
├── quickstart.md        # Phase 1 developer workflow
├── contracts/           # Phase 1 API contracts (OpenAPI)
└── tasks.md             # Phase 2 breakdown (/speckit.tasks output)
```

### Source Code (repository root)

```text
src/
├── bin/
│   └── marketplace.rs
├── lib.rs
└── marketplace/
    ├── api/
    ├── cache/
    ├── commands/
    ├── config.rs
    ├── error.rs
    ├── mod.rs
    ├── models/
    └── services/

tests/
├── common/
├── integration/
├── unit/
└── marketplace_help.rs

tests/data/marketplace/
└── curated/ + custom fixtures (manifests, README excerpts, cache snapshots)
```

**Structure Decision**: Single Rust crate with marketplace modules grouped by concern. Shared test
fixtures live under `tests/data/marketplace/`; helpers reside in `tests/common/`.

## Complexity Tracking

Not required (no constitution violations to justify).

## Implementation Strategy

1. **MVP First**: Complete Setup → Foundational → US1 to deliver browsing with offline support, curated default source, and baseline metrics/telemetry.
2. **Incremental Delivery**: Layer US2 (details), US3 (search + top-term telemetry), then US4 (source management) once shared services harden.
3. **Observability Last**: Apply instrumentation and documentation updates during Polish after behaviours stabilize.
4. **Test-First Discipline**: Maintain failing tests before each implementation task to satisfy constitutional gates and keep the branch demo-ready.

## Commit Checkpoints (required before pushing PRs)

1. **Fixtures & Toolchain** — After T001–T003 pass (`cargo test --tests tests::common`), commit before touching feature logic.
2. **Foundation & Metrics Bootstrap** — After T004–T015 green (config/cache/refresh/metrics registry), commit to isolate foundational changes.
3. **US1 – List MVP** — After T016–T021 tests/demos succeed online/offline, commit the first demoable slice.
4. **US2 – Detail View** — After T022–T027 pass, commit the detail functionality with validation output.
5. **US3 – Search & Telemetry** — After T028–T033 pass, commit search enhancements and telemetry persistence.
6. **US4 – Source & Cache Management** — After T034–T040 pass, commit lifecycle/cache commands.
7. **Polish & Release Readiness** — After T041–T043 (fmt/clippy/test + docs) succeed, commit final polish prior to PR.

Each checkpoint must end with passing tests and a runnable CLI/API demo to prevent oversized commits during `/speckit.implement`.
