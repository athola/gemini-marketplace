---

description: "Task list for the Gemini CLI Marketplace"
---

# Tasks: Gemini CLI Marketplace

**Input**: Design documents from `/specs/001-build-a-gemini/`
**Prerequisites**: plan.md (required), spec.md, research.md, data-model.md, contracts/, quickstart.md

**Tests**: Start every user story with failing tests (unit, integration, snapshot, contract) before implementation. Capture the commands and selective reruns before creating a pull request. Log CPU/GPU baselines and minutes burned for heavy suites, per Constitution Principle VI.

**Organization**: Tasks are grouped by user story to ensure each increment is independently testable.

## Phase 1: Setup (Shared Infrastructure)

- [ ] **T001**: Document Rust 1.82.0 and Gemini CLI prerequisites and baseline commands in `specs/001-build-a-gemini/quickstart.md`.
- [ ] **T002**: Align `Makefile` targets (`format`, `lint`, `test`) to call workspace-wide `cargo` commands.
- [ ] **T003**: Summarize the current CLI contract surfaces (`list`, `show`, `search`, `sources`, `cache`, `status`) in `README.md` to keep the spec, plan, and help text in sync.

---

## Phase 2: Foundational (Blocking Prerequisites)

- [ ] **T004**: Convert the root `Cargo.toml` into a workspace manifest and move the existing crate into `crates/marketplace-core/Cargo.toml`.
- [ ] **T005**: Update `specs/001-build-a-gemini/quickstart.md` with workspace build instructions and selective rerun guidance.
- [ ] **T006**: Implement `Extension`, `MarketplaceSource`, and `ManifestCacheEntry` structs in `crates/marketplace-core/src/marketplace/models/{domain.rs,manifest.rs}` with serde and schemars schemas.
- [ ] **T007**: Add a manifest schema and semantic validation pipeline in `crates/marketplace-core/src/marketplace/ingest/validation.rs`.
- [ ] **T008**: Build a cache store with TTL metadata and checksum enforcement in `crates/marketplace-core/src/marketplace/cache/store.rs`.
- [ ] **T009**: Persist user preferences and custom sources in `crates/marketplace-core/src/marketplace/services/preferences.rs`.
- [ ] **T010**: Implement a dual-mode telemetry logger and counters in `crates/marketplace-core/src/telemetry/mod.rs`.
- [ ] **T011**: Add a background refresh queue with retry handling in `crates/marketplace-core/src/marketplace/services/refresh.rs`.

---

## Phase 2b: MCP Workspace Enablement

- [ ] **T012**: Create `crates/marketplace-mcp-server/Cargo.toml` and `src/main.rs` wiring `rmcp` stdio server entry points to marketplace handlers.
- [ ] **T013**: Implement an MCP tool registry and request routing in `crates/marketplace-mcp-server/src/tools.rs`.
- [ ] **T014**: Build a developer harness that auto-spawns the MCP server child process and proxies CLI-style arguments over MCP.
- [ ] **T015**: Add integration tests that exercise CLI-to-MCP-server round trips in `tests/integration/mcp_cli.rs`.

---

## Phase 3: User Story 1 – Browse Available Extensions (P1 MVP)

**Goal**: List and catalog extensions with pagination, metadata columns, and cache freshness badges.

**Test**: `cargo test tests/contract/list_extensions.rs tests/snapshot/list_command.rs tests/integration/list_cli.rs -p marketplace-core`

### Tests

- [ ] **T016**: Add a contract test for the `/extensions` listing.
- [ ] **T017**: Add a CLI snapshot test for the `gemini marketplace list` columns.
- [ ] **T018**: Add an integration test for interactive pagination.

### Implementation

- [ ] **T019**: Implement a paged list handler.
- [ ] **T020**: Implement an interactive pagination controller.
- [ ] **T021**: Render cache freshness badges, warnings, and other badges.
- [ ] **T022**: Wire lazy loading through the catalog service.
- [ ] **T023**: Update the CLI help text and `README.md` for the listing commands.

---

## Phase 4: User Story 2 – View Extension Details (P2)

