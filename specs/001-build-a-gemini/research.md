# Research: Gemini CLI Extension Marketplace

## Summary

Research focused on confirming the CLI's usability, data validation, cache design, observability, and background refresh. These areas are critical for meeting the project's requirements.

### CLI Pagination & Interaction Model
Decision: Keep `gemini marketplace list` single-shot by default with opt-in interactive paging (`--interactive`) and explicit navigation commands.  
Rationale: Mirrors familiar `cargo`/`git` patterns, reduces statefulness, and simplifies automation since default output stays non-interactive. Interactive mode still supports long sessions without re-running the command.  
Alternatives considered: Always-interactive pager (rejected—breaks piping/automation); environment-based pager integration only (rejected—less discoverable for CLI users).

### Manifest Validation Strategy
Decision: Apply schema validation (`schemars`) on fetch, then run semantic checks (semver, URLs, capabilities) lazily when users request details.  
Rationale: Balances responsiveness with correctness; avoids blocking list views on expensive checks while guaranteeing detail view accuracy.  
Alternatives considered: Full validation on fetch (rejected—slows pagination); minimal validation only (rejected—risks surfacing broken data).

### Cache TTL & Refresh Mechanism
Decision: Persist TTL metadata alongside cached payloads, defaulting to 24 hours, allow user overrides via `cache ttl set`, and trigger background refresh when TTL expires or user runs `cache refresh`.  
Rationale: Aligns with the requirement for offline use and user control over freshness/API usage trade-offs, as outlined in the project's core principles.  
Alternatives considered: Hard-coded TTL (rejected—lacks flexibility); on-demand fetching without cache (rejected—breaks offline requirement and risks rate limiting).

### Observability & Metrics
Decision: Instrument asynchronous flows with `tracing` spans (INFO level), emit structured metrics (`marketplace.cache_hits`, `marketplace.rate_limit_wait_seconds`, etc.), and expose verbose mode trace IDs for support.  
Rationale: Meets the observability requirements, makes triage feasible, and keeps observability consistent with existing repository patterns.  
Alternatives considered: Logging only (rejected—no metrics for SC-005); external metrics dependency (rejected—adds setup burden).

### Background Refresh & Rate Limits
Decision: Queue refresh jobs when API errors or rate limits occur, wait for reset windows before retrying, and continue serving cached data with visible countdowns.  
Rationale: Honours GitHub rate limits, prevents user downtime, and integrates with existing service abstractions.  
Alternatives considered: Immediate retries (rejected—would thrash rate limits); silent failures (rejected—violates user transparency goals).
