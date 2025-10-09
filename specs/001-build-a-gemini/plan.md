# Implementation Plan: Gemini CLI Extension Marketplace

**Branch**: `001-build-a-gemini` | **Date**: 2025-10-09 | **Spec**: specs/001-build-a-gemini/spec.md
**Input**: Feature specification from `specs/001-build-a-gemini/spec.md`

## Summary

Build a Rust-based Gemini CLI extension that aggregates third-party extension catalogs, parses `gemini-extension.json` manifests, and delivers searchable, cached marketplace discovery with namespaced identifiers, rate-limit aware refresh, and private-source support via existing Git credentials.

## Technical Context

<!--
  ACTION REQUIRED: Replace the content in this section with the technical details
  for the project. The structure here is presented in advisory capacity to guide
  the iteration process.
-->

**Language/Version**: Rust 1.82.0 (MSRV aligned with Gemini CLI extension tooling)  
**Primary Dependencies**: reqwest + tokio (HTTP), serde/serde_json (manifests), directories (config paths), thiserror/anyhow (errors), indicatif (UX); dev: assert_cmd, insta, wiremock  
**Storage**: Per-source JSON cache under `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata and manual refresh  
**Testing**: `cargo test` unit coverage plus assert_cmd CLI contracts, wiremock-backed integration tests with insta snapshots  
**Target Platform**: Linux and macOS official; Windows 11+ best-effort via directories crate and documented caveats  
**Project Type**: Rust CLI extension crate integrated with Gemini CLI  
**Performance Goals**: List rendering ≤2s from cache; remote fetch optimized via search-before-fetch when enabled  
**Constraints**: Must queue refresh under GitHub rate limits; must avoid storing credentials; offline cache availability  
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
<!--
  ACTION REQUIRED: Replace the placeholder tree below with the concrete layout
  for this feature. Delete unused options and expand the chosen structure with
  real paths (e.g., apps/admin, packages/something). The delivered plan must
  not include Option labels.
-->

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

*Fill ONLY if Constitution Check has violations that must be justified*

No constitution violations require justification at this time.
