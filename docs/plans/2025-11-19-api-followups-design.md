# API Follow-ups Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Address API review follow-ups (OpenAPI governance, module docs, env var references, local publish defaults).

**Architecture:** Documentation-first pass touching README/Quickstart/commands docs + Makefile; add tooling hook for OpenAPI linting (placeholder script). No executable code changes besides doc comments and build scripts.

**Tech Stack:** Rust workspace (cargo, make), Markdown docs, OpenAPI file.

## Task 1: Publish & Lint OpenAPI Spec

**Files:**
- Modify: `README.md`, `specs/001-build-a-gemini/quickstart.md`
- Create: `scripts/lint-openapi.sh`
- Modify: `Makefile`

**Steps:**
1. Add "Contracts" section to README referencing `specs/001-build-a-gemini/contracts/marketplace-openapi.yaml` with instructions.
2. Add Quickstart bullet describing contract location + lint command.
3. Create `scripts/lint-openapi.sh` (bash) that runs `npx @redocly/cli lint` if available, otherwise warns (no install).
4. Update Makefile with `contract-lint` target calling the script; add to `help` output.
5. (Doc note) Mention in README that CI should run `make contract-lint` (actual CI change deferred).

## Task 2: Document crate modules & stability

**Files:**
- Modify: `crates/marketplace-core/src/lib.rs`
- Modify: `README.md`
- Create/Modify: `CHANGELOG.md`

**Steps:**
1. Add crate-level doc comment summarizing API layers and stability promises.
2. Add per-module doc comments for `cli`, `marketplace`, `telemetry` describing scope.
3. Introduce README section on SemVer/stability referencing CHANGELOG.
4. Create or update `CHANGELOG.md` with initial entry describing workspace split + HPC release (YYYY-MM-DD header).

## Task 3: Environment variable reference

**Files:**
- Modify: `README.md`
- Modify: `commands/README.txt`

**Steps:**
1. Add table enumerating env vars (GEMINI_MARKETPLACE_HOME, GEMINI_MARKETPLACE_SOURCE_URL, GEMINI_MARKETPLACE_LOG, MARKETPLACE_CLI_BIN, MARKETPLACE_MCP_SERVER_BIN) with descriptions/defaults/stability.
2. Update commands README to mention same env vars for slash command context.

## Task 4: Local publish defaults

**Files:**
- Modify: `Makefile`
- Modify: `specs/001-build-a-gemini/quickstart.md`

**Steps:**
1. Update `local-publish` target to `mkdir -p $(EXTENSION_DIRS)` before deleting; when `rm -rf` fails, show instructions referencing README Env section.
2. Document in Quickstart how to set `GEMINI_CONFIG` to an owned location and mention permission requirement.

