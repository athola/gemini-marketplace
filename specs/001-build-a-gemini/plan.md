# Implementation Plan: Gemini CLI Extension Marketplace

**Branch**: `001-build-a-gemini` | **Date**: 2025-10-12 | **Spec**: `/specs/001-build-a-gemini/spec.md`  
**Input**: Feature specification from `/specs/001-build-a-gemini/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Deliver a Rust 1.82 Gemini CLI extension that exposes a `gemini marketplace` command suite for discovering, inspecting, searching, and managing third-party Gemini CLI extensions. The extension must aggregate results from multiple sources, cache manifests under `$GEMINI_CONFIG/extensions/marketplace/`, surface dual-format (table/JSON) output, and remain usable offline through cached data with background retries and rate-limit countdowns. Progressive manifest validation (schema on fetch, semantic on detail) and pagination with optional interactive navigation (`--interactive`) are required.

## Technical Context

**Language/Version**: Rust 1.82.0 (MSRV locked by constitution)  
**Primary Dependencies**: `reqwest`, `tokio`, `serde`/`serde_json`, `directories`, `thiserror`, `anyhow`, `indicatif`, `semver`, `clap`, `axum`, `sha2`  
**Storage**: Local filesystem cache per source under `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata  
**Testing**: `cargo test` with `assert_cmd`, `insta`, `predicates`, `humantime`, `axum` fixtures  
**Target Platform**: Cross-platform Gemini CLI environments (macOS, Linux, Windows terminals)  
**Project Type**: Rust CLI extension packaged as a single crate  
**Performance Goals**: Cached listings render within 2 seconds; remote catalogs fetched in 500-extension batches with lazy loading  
**Constraints**: Offline-first browsing, dual-format output (`--json` parity), rate-limit countdown UX, no credential storage, progressive manifest validation  
**Scale/Scope**: Thousands of extensions across multiple sources, configurable recursion depth (default 5) for monorepos

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Description | Status | Notes |
|-----------|-------------|--------|-------|
| I. Test-First CLI Delivery | Failing tests precede implementation | PASS | Plan sequences test tasks before code for each story |
| II. Consistent Dual-Format UX | Table + JSON parity for commands | PASS | All CLI handlers will emit table + `--json` output |
| III. Offline-First Resilience | Cache fallback + background retries | PASS | Cache store + refresh queue prioritized in foundational phase |
| IV. Observability & Diagnostics | Structured logging, metrics, countdowns | PASS | Telemetry instrumentation scoped in polish tasks |
| V. Dependency & Version Stewardship | Rust 1.82 MSRV, limited new crates | PASS | No additional crates beyond approved stack anticipated |

Re-evaluated after Phase 1 design artifacts: all principles remain in PASS status.

## Project Structure

### Documentation (this feature)

```
specs/001-build-a-gemini/
├── plan.md              # Implementation plan (this file)
├── research.md          # Phase 0 research outcomes
├── data-model.md        # Phase 1 entities & relationships
├── quickstart.md        # Phase 1 developer quickstart
├── contracts/           # Phase 1 API contracts (OpenAPI)
└── tasks.md             # Phase 2 task breakdown (generated separately)
```

### Source Code (repository root)

```
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
├── marketplace_help.rs
└── unit/
```

**Structure Decision**: Single Rust crate with marketplace modules under `src/marketplace/` and coverage in `tests/{integration,unit}` aligning with existing CLI architecture.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| _None_ | — | — |
