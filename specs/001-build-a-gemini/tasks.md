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

- [X] T001 [Setup] Scaffold Rust crate with `Cargo.toml`, `src/lib.rs`, and `src/bin/marketplace.rs` aligned to the planned structure
- [X] T002 [Setup] Declare required crates in `Cargo.toml` (`tokio`, `reqwest`, `serde`, `directories`, `thiserror`, `anyhow`, `indicatif`, dev `assert_cmd`, `insta`, `wiremock`)
- [X] T003 [Setup] Add repository-wide tooling configs (`rust-toolchain.toml`, `.cargo/config.toml`, `clippy.toml`) enforcing Rust 1.82.0 and lint rules
- [X] T004 [Setup] Create testing scaffolds (`tests/integration/`, `tests/unit/`, `tests/common/mod.rs`) with placeholder harness setup

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T005 [Foundation] Scaffold `src/marketplace/` module tree with stub files (`mod.rs`, `commands/mod.rs`, `services/mod.rs`, `models/{manifest.rs,domain.rs}`, `cache/mod.rs`, `api/mod.rs`)
- [X] T006 [Foundation] Write unit tests for configuration helper path resolution in `tests/unit/config.rs`
- [X] T007 [Foundation] Implement configuration helpers in `src/marketplace/config.rs` to resolve cache and preferences directories using the `directories` crate
- [X] T008 [Foundation] Write unit tests for shared error mappings in `tests/unit/error.rs`
- [X] T009 [Foundation] Introduce shared error enums and result aliases in `src/marketplace/error.rs` leveraging `thiserror`
- [X] T010 [Foundation] Write unit tests for manifest parsing/normalization in `tests/unit/manifest.rs`
- [X] T011 [Foundation] Parse `gemini-extension.json` manifests in `src/marketplace/models/manifest.rs`, including validation and warning aggregation
- [X] T012 [Foundation] Write unit tests for domain entities and install state transitions in `tests/unit/domain.rs`
- [X] T013 [Foundation] Model domain entities and install state in `src/marketplace/models/domain.rs` per `data-model.md`
- [X] T014 [Foundation] Write integration tests for JSON cache persistence in `tests/integration/cache_store.rs`
- [X] T015 [Foundation] Implement JSON cache store with TTL metadata in `src/marketplace/cache/store.rs`
- [X] T016 [Foundation] Write integration tests for source synchronization with rate-limit simulation in `tests/integration/source_fetcher.rs`
- [X] T017 [Foundation] Build source fetcher with GitHub rate-limit queue handling in `src/marketplace/services/source_fetcher.rs`
- [X] T018 [Foundation] Write CLI parsing tests using `assert_cmd` for command routing in `tests/integration/cli_parse.rs`
- [X] T019 [Foundation] Set up Clap-driven CLI bootstrap and command routing skeleton in `src/bin/marketplace.rs`
- [X] T020 [Foundation] Write integration tests for Axum server bootstrap in `tests/integration/api_server.rs`
- [X] T021 [Foundation] Implement lightweight HTTP server harness in `src/marketplace/api/server.rs` ready to host OpenAPI routes
- [ ] T022 [Foundation] Write unit tests for default curated source configuration in `tests/unit/default_source.rs`
- [ ] T023 [Foundation] Seed default curated marketplace source configuration and validation logic in `src/marketplace/services/sources.rs`

**Checkpoint**: Foundation ready — user story implementation can now begin **(commit & open PR with setup/foundation)**

---

## Phase 3: User Story 1 - Browse Available Extensions (Priority: P1) 🎯 MVP

**Goal**: Allow users to list available extensions with key metadata, respecting cache and install status requirements.

**Independent Test**: Run `gemini marketplace list` with mocked sources to confirm the CLI displays names, descriptions, versions, install status, and source namespaces within 2 seconds using cached data.

### Tests for User Story 1

- [X] T024 [US1] Author `tests/integration/list_extensions.rs` using `wiremock` + `assert_cmd` to assert list output, caching, and warning behavior when manifests are skipped
- [X] T025 [US1] Write tests for network failure messaging and cache fallback in `tests/integration/list_extensions.rs`

### Implementation for User Story 1

- [X] T026 [US1] Implement listing workflow in `src/marketplace/services/catalog.rs` to merge cache, fetch manifests, compute install status, and skip invalid manifests with warnings
- [X] T027 [P] [US1] Render tabular and JSON outputs in `src/marketplace/commands/list.rs`, including namespace formatting and pagination support
- [X] T028 [US1] Wire the `list` command and options into Clap routing within `src/bin/marketplace.rs`
- [ ] T029 [P] [US1] Expose `GET /marketplace/extensions` handler in `src/marketplace/api/extensions.rs` returning `ExtensionListResponse`
- [X] T030 [US1] Implement graceful network-error messaging and fallback pathways across CLI and API list flows

**Checkpoint**: User Story 1 independently testable (MVP) **(commit & extend PR or open US1 PR)**

---

## Phase 4: User Story 2 - View Extension Details (Priority: P2)

