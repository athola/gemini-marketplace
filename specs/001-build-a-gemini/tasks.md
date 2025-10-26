# Tasks: Gemini CLI Extension Marketplace

**Input**: Design documents from `/specs/001-build-a-gemini/`  
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Constitution requires test-first delivery. Each story phase begins with failing tests before implementation.

**Organization**: Tasks are grouped by user story so each story can ship and be tested independently.

## Format: `[ID] [P?] [Story] Description`
- **[P]** marks tasks that can run in parallel (different files, no direct dependencies).
- **[Story]** labels the owning story (e.g., US1, US2). Setup/Foundational/Polish use descriptive labels.
- Always include concrete file paths so the task is directly executable.

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Prepare shared fixtures and helpers needed by all stories.

- [ ] T001 [Setup] Create reusable catalog and manifest fixtures under `tests/data/marketplace/` (curated + custom sources with README excerpts) to drive integration scenarios.
- [ ] T002 [Setup] Add an Axum-based fixture server helper with rate-limit toggles in `tests/common/http.rs` and expose it via `tests/common/mod.rs` for use in story tests.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core services that every user story depends on; must be complete before story work starts.

- [ ] T003 [Foundation] Implement persistent user preferences loading/saving (`config/preferences.json`) in `src/marketplace/services/preferences.rs` with coverage in `tests/unit/preferences_store.rs`.
- [ ] T004 [Foundation] Build refresh job queue + background worker persisting state under the cache directory in `src/marketplace/services/refresh.rs` (support queued retry + rate-limit countdowns) with tests in `tests/unit/refresh_queue.rs`.
- [ ] T005 [Foundation] Refactor `src/marketplace/services/source_fetcher.rs` and `src/marketplace/cache/store.rs` to fetch/save 500-extension batches with TTL metadata; add tests in `tests/unit/source_fetcher_batches.rs`.

**Checkpoint**: Foundation ready — verification that preferences, refresh queue, and batch caching all pass before entering User Story 1.

---

## Phase 3: User Story 1 – Browse Available Extensions (Priority: P1) 🎯 MVP

**Goal**: Allow users to run `gemini marketplace list` to browse extensions with pagination, installed status, warnings, and offline cache fallback.

**Independent Test**: Running `gemini marketplace list` online, then offline, shows paginated results with install indicators, cache warnings, and identical JSON output via `--json`.

### Tests (write first; ensure they fail)

- [ ] T006 [US1] Add integration coverage for list pagination, offline fallback, and JSON parity in `tests/integration/marketplace_list.rs` using the fixture server.
- [ ] T007 [P] [US1] Add unit tests for `CatalogService::list` covering installed detection, warnings aggregation, and sort order in `tests/unit/catalog_list.rs`.

### Implementation

- [ ] T008 [US1] Introduce an install detection service that checks the registry then falls back to filesystem scans in `src/marketplace/services/install.rs`, and integrate it into `src/marketplace/services/catalog.rs`.
- [ ] T009 [US1] Extend `ListRequest`/`ListResponse` in `src/marketplace/services/catalog.rs` (and supporting structs in `src/marketplace/models/domain.rs`) to include pagination, totals, cache freshness, and rate-limit metadata.
- [ ] T010 [US1] Rework `src/marketplace/commands/list.rs` and wire options in `src/bin/marketplace.rs` to render paginated tables with next/prev prompts, installed badges, category chips, and aligned `--json` output.
- [ ] T011 [US1] Implement the real `/marketplace/extensions` handler in `src/marketplace/api/extensions.rs` (map query params, return contract-compliant JSON) and update `src/marketplace/api/server.rs` to route through the stateful handler.

**Checkpoint**: User Story 1 complete — CLI list + API endpoint validated online/offline with recorded fixtures.

---

## Phase 4: User Story 2 – View Extension Details (Priority: P2)

