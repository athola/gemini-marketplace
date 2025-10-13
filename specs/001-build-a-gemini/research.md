- Decision: Adopt a top-level `gemini marketplace` command suite with explicit subcommands (`list`, `show`, `search`, `sources add/list/remove`, `cache refresh`, `cache ttl set`) to mirror core user journeys without nesting under `gemini extensions`.
- Rationale: Keeps marketplace actions discoverable, aligns with clarified UX expectations, and simplifies help text while avoiding ambiguity between extension management and marketplace browsing.
- Alternatives considered: Nesting under `gemini extensions marketplace` (risks overly long command paths and confusion with core extension commands); single interactive prompt (`gemini marketplace browse`) (would restrict scripting/automation and obscure parity with JSON output).

- Decision: Store per-source marketplace responses as JSON files under `$GEMINI_CONFIG/extensions/marketplace/` with TTL metadata and manual refresh hooks.
- Rationale: Reuses existing Gemini CLI config patterns via the `directories` crate, enabling offline reads, configurable expiration, and incremental source refresh without external services.
- Alternatives considered: In-memory cache (fails offline requirement and loses data between sessions); embedded SQLite (adds dependency surface and exceeds constitution stewardship principle).

- Decision: Use progressive manifest validation—schema checks during fetch to populate listings, deeper semantic validation on demand when users run `gemini marketplace show`.
- Rationale: Balances responsiveness with accuracy, allowing lazy loading batches of 500 while deferring heavy validation until details are needed, consistent with spec FR-013a/c.
- Alternatives considered: Full validation at fetch time (slows initial listing and increases rate-limit exposure); no semantic validation (violates spec requirement for detailed error surfacing).

- Decision: Emit dual-mode telemetry by defaulting to human-readable tables with `indicatif`-style progress, while exposing `--json` output and structured metrics counters (cache hits, rate-limit waits, skipped manifests) for observability.
- Rationale: Satisfies constitution principles II and IV, aligns with clarified logging expectations, and preserves consistent scripting interfaces.
- Alternatives considered: Human-readable logs only (fails constitution dual-format commitment); external tracing backend (adds unnecessary dependencies at this stage).

- Decision: Default `gemini marketplace list` to single-shot execution while providing an opt-in `--interactive` flag for in-command pagination prompts (`next`/`prev`).
- Rationale: Preserves scriptability for common listing workflows and avoids unexpected blocking behavior, while still supporting interactive browsing when explicitly requested.
- Alternatives considered: Always-interactive mode (breaks automation use cases); separate interactive subcommand (adds redundant surface area).