**Goal**: Provide detailed information for a selected extension, including docs, compatibility, and install guidance.

**Independent Test**: Execute `gemini marketplace show <source/extension>` to confirm extended metadata renders, README excerpts are available, and install instructions are surfaced.

### Tests for User Story 2

- [ ] T031 [US2] Create `tests/integration/view_extension_details.rs` covering detail retrieval, README excerpt rendering, and missing extension 404 handling

### Implementation for User Story 2

- [ ] T032 [US2] Extend `src/marketplace/services/catalog.rs` with detail retrieval, README extraction, and manifest checksum exposure
- [ ] T033 [P] [US2] Implement `show` command presenter in `src/marketplace/commands/show.rs` with rich formatting and error messaging
- [ ] T034 [US2] Register `show` subcommand and arguments in `src/bin/marketplace.rs`
- [ ] T035 [P] [US2] Add `GET /marketplace/extensions/{extensionId}` route in `src/marketplace/api/extensions.rs` returning `ExtensionDetail`

**Checkpoint**: User Stories 1 & 2 independently deliverable **(commit & open/extend PR for US2)**

---

## Phase 5: User Story 3 - Search and Filter Extensions (Priority: P3)

**Goal**: Allow users to search and filter extensions by keyword, category, source, and installed status with optional pre-fetch filtering.

**Independent Test**: Use `gemini marketplace list --search foo --category analytics --installed false` to confirm results filter correctly and pre-fetch mode limits outbound requests.

### Tests for User Story 3

- [ ] T036 [US3] Add `tests/integration/search_extensions.rs` to verify local filtering, pre-fetch optimization toggles, category filtering, and installed-only views

### Implementation for User Story 3

- [ ] T037 [US3] Enhance `src/marketplace/services/catalog.rs` with search/filter parameters, pre-fetch optimization, and metrics for filtered counts
- [ ] T038 [P] [US3] Extend `src/marketplace/commands/list.rs` to accept `--search`, `--category`, `--source`, and `--installed` flags plus toggle for pre-fetch mode
- [ ] T039 [P] [US3] Update `src/marketplace/api/extensions.rs` to parse query params and delegate to the enhanced search logic

**Checkpoint**: User Stories 1–3 independently testable **(commit & open/extend PR for US3)**

---

## Phase 6: User Story 4 - Manage Marketplace Sources (Priority: P4)

**Goal**: Enable users to add, list, and remove marketplace sources, configure cache/search preferences, and observe refresh/status information respecting credential helpers.

**Independent Test**: Run `gemini marketplace sources add ...`, `sources ls`, `sources rm`, and `marketplace status` to verify source lifecycle, preference persistence, and refresh queuing across authenticated/private repositories.

### Tests for User Story 4

- [ ] T040 [US4] Create `tests/integration/manage_sources.rs` covering add/list/remove, preference persistence (TTL & search mode), refresh queuing, and credential-helper reliance
- [ ] T041 [US4] Write tests for dual-mode logging and metrics emission in `tests/integration/observability.rs`

### Implementation for User Story 4

- [ ] T042 [US4] Persist user preferences (TTL, search mode) in `src/marketplace/services/preferences.rs` and expose safe getters/setters
- [ ] T043 [US4] Implement source registry lifecycle in `src/marketplace/services/sources.rs`, including validation, namespacing, and skip warnings
- [ ] T044 [P] [US4] Build `sources` CLI subcommands in `src/marketplace/commands/sources.rs` for add/list/remove utilizing preference + source services
- [ ] T045 [US4] Register nested `sources` CLI tree within `src/bin/marketplace.rs`
- [ ] T046 [P] [US4] Implement `/marketplace/sources` REST routes in `src/marketplace/api/sources.rs` (GET/POST/DELETE) aligned to OpenAPI contract
- [ ] T047 [US4] Implement refresh scheduler and rate-limit countdown in `src/marketplace/services/refresh.rs`
- [ ] T048 [P] [US4] Add CLI commands in `src/marketplace/commands/refresh.rs` for manual refresh and status inspection
- [ ] T049 [P] [US4] Expose `/marketplace/cache/refresh` and `/marketplace/status` endpoints in `src/marketplace/api/status.rs` returning queue + cache metrics
- [ ] T050 [US4] Implement credential-helper detection, warnings, and documentation hooks in `src/marketplace/services/sources.rs`
- [ ] T051 [US4] Implement dual-mode logging and structured metrics emission across CLI and API paths

**Checkpoint**: All user stories complete and independently verifiable **(commit & open/extend PR for US4)**

---

## Phase 7: Gemini CLI Integration (Pre-Release)

**Purpose**: Validate the extension when invoked through the Gemini CLI binary. Keep this phase separate to prevent breakages in Gemini’s loader surface before polish work begins. **(commit once integration checks pass; consider pre-release PR)**

