# Tasks: Gemini CLI Extension Marketplace

**Input**: Design documents from `/specs/001-build-a-gemini/`  
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md, contracts/

**Tests**: Constitution mandates test-first delivery. Every phase introduces failing tests before implementation tasks.

**Organization**: Tasks are grouped by user story so each slice ships independently with dual-format CLI output, offline support, telemetry, and observability hooks.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths for every task
- Ensure each story covers dual-format UX, offline/cache behaviour, telemetry, and observability expectations

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare fixtures, helpers, and dependencies required by all user stories.

- [X] T001 Create curated and custom marketplace fixtures under `tests/data/marketplace/` (manifests, README excerpts, cache snapshots) for integration coverage.
- [X] T002 Add an Axum/reqwest test server helper with rate-limit toggles in `tests/common/http.rs` and expose it via `tests/common/mod.rs`.
- [X] T003 Declare marketplace crate dependencies and MSRV metadata in `Cargo.toml` and ensure `rust-toolchain.toml` pins Rust 1.82.0.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Establish configuration, caching, background refresh, metrics registry, and default curated source bootstrap before any story work.

- [ ] T004 Add unit tests for config directory resolution and curated source bootstrapping in `tests/unit/config_bootstrap.rs`.
- [ ] T005 Implement config resolver and default curated source seeding (athola repo enabled by default) in `src/marketplace/config.rs` and hook initialization in `src/bin/marketplace.rs`.
- [ ] T006 Add unit tests for marketplace preferences read/write in `tests/unit/preferences_store.rs`.
- [ ] T007 Implement preferences store persistence in `src/marketplace/services/preferences.rs`, storing JSON under `$GEMINI_CONFIG/extensions/marketplace/config/preferences.json`.
- [ ] T008 Add unit tests for cache store TTL and 500-extension batching in `tests/unit/cache_store.rs`.
- [ ] T009 Implement cache store with TTL metadata and lazy batch writes in `src/marketplace/cache/store.rs`.
- [ ] T010 Add unit tests for refresh queue rate-limit countdown behaviour in `tests/unit/refresh_queue.rs`.
- [ ] T011 Implement refresh queue worker with deferred rate-limit retries in `src/marketplace/services/refresh.rs`.
- [ ] T012 Add unit tests for marketplace metrics registry (cache hits, rate-limit waits, top search term tracking) in `tests/unit/metrics_registry.rs`.
- [ ] T013 Implement metrics registry capturing counters and rotating top search terms in `src/marketplace/services/metrics.rs` and expose hooks through `src/marketplace/mod.rs`.
- [ ] T014 Add integration tests proving curated source bootstrap populates listings offline in `tests/integration/marketplace_bootstrap.rs`.
- [ ] T015 Wire curated source bootstrap, metrics registry initialization, and offline cache readiness into CLI startup in `src/bin/marketplace.rs`.

---

## Phase 3: User Story 1 – Browse Available Extensions (Priority: P1) 🎯 MVP

**Goal**: Deliver `gemini marketplace list` with pagination, namespaced identifiers, installed status, offline cache support, and interactive navigation.

**Independent Test**: Running `gemini marketplace list` (online, offline, `--json`, `--interactive`) returns consistent paginated results with namespaced IDs, curated source entries, install badges, cache freshness, rate-limit countdown metadata, and updates metrics counters.

### Tests (write first; ensure they fail)

- [ ] T016 [US1] Add integration tests for list pagination, interactive mode, namespacing, offline cache, metrics increments, and `--json` parity in `tests/integration/marketplace_list.rs`.
- [ ] T017 [P] [US1] Add unit tests for `CatalogService::list` covering install detection, namespaced IDs, cache metadata, and sorting in `tests/unit/catalog_list.rs`.

### Implementation