**Goal**: Provide `gemini marketplace show <source/extension>` with full manifest details, README excerpt, and semantic validation results.

**Independent Test**: `gemini marketplace show curated/example` returns detailed output including manifest path, validation errors, and install instructions in both table and JSON formats.

### Tests (write first; ensure they fail)

- [ ] T012 [US2] Add integration tests in `tests/integration/marketplace_show.rs` verifying detail rendering, README excerpts, and validation error surfacing.
- [ ] T013 [P] [US2] Add unit tests for `CatalogService::detail` semantic validation flows in `tests/unit/catalog_detail.rs`.

### Implementation

- [ ] T014 [US2] Implement `CatalogService::detail` returning a rich detail DTO, reloading cache when stale, and enriching with validation + install info in `src/marketplace/services/catalog.rs`.
- [ ] T015 [US2] Add a semantic validation engine that performs deep manifest checks and README extraction in `src/marketplace/services/source_fetcher.rs` (new helper module `src/marketplace/services/validation.rs`).
- [ ] T016 [US2] Replace the stubbed CLI command in `src/marketplace/commands/show.rs` (and hook via `src/bin/marketplace.rs`) to present detailed tables/JSON plus clear validation warnings.
- [ ] T017 [US2] Complete the `/marketplace/extensions/:id` API route in `src/marketplace/api/extensions.rs` to return contract-compliant detail payloads and appropriate 404/422 errors.

**Checkpoint**: User Story 2 complete — detail views tested independently, ready for demo.

---

## Phase 5: User Story 3 – Search and Filter Extensions (Priority: P3)

**Goal**: Support keyword and category filtering through `gemini marketplace search`, honoring local-filter vs pre-fetch modes.

**Independent Test**: Running `gemini marketplace search observability --category analytics` filters the list appropriately and, when preferences request pre-fetch filtering, limits remote requests while reporting filtered totals.

### Tests (write first; ensure they fail)

- [ ] T018 [US3] Add integration test for search command filter behavior and telemetry when toggling pre-fetch mode in `tests/integration/marketplace_search.rs`.
- [ ] T019 [P] [US3] Add unit tests validating search-mode branching between local filter and pre-fetch in `tests/unit/catalog_search.rs`.

### Implementation

- [ ] T020 [US3] Enhance `src/marketplace/services/source_fetcher.rs` to accept optional search/category filters, short-circuit remote manifest fetches when in pre-fetch mode, and report network savings metrics.
- [ ] T021 [US3] Update `src/marketplace/commands/search.rs` (and shared list options) to set the pre-fetch flag, surface filter summaries, and reuse pagination helpers.
- [ ] T022 [US3] Extend `/marketplace/extensions` handling in `src/marketplace/api/extensions.rs` to honor search/category parameters and return filtered counts alongside totals.

**Checkpoint**: User Story 3 complete — targeted discovery flows validated.

---

## Phase 6: User Story 4 – Manage Marketplace Sources (Priority: P4)

**Goal**: Enable adding/removing/listing sources, recursive manifest discovery, and cache management commands (`sources`, `cache refresh`, `cache ttl set`).

**Independent Test**: Adding a custom source via `sources add`, listing shows synced status with namespacing, removing cleans up, and cache commands queue refresh + update TTL settings.

### Tests (write first; ensure they fail)

- [ ] T023 [US4] Add integration tests for `sources add/list/remove` including monorepo recursion in `tests/integration/marketplace_sources.rs`.
- [ ] T024 [P] [US4] Add integration tests for `cache refresh` and `cache ttl set` behavior (queue state, TTL persistence) in `tests/integration/marketplace_cache.rs`.

### Implementation