- [ ] T052 [Integration] Create integration harness to link the built extension via `gemini extensions path add` and manage temporary config overrides
- [ ] T053 [Integration] Execute smoke tests (`gemini marketplace list/show/status`) against mock data, capturing outputs for regression comparison
- [ ] T054 [Integration] Document Gemini CLI integration workflow in `docs/integration.md` (or similar) for repeatable validation steps

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Repository-wide QA and documentation alignment

- [ ] T055 [Polish] Update `quickstart.md` with final CLI commands, Windows notes for credential helpers, and testing instructions
- [ ] T056 [Polish] Update `README.md` / `docs/marketplace.md` summary with observability + credential guidance aligned to NFR-001 and FR-013c
- [ ] T057 [Polish] Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, and full `cargo test` to ensure clean build before review **(final commit & release PR)**

---

## Dependencies & Execution Order

### Phase Dependencies
- **Phase 1 → Phase 2**: Foundational work depends on crate/tooling setup (T001–T004).
- **Phase 2 → Phases 3–6**: All user story phases require foundational modules, config, cache, server scaffolding, and default source (T005–T023).
- **Phase 3 (US1)**: Must finish before later stories to supply shared catalog capabilities and network error handling.
- **Phase 4 (US2)** and **Phase 5 (US3)**: Depend on US1 service infrastructure but can start once listing service (T026) is stable.
- **Phase 6 (US4)**: Depends on Phase 2 infrastructure and US1 catalog logic.
- **Phase 7 (Integration)**: Runs after user story implementation to validate Gemini CLI end-to-end behavior.
- **Phase 8 (Polish)**: Executes once integration checks are complete.

### User Story Dependencies
- **US1**: Independent after foundational phase; delivers MVP.
- **US2**: Depends on US1’s catalog foundations.
- **US3**: Builds on US1 listing logic to extend search/filtering.
- **US4**: Extends shared services and introduces new management + observability modules; does not block earlier stories.

### Within-Story Ordering Highlights
- Tests precede implementation tasks for every feature per constitution (e.g., T010 → T011, T024 → T026).
- Services (T026, T032, T037, T043, T047) land before CLI/API wiring tasks that rely on them.
- CLI wiring in `src/bin/marketplace.rs` (T028, T034, T045) must follow individual command implementations.
- Integration harness tasks (T052–T054) should use built artifacts from prior phases.

---

## Parallel Opportunities
- **Foundation**: After T005 scaffolds modules, configuration (T006–T007) and error handling (T008–T009) can proceed alongside manifest work; cache (T014–T015) and fetcher (T016–T017) can run in parallel once dependencies mocked.
- **US1**: Following T026 service completion, CLI rendering (T027) and REST route (T029) can progress concurrently while network-error work (T030) finalizes messaging.
- **US2**: Detail command (T033) and API route (T035) operate in different files post-service update (T032).
- **US3**: With T037 done, CLI flag wiring (T038) and API query parsing (T039) can run side by side.
- **US4**: After T043 and T047 establish services, CLI commands (T044, T048) and HTTP routes (T046, T049) offer multiple parallel tracks, while observability (T041, T051) can execute with minimal overlap.
- **Integration**: Gemini CLI smoke checks (T052–T054) rely on compiled binaries; these can run concurrently once US4 is done.

---

## Parallel Execution Examples

- **User Story 1**: After completing T026, run T027 and T029 simultaneously to implement CLI and API surfaces for listing.
- **User Story 2**: Following T032, split work so one developer handles T033 (CLI presenter) while another handles T035 (detail API route).
- **User Story 3**: With T037 done, execute T038 (CLI flags) and T039 (API query parsing) in parallel to cover both interfaces.
- **User Story 4**: Once T043 and T047 are finished, parallelize T044/T048 (CLI subcommands) and T046/T049 (REST endpoints), while observability tasks (T041, T051) run independently.

---

## Implementation Strategy

### MVP First (User Story 1 Only)
1. Complete Phases 1–2 to establish the crate, infrastructure, and services.
2. Deliver Phase 3 (US1) to provide browsing capability — this is the minimum viable marketplace.
3. Validate via T024–T030 and ensure cached listings function before proceeding.

### Incremental Delivery
1. Ship US1 (P1) for initial marketplace visibility.
2. Layer on US2 (P2) to add detail views without disrupting listing functionality.
3. Introduce US3 (P3) to improve discoverability through search/filtering.
4. Finish with US4 (P4) for source management, observability, preferences, and refresh/status tooling.
5. Run Phase 7 integration tasks to validate Gemini CLI end-to-end behavior.
6. Apply Phase 8 polish tasks before requesting review or release.

### Parallel Team Strategy
1. Team collaborates on Phases 1–2.
2. Assign US1 to Developer A to secure MVP while Developer B prepares US2 service extensions once T026 is merged.
3. Developer C can begin US4 observability groundwork (T041, T051) after foundational tasks using mocks.
4. Use the parallel opportunities list to avoid file conflicts and maintain independent delivery per story.

---

**Suggested MVP Scope**: Complete through Phase 3 (User Story 1). Subsequent stories enhance depth but are not required for initial release.