**Goal**: Display detailed metadata, README excerpts, install instructions, and validation diagnostics for an extension.

**Test**: `cargo test tests/integration/show_extension.rs tests/snapshot/show_command.rs -p marketplace-core`

### Tests

- [ ] **T024**: Add an integration test for the `gemini marketplace show <namespace>` success path.
- [ ] **T025**: Add a failure-path test to ensure that semantic validation errors are surfaced.
- [ ] **T026**: Add a CLI snapshot test that covers the detail output and diagnostics.

### Implementation

- [ ] **T027**: Implement detail fetching and schema/semantic validation gating.
- [ ] **T028**: Render documentation, README excerpts, and install instructions.
- [ ] **T029**: Surface manifest diagnostics (e.g., invalid semver, checksum mismatch).
- [ ] **T030**: Document installation workflow updates in the quickstart guide.

---

## Phase 5: User Story 3 – Search and Filter Extensions (P3)

**Goal**: Support keyword and category filtering, both locally and via server pre-filtering.

**Test**: `cargo test tests/unit/search_filter.rs tests/integration/search_command.rs -p marketplace-core`

### Tests

- [ ] **T031**: Add a unit test for local keyword filtering.
- [ ] **T032**: Add an integration test for the category filter and reset behavior.

### Implementation

- [ ] **T033**: Implement the search CLI entrypoint (flags, category filtering).
- [ ] **T034**: Implement dual-mode filtering (local vs. server pre-filter).
- [ ] **T035**: Update the CLI help and `README.md` for search usage.

---

## Phase 6: User Story 4 – Manage Marketplace Sources (P4)

**Goal**: Add, list, and remove custom sources with aliases and recursion-depth controls.

**Test**: `cargo test tests/integration/sources_command.rs tests/unit/source_alias.rs -p marketplace-core`

### Tests

- [ ] **T036**: Add an integration test that covers the `sources add/remove/list` flows.
- [ ] **T037**: Add a unit test for the alias prompt defaulting to a slug.

### Implementation

- [ ] **T038**: Implement `sources add` with alias prompt and recursion depth options.
- [ ] **T039**: Implement `list` and `remove` subcommands that reflect source metadata.
- [ ] **T040**: Extend preferences storage to persist aliases and depths.
- [ ] **T041**: Update the quickstart for source management.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] **T042**: Harden the telemetry export and opt-out plumbing.
- [ ] **T043**: Add a human/JSON log parity verification document.
- [ ] **T044**: Run the selective rerun evidence script and log CPU/GPU usage.
- [ ] **T045**: Update `README.md` and `CHANGELOG.md` with new commands, the MCP architecture, and configuration paths.
- [ ] **T046**: Prepare release checklist entries in the release issue template.

---

## Dependencies & Execution Order

1.  **Phase 1** (Setup) → **Phase 2** (Foundational) → **Phase 2b** (MCP Workspace) → **Phase 3** (US1) → **Phase 4** (US2) → **Phase 5** (US3) → **Phase 6** (US4) → **Phase 7** (Polish).
2.  User stories depend on the foundational and MCP phases. US2–US4 depend on US1 data structures and patterns.
3.  MCP CLI/server tasks (T012–T015) must finish before the user stories so that Gemini and the test harness share the same implementation surface.

## Parallel Execution Examples

*   The tests for US1 (T016–T018) can run in parallel once the foundational work is done.
*   The US2 tests (T024–T026) and implementation (T027–T029) can be split across separate files, allowing for parallel work.
*   The US3 tests (T031, T032) and services (T033, T034) can proceed concurrently once the catalog paging logic (T022) lands.
*   The tests for T036/T037 and the implementation of T038–T040 can run in parallel after the preferences service (T009) is ready.

## Implementation Strategy

-   **MVP**: Deliver through Phase 3 (US1) to ship list/pagination functionality backed by the MCP server.
-   **Increment 2**: Complete US2 for the detail view and diagnostics.
-   **Increment 3**: Deliver US3 search/filter flows.
-   **Increment 4**: Ship US4 source management.
-   **Final Polish**: Observability, docs, and release artifacts once all user stories pass independent tests.
