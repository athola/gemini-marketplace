# Tasks: Gemini CLI Extension Marketplace

**Input**: Design documents from `specs/001-build-a-gemini/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Tests**: Integration tests are requested via the spec’s independent test criteria and plan’s testing strategy. Each user story phase includes targeted tests using `assert_cmd`, `wiremock`, and `insta`.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`
- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3). Setup/Foundational/Polish tasks use `[Setup]`, `[Foundation]`, or `[Polish]`.
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and baseline tooling

- [ ] T001 [Setup] Scaffold Rust crate with `Cargo.toml`, `src/lib.rs`, and `src/bin/marketplace.rs` aligned to the planned structure
- [ ] T002 [Setup] Declare required crates in `Cargo.toml` (`tokio`, `reqwest`, `serde`, `directories`, `thiserror`, `anyhow`, `indicatif`, dev `assert_cmd`, `insta`, `wiremock`)
- [ ] T003 [Setup] Add repository-wide tooling configs (`rust-toolchain.toml`, `.cargo/config.toml`, `clippy.toml`) enforcing Rust 1.82.0 and lint rules
- [ ] T004 [Setup] Create testing scaffolds (`tests/integration/`, `tests/unit/`, `tests/common/mod.rs`) with placeholder harness setup

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T005 [Foundation] Scaffold `src/marketplace/` module tree with stub files (`mod.rs`, `commands/mod.rs`, `services/mod.rs`, `models/{manifest.rs,domain.rs}`, `cache/mod.rs`, `api/mod.rs`)
- [ ] T006 [P] [Foundation] Implement configuration helpers in `src/marketplace/config.rs` to resolve cache and preferences directories using the `directories` crate
- [ ] T007 [P] [Foundation] Introduce shared error enums and result aliases in `src/marketplace/error.rs` leveraging `thiserror`
- [ ] T008 [Foundation] Parse `gemini-extension.json` manifests in `src/marketplace/models/manifest.rs`, including validation and warning aggregation
- [ ] T009 [Foundation] Model domain entities and install state in `src/marketplace/models/domain.rs` per `data-model.md`
- [ ] T010 [Foundation] Implement JSON cache store with TTL metadata in `src/marketplace/cache/store.rs`
- [ ] T011 [Foundation] Build source fetcher with GitHub rate-limit queue handling in `src/marketplace/services/source_fetcher.rs`
- [ ] T012 [Foundation] Set up Clap-driven CLI bootstrap and command routing skeleton in `src/bin/marketplace.rs`
- [ ] T013 [Foundation] Implement lightweight HTTP server harness in `src/marketplace/api/server.rs` ready to host OpenAPI routes

**Checkpoint**: Foundation ready — user story implementation can now begin

---

## Phase 3: User Story 1 - Browse Available Extensions (Priority: P1) 🎯 MVP

**Goal**: Allow users to list available extensions with key metadata, respecting cache and install status requirements.

**Independent Test**: Run `gemini marketplace list` with mocked sources to confirm the CLI displays names, descriptions, versions, install status, and source namespaces within 2 seconds using cached data.

### Tests for User Story 1

- [ ] T014 [US1] Author `tests/integration/list_extensions.rs` using `wiremock` + `assert_cmd` + `insta` to assert list output, caching, and warning behavior when manifests are skipped

### Implementation for User Story 1

- [ ] T015 [US1] Implement listing workflow in `src/marketplace/services/catalog.rs` to merge cache, fetch manifests, compute install status, and skip invalid manifests with warnings
- [ ] T016 [P] [US1] Render tabular and JSON outputs in `src/marketplace/commands/list.rs`, including namespace formatting and pagination support
- [ ] T017 [US1] Wire the `list` command and options into Clap routing within `src/bin/marketplace.rs`
- [ ] T018 [P] [US1] Expose `GET /marketplace/extensions` handler in `src/marketplace/api/extensions.rs` returning `ExtensionListResponse`

**Checkpoint**: User Story 1 independently testable (MVP)

---

## Phase 4: User Story 2 - View Extension Details (Priority: P2)