- [ ] T025 [US4] Extend `src/marketplace/services/sources.rs` to persist recursion depth, enabled flags, requires-auth metadata, and prevent conflicting slugs.
- [ ] T026 [US4] Implement recursive manifest discovery with a configurable depth limit and skipped-manifest warnings in `src/marketplace/services/source_fetcher.rs`.
- [ ] T027 [US4] Update CLI source commands in `src/marketplace/commands/sources.rs` and `src/bin/marketplace.rs` to accept display-name, recursion-depth, requires-auth flags, and render sync state.
- [ ] T028 [US4] Implement cache management logic in `src/marketplace/commands/cache.rs`, wiring into `src/marketplace/services/refresh.rs` and preferences for TTL persistence with JSON output parity.
- [ ] T029 [US4] Fulfill `/marketplace/sources`, `/marketplace/sources/add`, `/marketplace/sources/:name`, and `/marketplace/cache/refresh` routes in `src/marketplace/api/sources.rs` and `src/marketplace/api/status.rs`, returning contract-compliant responses.

**Checkpoint**: User Story 4 complete — source lifecycle and cache controls independently verified.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final observability, documentation, and release readiness after all user stories.

- [ ] T030 [Polish] Add structured logging + metrics counters (cache hits, rate-limit waits, skipped manifests) across commands/services in `src/marketplace/commands/list.rs`, `src/marketplace/services/catalog.rs`, and `src/marketplace/services/refresh.rs`.
- [ ] T031 [P] [Polish] Refresh documentation (`README.md`, `specs/001-build-a-gemini/quickstart.md`) with final command examples, environment variables, and troubleshooting notes.
- [ ] T032 [Polish] Run and capture release validation (`cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test --all-targets`) and attach logs for review.

---

## Dependencies & Execution Order

- **Setup → Foundational**: T001–T002 must finish before T003–T005 so shared fixtures/helpers exist.
- **Foundational → Stories**: T003–T005 unblock all story phases; no story work begins until they pass.
- **Story Priority**: Execute US1 → US2 → US3 → US4. Later stories depend on catalog/list infrastructure established earlier but remain independently testable.
- **Polish**: T030–T032 run only after all targeted user stories are complete.

### Story Dependency Graph

```
Setup → Foundational → US1 → US2 → US3 → US4 → Polish
                 └──────────────┬──────────────┘
                  (US2–US4 reuse catalog/cache infrastructure from US1)
```

---

## Parallel Execution Examples

- **Setup**: T001 (fixtures) and T002 (fixture server helper) can proceed concurrently once directories are created.
- **US1**: T006 (integration) and T007 (unit) are parallelizable before implementation begins.
- **US2**: T012 and T013 target different test suites and can run in parallel.
- **US3**: T018 and T019 can be split between contributors while foundational work finishes.
- **US4**: T023 (sources) and T024 (cache) cover distinct flows and can run concurrently.
- **Polish**: T031 (docs) can proceed while T030 finalizes instrumentation, with T032 waiting for all code tasks to finish.

---

## Implementation Strategy

### MVP First (User Story 1 Only)
1. Complete Phases 1–2 to solidify preferences, caching, and refresh infrastructure.
2. Deliver US1 to provide discoverability and offline support.
3. Validate MVP via CLI + API smoke tests before expanding scope.

### Incremental Delivery
1. Layer US2 for detailed inspection once listing works reliably.
2. Add US3 to improve findability via search and category filters.
3. Finish with US4 to empower custom sources and cache controls.

### Parallel Team Strategy
1. Collaborate on foundational tasks, ensuring shared interfaces (`CatalogService`, `SourceFetcher`, `RefreshService`) are stable.
2. Assign user stories to different contributors (e.g., Dev A on US1, Dev B on US2, Dev C on US3/US4) leveraging the fixture server and shared services.
3. Reconvene for polish tasks and final validation prior to release.

---

## Notes

- Keep tests failing until corresponding code tasks are complete to honor the constitution’s test-first mandate.
- Mark tasks `[P]` only when files do not overlap and there are no logical dependencies.
- Checkpoints after each story ensure incremental deliverables can be demoed or released independently.