- [ ] T018 [US1] Implement install detection service (registry first, with a secondary filesystem scan) in `src/marketplace/services/install.rs` with trait injection for testing.
- [ ] T019 [US1] Extend list DTOs and catalogue aggregation to emit paginated, namespaced results with cache freshness + warnings in `src/marketplace/services/catalog.rs`.
- [ ] T020 [US1] Rework CLI list command for table/JSON parity and interactive navigation in `src/marketplace/commands/list.rs` and `src/bin/marketplace.rs`.
- [ ] T021 [US1] Implement `/marketplace/extensions` list endpoint with pagination/search params in `src/marketplace/api/extensions.rs` and register route in `src/marketplace/api/server.rs`.

**Checkpoint**: User Story 1 complete — list command/API validated online/offline with tests passing.

---

## Phase 4: User Story 2 – View Extension Details (Priority: P2)

**Goal**: Provide `gemini marketplace show <source/extension>` with README excerpt, semantic validation, install guidance, and dual-format output.

**Independent Test**: `gemini marketplace show curated/example --json` surfaces detail data, README excerpt, validation messages, and install instructions within 30 seconds, online or cached.

### Tests (write first; ensure they fail)

- [ ] T022 [US2] Add integration tests for detail rendering, README excerpts, validation messaging, and cache reload behaviour in `tests/integration/marketplace_show.rs`.
- [ ] T023 [P] [US2] Add unit tests for `CatalogService::detail` semantic validation flows in `tests/unit/catalog_detail.rs`.

### Implementation

- [ ] T024 [US2] Implement semantic validation helpers (schema vs semantic reports) in `src/marketplace/services/validation.rs`.
- [ ] T025 [US2] Enhance catalog detail pipeline to reload stale cache, run validation, and enrich install info in `src/marketplace/services/catalog.rs`.
- [ ] T026 [US2] Replace CLI show command with table/JSON detail output including warnings in `src/marketplace/commands/show.rs` and route wiring in `src/bin/marketplace.rs`.
- [ ] T027 [US2] Complete `/marketplace/extensions/{id}` API detail handler with 404/422 semantics in `src/marketplace/api/extensions.rs`.

**Checkpoint**: User Story 2 complete — detail flows validated with semantic errors surfaced.

---

## Phase 5: User Story 3 – Search and Filter Extensions (Priority: P3)

**Goal**: Support keyword and category filtering through `gemini marketplace search`, honoring local filter vs pre-fetch mode, tracking network savings, and capturing telemetry for top search terms.

**Independent Test**: `gemini marketplace search observability --category analytics --prefetch` reduces remote requests, reports filtered totals, updates top-search telemetry, and maintains dual-format parity.

### Tests (write first; ensure they fail)

- [ ] T028 [US3] Add integration tests for search keyword/category filters, pre-fetch savings reporting, telemetry capture, and JSON parity in `tests/integration/marketplace_search.rs`.
- [ ] T029 [P] [US3] Add unit tests for search mode branching, metrics counters, and top-search term rotation in `tests/unit/catalog_search.rs`.

### Implementation

- [ ] T030 [US3] Enhance `src/marketplace/services/source_fetcher.rs` to apply local vs pre-fetch filters, track network savings, and publish metrics hooks.
- [ ] T031 [US3] Update CLI search command and shared list options in `src/marketplace/commands/search.rs` to surface filter summaries, telemetry stats, and metrics output.
- [ ] T032 [US3] Extend `/marketplace/extensions` API handling to honor search/category params and return filter totals + telemetry snapshots in `src/marketplace/api/extensions.rs`.
- [ ] T033 [US3] Persist top search term telemetry for the current release cycle in `src/marketplace/services/metrics.rs` and expose retrieval via CLI/API.

**Checkpoint**: User Story 3 complete — targeted discovery flows validated with filter telemetry.

---

## Phase 6: User Story 4 – Manage Marketplace Sources (Priority: P4)

**Goal**: Enable adding/removing/listing sources, recursive manifest discovery with depth controls, cache management commands, and credential-helper-aligned auth.

