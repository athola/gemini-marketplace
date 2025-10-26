# Phase 0 Research – Gemini CLI Extension Marketplace

## Runtime & Toolchain
- **Decision**: Adopt Rust 1.82.0 with stable MSRV enforcement across the workspace.
- **Rationale**: Aligns with constitutional dependency stewardship, guarantees async/trait
  ergonomics required by modern crates, and matches the Gemini CLI extension ecosystem.
- **Alternatives considered**: Staying on an older MSRV (1.74–1.79) would block `tokio`/`reqwest`
  upgrades and risk missing bug fixes; nightly toolchains rejected to avoid stability concerns.

## Async Networking Stack
- **Decision**: Pair `reqwest` with `tokio` for HTTP requests and concurrency.
- **Rationale**: Mature ecosystem, robust GitHub API support, built-in retry/backoff hooks, and
  compatibility with structured metrics instrumentation.
- **Alternatives considered**: `ureq` lacks async; `surf` introduces dependency sprawl; direct
  `hyper` adds boilerplate for JSON handling without measurable benefit.

## Manifest Validation Strategy
- **Decision**: Execute schema validation on fetch via `schemars`-generated schema + `serde`,
  then run semantic validation (semver parsing, URL checks, README extraction) when details are
  requested.
- **Rationale**: Matches progressive validation requirement, avoids over-fetching, and provides
  actionable warnings aligned with user flows.
- **Alternatives considered**: Performing full validation during fetch would slow catalog loads;
  deferring all validation to detail views risks polluting list results with invalid entries.

## Caching & Persistence
- **Decision**: Store cache artifacts under `$GEMINI_CONFIG/extensions/marketplace/`, using the
  `directories` crate for cross-platform paths and JSON metadata describing TTL, source hashes,
  and queue state.
- **Rationale**: Satisfies constitution constraint for cache location, keeps filesystem logic
  centralized, supports deterministic offline behaviour, and integrates with background refresh.
- **Alternatives considered**: `dirs` crate deprecated; manual path assembly risks diverging
  across platforms; database-backed cache (e.g., SQLite) adds complexity without clear value.

## Observability & Metrics
- **Decision**: Emit human-readable logs via `tracing` + `tracing-subscriber`, expose structured
  JSON logs when `--json` enabled, and collect metrics (cache hits/misses, rate-limit waits,
  retry queue depth, top search terms for SC-005) through a lightweight in-process registry
  surfaced by CLI/API endpoints.
- **Rationale**: Provides consistent diagnostics across commands, supports countdown UX, and
  keeps instrumentation overhead minimal.
- **Alternatives considered**: Relying solely on stdout logging would complicate filtering;
  integrating Prometheus exporters is overkill for local CLI use cases.