**Goal**: Provide detailed information for a selected extension, including docs, compatibility, and install guidance.

**Independent Test**: Execute `gemini marketplace show <source/extension>` to confirm extended metadata renders, README excerpts are available, and install instructions are surfaced.

### Tests for User Story 2

- [ ] T019 [US2] Create `tests/integration/view_extension_details.rs` covering detail retrieval, README excerpt rendering, and missing extension 404 handling

### Implementation for User Story 2

- [ ] T020 [US2] Extend `src/marketplace/services/catalog.rs` with detail retrieval, README extraction, and manifest checksum exposure
- [ ] T021 [P] [US2] Implement `show` command presenter in `src/marketplace/commands/show.rs` with rich formatting and error messaging
- [ ] T022 [US2] Register `show` subcommand and arguments in `src/bin/marketplace.rs`
- [ ] T023 [P] [US2] Add `GET /marketplace/extensions/{extensionId}` route in `src/marketplace/api/extensions.rs` returning `ExtensionDetail`

**Checkpoint**: User Stories 1 & 2 independently deliverable

---

## Phase 5: User Story 3 - Search and Filter Extensions (Priority: P3)

**Goal**: Allow users to search and filter extensions by keyword, category, source, and installed status with optional pre-fetch filtering.

**Independent Test**: Use `gemini marketplace list --search foo --category analytics --installed false` to confirm results filter correctly and pre-fetch mode limits outbound requests.

### Tests for User Story 3

- [ ] T024 [US3] Add `tests/integration/search_extensions.rs` to verify local filtering, pre-fetch optimization toggles, category filtering, and installed-only views

### Implementation for User Story 3

- [ ] T025 [US3] Enhance `src/marketplace/services/catalog.rs` with search/filter parameters, pre-fetch optimization, and metrics for filtered counts
- [ ] T026 [P] [US3] Extend `src/marketplace/commands/list.rs` to accept `--search`, `--category`, `--source`, and `--installed` flags plus toggle for pre-fetch mode
- [ ] T027 [P] [US3] Update `src/marketplace/api/extensions.rs` to parse query params and delegate to the enhanced search logic

**Checkpoint**: User Stories 1–3 independently testable

---

## Phase 6: User Story 4 - Manage Marketplace Sources (Priority: P4)

**Goal**: Enable users to add, list, and remove marketplace sources, configure cache/search preferences, and observe refresh/status information respecting credential helpers.

**Independent Test**: Run `gemini marketplace sources add ...`, `sources ls`, `sources rm`, and `marketplace status` to verify source lifecycle, preference persistence, and refresh queuing across authenticated/private repositories.

### Tests for User Story 4

- [ ] T028 [US4] Create `tests/integration/manage_sources.rs` covering add/list/remove, preference persistence (TTL & search mode), refresh queuing, and credential-helper reliance

### Implementation for User Story 4

- [ ] T029 [US4] Persist user preferences (TTL, search mode) in `src/marketplace/services/preferences.rs` and expose safe getters/setters
- [ ] T030 [US4] Implement source registry lifecycle in `src/marketplace/services/sources.rs`, including validation, namespacing, and skip warnings
- [ ] T031 [P] [US4] Build `sources` CLI subcommands in `src/marketplace/commands/sources.rs` for add/list/remove utilizing preference + source services
- [ ] T032 [US4] Register nested `sources` CLI tree within `src/bin/marketplace.rs`
- [ ] T033 [P] [US4] Implement `/marketplace/sources` REST routes in `src/marketplace/api/sources.rs` (GET/POST/DELETE) aligned to OpenAPI contract
- [ ] T034 [US4] Implement refresh scheduler and rate-limit countdown in `src/marketplace/services/refresh.rs`
- [ ] T035 [P] [US4] Add CLI commands in `src/marketplace/commands/refresh.rs` for manual refresh and status inspection
- [ ] T036 [P] [US4] Expose `/marketplace/cache/refresh` and `/marketplace/status` endpoints in `src/marketplace/api/status.rs` returning queue + cache metrics

**Checkpoint**: All user stories complete and independently verifiable

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Repository-wide QA and documentation alignment