**Independent Test**: Adding a private source via `sources add`, listing shows sync status with namespace, removing cleans up cache, `cache refresh` queues jobs with countdown, and `cache ttl set` persists preferences without storing credentials.

### Tests (write first; ensure they fail)

- [ ] T034 [US4] Add integration tests for `sources add/list/remove` including monorepo recursion/namespace handling in `tests/integration/marketplace_sources.rs`.
- [ ] T035 [P] [US4] Add integration tests for `cache refresh` and `cache ttl set` queue/TTL persistence in `tests/integration/marketplace_cache.rs`.

### Implementation

- [ ] T036 [US4] Extend `src/marketplace/services/sources.rs` to persist sources, guard default curated source, rely on git credential helpers (no secret storage), and expose recursion depth + auth flags.
- [ ] T037 [US4] Implement recursive manifest discovery with depth limit and skipped-manifest warnings in `src/marketplace/services/source_fetcher.rs`.
- [ ] T038 [US4] Update CLI sources commands for add/list/remove with namespace display in `src/marketplace/commands/sources.rs` and hook options in `src/bin/marketplace.rs`.
- [ ] T039 [US4] Implement cache management CLI commands (`cache refresh`, `cache ttl set`) with countdown output in `src/marketplace/commands/cache.rs`.
- [ ] T040 [US4] Fulfill `/marketplace/sources` and cache routes in `src/marketplace/api/sources.rs` and `src/marketplace/api/status.rs`.

**Checkpoint**: User Story 4 complete — source lifecycle and cache controls independently verified.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Finalize observability, documentation, and release readiness once all user stories pass.

- [ ] T041 Add structured logging + metrics counters (cache hits, rate-limit waits, top search terms, skipped manifests) across `src/marketplace/commands/list.rs`, `src/marketplace/commands/search.rs`, `src/marketplace/services/catalog.rs`, and `src/marketplace/services/refresh.rs`.
- [ ] T042 Refresh documentation with final command examples, telemetry notes, and troubleshooting guidance in `README.md` and `specs/001-build-a-gemini/quickstart.md`.
- [ ] T043 Run release validation (`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`) and capture logs in `specs/001-build-a-gemini/reports/release-validation.md`.

---

## Dependencies & Execution Order

- **Setup (Phase 1)** → **Foundational (Phase 2)** → **US1 → US2 → US3 → US4** → **Polish**
- User stories are independently shippable once their checkpoints pass.
- Foundational work (config, cache, refresh, metrics, curated source bootstrap) must complete before US1 starts.
- Polish tasks run only after all targeted stories finish.

### Story Dependency Graph

```
Setup → Foundational → US1 → US2 → US3 → US4 → Polish
```

---

## Parallel Execution Examples

- Setup: T001 and T002 touch different directories; T003 can follow once fixtures land.
- US1: T016 and T017 can proceed in parallel before implementation; implementation tasks split by CLI vs API.
- US2: T022 and T023 target different test scopes and run concurrently.
- US3: T028 and T029 can be split between contributors while US1/US2 code stabilizes; telemetry persistence (T033) runs after metrics registry (T013).
- US4: T034 and T035 exercise independent CLI flows; T036 waits on those tests.
- Polish: T042 documentation can proceed while T041 instrumentation completes; T043 waits for all prior tasks.

---

## Implementation Strategy

1. **MVP First**: Complete Setup → Foundational → US1 to deliver browsing with offline support, curated default source, and baseline telemetry.
2. **Incremental Delivery**: Layer US2 (details), US3 (search + top-term telemetry), then US4 (source management) once shared services harden.
3. **Observability Last**: Apply instrumentation and documentation updates during Polish after behaviours stabilize.
4. **Test-First Discipline**: Maintain failing tests before each implementation task to satisfy constitutional gates and keep the branch demo-ready.
