---

description: "Task list template for feature implementation"
---

# Tasks: Gemini CLI Extension Marketplace

**Input**: Design documents from `/specs/001-build-a-gemini/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Principle V requires unit, integration, and snapshot coverage for every impacted area. Only omit a test task when the spec explicitly documents why no automated check is needed.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

**After completing Phase N tasks**: Execute the Delivery Checkpoints from plan.md (formatting, linting, tests) before committing and opening a PR.

## Task Format: `[ID] [P?] [Story] Description`

- **[P]**: Indicates tasks that can run in parallel (different files, no dependencies).
- **[Story]**: Refers to the user story this task belongs to (e.g., US1, US2, US3).
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- **Web app**: `backend/src/`, `frontend/src/`
- **Mobile**: `api/src/`, `ios/src/` or `android/src/`
- Paths shown below assume single project - adjust based on plan.md structure



## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure.

- [x] T001 Verify Rust toolchain 1.82.0 and workspace dependencies in `rust-toolchain.toml` / `Cargo.toml`
- [x] T002 Add default marketplace source seed data for `https://github.com/athola/gemini-marketplace` in `src/marketplace/config.rs`
- [x] T003 Ensure configuration directory structure exists under `$GEMINI_CONFIG/extensions/marketplace/` via helper in `src/marketplace/cache/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that must be complete before any user story implementation can begin.

**Note**: No user story work can begin until this phase is complete.

- [x] T004 Implement preferences serialization/deserialization in `src/marketplace/services/preferences.rs`
- [x] T005 Implement cache store foundation (read/write with TTL metadata) in `src/marketplace/cache/store.rs`
- [x] T006 Implement background refresh job queue skeleton in `src/marketplace/services/refresh.rs`
- [x] T007 Integrate tracing & metrics initialization hooks in `src/bin/marketplace.rs`
- [x] T008 [P] Create integration harness bootstrap utilities in `tests/integration/bootstrap.rs`

**Checkpoint**: Foundation ready; user story implementation can now begin in parallel.

---

## Phase 3: User Story 1 - Discover and Browse Extensions (Priority: P1) 🎯 MVP

**Goal**: Users can list available Gemini CLI extensions from the default and configured sources with paginated CLI output and offline-friendly caching.

**Independent Test**: Run `gemini marketplace list` with and without network connectivity, ensuring paginated results display within 2 seconds when cached and warnings surface when data is stale.

### Tests for User Story 1 (Mandatory, unless an exception is documented in the spec)

*Write these tests first, ensuring they fail before implementation.*

- [ ] T009 [P] [US1] Add snapshot coverage for `gemini marketplace --help` and paginated list output in `tests/marketplace_help.rs`
- [ ] T010 [P] [US1] Add integration test for cached list rendering and stale warning in `tests/integration/list_cli.rs`
- [ ] T049 [P] [US1] Add integration test verifying install status detection (registry first, filesystem fallback) in `tests/integration/list_cli.rs`

### Implementation for User Story 1

- [ ] T011 [US1] Implement CLI command wiring for `list` and `search` in `src/marketplace/commands/list.rs` and `src/marketplace/commands/search.rs`
- [ ] T012 [US1] Implement catalog pagination and namespacing logic in `src/marketplace/services/catalog.rs`
- [ ] T013 [US1] Implement lazy loading batch fetch in `src/marketplace/services/source_fetcher.rs`
- [ ] T014 [US1] Implement list API route in `src/marketplace/api/extensions.rs` per `/extensions` contract
- [ ] T015 [US1] Implement offline cache fallback with stale detection in `src/marketplace/cache/mod.rs`
- [ ] T016 [US1] Wire telemetry counters for list/search usage in `src/marketplace/services/catalog.rs`
- [ ] T050 [US1] Implement install status detection and display by querying the extension registry then filesystem in `src/marketplace/services/catalog.rs` and `src/marketplace/commands/list.rs`

**Checkpoint**: User Story 1 should now be fully functional and independently testable.

---

## Phase 4: User Story 2 - View Extension Details with Progressive Validation (Priority: P1)

**Goal**: Users can view detailed metadata for a specific extension, with schema validation on fetch and semantic validation when details are requested.

**Independent Test**: Run `gemini marketplace show source/name` to confirm details include validation state, error diagnostics for invalid manifests, and that execution stops when manifest verification fails.

### Tests for User Story 2 (Mandatory, unless an exception is documented in the spec)

- [ ] T017 [P] [US2] Add unit tests for manifest schema validation in `src/marketplace/models/manifest.rs`
- [ ] T018 [P] [US2] Add integration test for detail view semantic validation in `tests/integration/list_extensions.rs`
- [ ] T051 [P] [US2] Add unit tests for manifest checksum and semantic version verification in `src/marketplace/models/manifest.rs`

### Implementation for User Story 2

- [ ] T019 [US2] Implement manifest schema parser using `schemars` in `src/marketplace/models/manifest.rs`
- [ ] T020 [US2] Implement detail CLI command in `src/marketplace/commands/show.rs`
- [ ] T021 [US2] Implement `/extensions/{namespace}` API route in `src/marketplace/api/extensions.rs`
- [ ] T022 [US2] Implement semantic validation workflow and warning generation in `src/marketplace/services/catalog.rs`
- [ ] T023 [US2] Surface fatal errors for invalid manifests in CLI output in `src/marketplace/commands/list.rs`
- [ ] T052 [US2] Implement manifest checksum and semantic version verification prior to caching in `src/marketplace/services/catalog.rs`

**Checkpoint**: Detail views and validations now operate end-to-end.

---

## Phase 5: User Story 3 - Manage Marketplace Sources (Priority: P2)

**Goal**: Users can add, list, and remove marketplace sources, with directory recursion controls and credential helper integration.

**Independent Test**: Use `gemini marketplace sources add/remove/list` to manage sources, ensuring monorepo recursion limit is honored and sources persist in preferences.

### Tests for User Story 3 (Mandatory, unless an exception is documented in the spec)

- [ ] T024 [P] [US3] Add integration test covering add/list/remove flows in `tests/integration/list_extensions.rs`
- [ ] T025 [P] [US3] Add unit tests for preferences persistence in `tests/unit/preferences.rs`
- [ ] T053 [P] [US3] Add integration test confirming credential helper usage without storing secrets in `tests/integration/source_fetcher.rs`

### Implementation for User Story 3

- [ ] T026 [US3] Implement source management CLI commands in `src/marketplace/commands/sources.rs`
- [ ] T027 [US3] Implement `/sources` GET/POST/DELETE routes in `src/marketplace/api/sources.rs`
- [ ] T028 [US3] Implement recursive discovery with configurable depth in `src/marketplace/services/source_fetcher.rs`
- [ ] T029 [US3] Persist source configuration updates in `src/marketplace/services/preferences.rs`
- [ ] T054 [US3] Ensure source fetching relies on Git credential helpers without persisting credentials in `src/marketplace/services/source_fetcher.rs`

**Checkpoint**: Source catalog management is fully functional and persists across runs.

---

## Phase 6: User Story 4 - Cache Refresh & TTL Controls (Priority: P2)

**Goal**: Users can configure cache TTLs, trigger refresh queues, and observe rate-limit aware behavior.

**Independent Test**: Adjust TTL via CLI, force a refresh, and verify background jobs queue with rate-limit handling while cached data remains available.

### Tests for User Story 4 (Mandatory, unless an exception is documented in the spec)

- [ ] T030 [P] [US4] Add integration test for TTL update and refresh queue in `tests/integration/cache_store.rs`
- [ ] T031 [P] [US4] Add unit tests for refresh job scheduling in `tests/unit/preferences.rs`
- [ ] T055 [P] [US4] Add integration test verifying rate-limit countdown output in `tests/integration/list_cli.rs`

### Implementation for User Story 4

- [ ] T032 [US4] Implement `cache refresh` and `cache ttl set` CLI commands in `src/marketplace/commands/refresh.rs` and `src/marketplace/commands/cache.rs`
- [ ] T034 [US4] Implement TTL update persistence and validation in `src/marketplace/services/preferences.rs`
- [ ] T035 [US4] Implement rate-limit aware deferred retries in `src/marketplace/services/refresh.rs`
- [ ] T056 [US4] Implement rate-limit countdown messaging for CLI and telemetry in `src/marketplace/services/refresh.rs` and `src/marketplace/commands/list.rs`

**Checkpoint**: Cache management respects TTLs and rate-limit policies.

---

## Phase 7: User Story 5 - Observability & Status Reporting (Priority: P3)

**Goal**: Provide dual-mode logging, structured metrics, and status inspection so operators can monitor marketplace health.

**Independent Test**: Enable verbose logging and retrieve status to confirm metrics reflect cache hits/misses, queue depth, and rate-limit timers.

### Tests for User Story 5 (Mandatory, unless an exception is documented in the spec)

- [ ] T037 [P] [US5] Add unit tests for metrics emission in `tests/unit/domain.rs`
- [ ] T059 [P] [US5] Add unit tests confirming telemetry defaults to opt-out in `tests/unit/preferences.rs`

### Implementation for User Story 5

- [ ] T039 [US5] Implement structured logging toggle (human vs JSON) in `src/bin/marketplace.rs`
- [ ] T040 [US5] Emit metrics counters in `src/marketplace/services/catalog.rs` and `src/marketplace/services/refresh.rs`
- [ ] T060 [US5] Default telemetry collection to opt-out and wire configuration flag handling in `src/marketplace/services/preferences.rs` and `src/marketplace/config.rs`
- [ ] T042 [US5] Document observability usage in `docs/` or `quickstart.md`

**Checkpoint**: Observability and status reporting align with NFR-001.

---

## Phase N: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T043 [P] Update README.md with CLI usage examples and cache management guidance
- [ ] T044 [P] Review and update `specs/001-build-a-gemini/quickstart.md` with any new flags or workflows
- [ ] T045 Ensure `insta` snapshots are reviewed and accepted in `tests/integration/__snapshots__/`
- [ ] T046 Conduct `cargo fmt`, `cargo clippy --all-targets --all-features -D warnings`, and `cargo test` runs; capture results for PR evidence
- [ ] T047 Finalize documentation of metrics and telemetry opt-out controls in `docs/observability.md` (create if missing)
- [ ] T048 Prepare changelog or release notes referencing constitution principles and observed metrics

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1)**: Depends on US1 catalog structures for detail lookup
- **User Story 3 (P2)**: Depends on US1 cache/catalog foundations to persist sources
- **User Story 4 (P2)**: Depends on US1 cache infrastructure and US3 preferences persistence
- **User Story 5 (P3)**: Depends on all prior stories to expose full observability metrics

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, US1 work may proceed while documentation tasks from prior phases continue
- Tasks within a user story marked [P] can run concurrently, provided they touch separate modules (e.g., list snapshot test vs API route implementation)

---

## Parallel Example: User Story 1

```bash
# Terminal 1: write failing tests
cargo test tests::integration::list_cli::list_displays_cached_results -- --nocapture

# Terminal 2: implement catalog service
$EDITOR src/marketplace/services/catalog.rs

# Terminal 3: update CLI command
$EDITOR src/marketplace/commands/list.rs
```

When tests fail as expected, implement the service and commands, then re-run `cargo test` until green.

---

## Implementation Strategy

- Deliver MVP by completing Setup, Foundational, and User Story 1. This enables browsing cached/online catalogs with paginated CLI output.
- Tackle User Story 2 once MVP is stable to unlock detail inspection and validation workflows.
- Execute User Stories 3–5 based on risk/priority: sources management (US3), cache/refresh controls (US4), and observability/status (US5) can proceed in parallel when dependencies are met.
- Reserve Polish tasks for post-implementation hardening, ensuring Delivery Checkpoints (fmt, clippy, test) are satisfied before commit/PR handoff.