- [ ] T037 [Polish] Update `quickstart.md` with final CLI commands, Windows notes for credential helpers, and testing instructions
- [ ] T038 [Polish] Add developer-facing usage docs/README snippet under `docs/marketplace.md` summarizing commands and API endpoints
- [ ] T039 [Polish] Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and full `cargo test` to ensure clean build before review

---

## Dependencies & Execution Order

### Phase Dependencies
- **Phase 1 → Phase 2**: Foundational work depends on crate/tooling setup (T001–T004).
- **Phase 2 → Phases 3–6**: All user story phases require foundational modules, config, cache, and server scaffolding (T005–T013).
- **Phase 3 (US1)**: Must finish before later stories to supply shared catalog capabilities.
- **Phase 4 (US2)** and **Phase 5 (US3)**: Depend on US1 service infrastructure but can start once US1 service (T015) is stable.
- **Phase 6 (US4)**: Depends on Phase 2 infrastructure and US1 cache/service mechanics.
- **Phase 7 (Polish)**: Runs after desired user stories are complete.

### User Story Dependencies
- **US1**: Independent after foundational phase; delivers MVP.
- **US2**: Depends on US1’s catalog foundations.
- **US3**: Builds on US1 listing logic to extend search/filtering.
- **US4**: Extends shared services and introduces new management modules; does not block earlier stories.

### Within-Story Ordering Highlights
- Tests (T014, T019, T024, T028) precede implementation tasks for their stories.
- Services (T015, T020, T025, T030, T034) land before CLI/API wiring tasks that rely on them.
- CLI wiring in `src/bin/marketplace.rs` (T017, T022, T032) must follow individual command implementations.

---

## Parallel Opportunities
- **Foundation**: After T005, configuration (T006) and error handling (T007) can progress in parallel.
- **US1**: Once catalog logic (T015) is in place, CLI rendering (T016) and REST route (T018) can proceed concurrently.
- **US2**: Detail command (T021) and API route (T023) operate in different files post-service update (T020).
- **US3**: After enhancing the service (T025), CLI flag wiring (T026) and API query parsing (T027) can run side by side.
- **US4**: With services ready (T030, T034), CLI commands (T031, T035) and HTTP routes (T033, T036) offer multiple parallel tracks.

---

## Parallel Execution Examples

- **User Story 1**: After completing T015, run T016 and T018 simultaneously to implement CLI and API surfaces for listing.
- **User Story 2**: Following T020, split work so one developer handles T021 (CLI presenter) while another handles T023 (detail API route).
- **User Story 3**: With T025 done, execute T026 (CLI flags) and T027 (API query parsing) in parallel to cover both interfaces.
- **User Story 4**: Once T030 and T034 are finished, parallelize T031/T035 (CLI subcommands) and T033/T036 (REST endpoints).

---

## Implementation Strategy

### MVP First (User Story 1 Only)
1. Complete Phases 1–2 to establish the crate, infrastructure, and services.
2. Deliver Phase 3 (US1) to provide browsing capability — this is the minimum viable marketplace.
3. Validate via T014 and ensure cached listings function before proceeding.

### Incremental Delivery
1. Ship US1 (P1) for initial marketplace visibility.
2. Layer on US2 (P2) to add detail views without disrupting listing functionality.
3. Introduce US3 (P3) to improve discoverability through search/filtering.
4. Finish with US4 (P4) for source management, preferences, and refresh/status tooling.
5. Apply Phase 7 polish tasks before requesting review or release.

### Parallel Team Strategy
1. Team collaborates on Phases 1–2.
2. Assign US1 to Developer A to secure MVP.
3. While US1 stabilizes, Developer B can begin US2 service extensions once T015 is merged.
4. Developer C can prepare US4 services in parallel after foundational tasks, aligning integrations once US1 service contracts are stable.
5. Use the parallel opportunities list to avoid file conflicts and maintain independent delivery per story.

---

**Suggested MVP Scope**: Complete through Phase 3 (User Story 1). Subsequent stories enhance depth but are not required for initial release.
