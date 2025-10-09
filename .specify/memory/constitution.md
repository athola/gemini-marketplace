# Gemini Marketplace Constitution

## Core Principles

### I. Test-First CLI Delivery (NON-NEGOTIABLE)
Every new behavior ships with failing automated coverage first (unit or integration) using `cargo test`, `assert_cmd`, and `wiremock` where appropriate. Passing tests are the release gate for all work.

### II. Consistent Dual-Format UX
All commands must support both human-readable table output and JSON flags. Error messages return actionable remediation steps and non-zero exit codes. API surfaces mirror CLI behavior.

### III. Offline-First Resilience
Features must function against cached data when network calls fail. Cache TTL, invalidation, and warning pathways are mandatory considerations for each story.

### IV. Observability & Diagnostics
Structured logging (JSON when `--json` or env flag is set) and traceable error contexts are required. Rate-limit countdowns and skipped manifest warnings must surface to users.

### V. Dependency & Version Stewardship
The minimum supported Rust version (MSRV) is pinned at 1.82.0. New third-party crates require documented rationale in plan/research. Breaking toolchain updates need migration notes.

## Delivery Standards
- **Language & Tooling**: Rust 1.82.0 (stable toolchain), `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
- **Configuration**: `rust-toolchain.toml` enforced at repo root. All platform-specific paths go through the `directories` crate.
- **Security**: Credentials are never stored; rely on OS credential helpers. Commands handling private sources must warn users when creds are missing.
- **Performance**: Cached listing responses must render within 2 seconds; rate-limit queues must provide ETA feedback.

## Development Workflow & Quality Gates
1. Prepare design artifacts (spec, plan, research) before implementation.
2. Add failing tests for each task/story prior to implementation code.
3. Implement functionality respecting module boundaries (`src/marketplace/...`) and update documentation.
4. Run `cargo fmt`, `cargo clippy`, and full `cargo test` locally before requesting review.
5. Reviews check constitution compliance; violations require explicit justification in plan/tasks and approval.

## Governance
This constitution supersedes ad-hoc practices for the Gemini Marketplace project. Amendments require:
- Documented rationale in `research.md`
- Approval from project maintainers
- Version bump noted below with ratification date

**Version**: 1.0.0 | **Ratified**: 2025-10-09 | **Last Amended**: 2025-10-09
