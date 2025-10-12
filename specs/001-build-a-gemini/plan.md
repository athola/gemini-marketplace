# Implementation Plan: Gemini CLI Extension Marketplace

**Branch**: `001-build-a-gemini` | **Date**: 2025-10-09 | **Spec**: specs/001-build-a-gemini/spec.md
**Input**: Feature specification from `specs/001-build-a-gemini/spec.md`

## Summary

Build a Rust-based Gemini CLI extension that reads third-party catalogs from GitHub, parses `gemini-extension.json` manifests, and presents searchable, cached marketplace listings. Retain namespacing, rate-limit awareness, and private-source support to allow Gemini CLI users to jump between corporate and public sources during a single session.

## Technical Context

**Language/Version**: Rust 1.82.0 (MSRV aligned with Gemini CLI extension tooling; newer ICU crates require ≥1.82)  
**Primary Dependencies**: reqwest + tokio (HTTP), serde/serde_json (manifests), directories (config paths), thiserror/anyhow (errors), indicatif (UX); dev stack: assert_cmd, insta, predicates, humantime, axum (for in-process test servers). These libraries match the Gemini CLI ecosystem, letting us bundle the extension without extra native deps.  
**Storage**: Per-source JSON cache under `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata and manual refresh. Avoid SQLite to keep the cache inspectable and version-control friendly.  
**Testing**: `cargo test` unit coverage plus assert_cmd CLI contracts and axum-backed integration tests. This keeps the test loop fast while invoking the CLI in a similar manner to Gemini.
**Target Platform**: Linux and macOS official; Windows 11+ best-effort via directories crate and documented caveats  
**Project Type**: Rust CLI extension crate integrated with Gemini CLI  
**Performance Goals**: List rendering ≤2s from cache; remote fetch optimized via search-before-fetch when enabled. Maintain these thresholds such that CLI users don’t wait longer than Gemini’s typical command latency.  
**Constraints**: Must queue refresh under GitHub rate limits; must avoid storing credentials; offline cache availability. These guardrails come from Gemini CLI trust requirements and from the constitution’s “no credential storage” stance.  
**Scale/Scope**: Initial release covers default curated source + user-added sources (dozens of extensions, low concurrency)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- Gate 1 (Governance Clarity): Gemini Marketplace Constitution v1.0.0 defines five core principles (Test-First CLI Delivery, Consistent Dual-Format UX, Offline-First Resilience, Observability & Diagnostics, Dependency Stewardship). Status: PASS.
- Gate 2 (Non-Negotiable Principles): Test-first delivery is explicitly marked non-negotiable and aligned with plan/testing strategy. Status: PASS.
- Gate 3 (Workflow Compliance): Plan documents testing strategy, data model, and contracts consistent with constitutional workflow. Status: PASS.

*Post-Phase 1 Update*: Gates remain PASS with constitution ratified on 2025-10-09.

## Project Structure

### Documentation (this feature)

```
specs/001-build-a-gemini/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```
src/
├── bin/
│   └── marketplace.rs
├── marketplace/
│   ├── commands/
│   ├── services/
│   ├── models/
│   └── cache/

tests/
├── integration/
└── unit/
```

**Structure Decision**: Single Rust crate aligned with Cargo conventions; binary entrypoint exposed via `src/bin/marketplace.rs`, internal modules organized under `src/marketplace/`, and tests split between `tests/unit` and `tests/integration`.

## Complexity Tracking

No constitution violations require justification at this time.

## Implementation Strategy

### MVP First (User Story 1 Only)
1. Complete Phases 1–2 to establish the crate, infrastructure, and services.
2. Deliver Phase 3 (US1) to provide browsing capability — this is the minimum viable marketplace.
3. Validate via T024–T030 and ensure cached listings function before proceeding. Commit and push a PR covering setup + US1.

### Incremental Delivery
1. Ship US1 (P1) for initial marketplace visibility.
2. Layer on US2 (P2) to add detail views without disrupting listing functionality.
3. Introduce US3 (P3) to improve discoverability through search/filtering.
4. Finish with US4 (P4) for source management, observability, preferences, and refresh/status tooling, committing after each story.
5. Run the Gemini CLI integration phase to exercise `gemini marketplace` commands end-to-end and push a pre-release validation PR.
6. Apply final polish tasks before requesting review or release (final release PR).

### Parallel Team Strategy
1. Team collaborates on Phases 1–2.
2. Assign US1 to Developer A to secure MVP while Developer B prepares US2 service extensions once T026 is merged.
3. Developer C can begin US4 observability groundwork (T041, T051) after foundational tasks using mocks.
4. Upon completion of US4, dedicate bandwidth to Phase 7 integration checks before moving to polish.
5. Use the parallel opportunities list in `tasks.md` to avoid file conflicts and maintain independent delivery per story.
