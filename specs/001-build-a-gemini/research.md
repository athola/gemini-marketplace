# Research – Gemini CLI Extension Marketplace

This document outlines the key decisions made during the research phase of the Gemini CLI Extension Marketplace project.

## Decision: Source Alias Prompt with Slug Default

-   **Rationale**: Using a slug as the default alias for a source makes namespaces predictable for users and automation, while still allowing teams to override them.
-   **Alternatives Considered**:
    -   Auto-generating incremental IDs was rejected because they are unreadable and brittle.
    -   Enforcing slug-only identifiers was rejected because of potential collisions between organizations with similar repository names.

## Decision: Progressive Manifest Validation

-   **Rationale**: A fast schema screening keeps list operations responsive, while full semantic validation on the `show` command surfaces detailed diagnostics only when needed.
-   **Alternatives Considered**:
    -   Full validation on every fetch was rejected because it would be too expensive for thousands of manifests.
    -   Trusting manifests without validation was rejected because it would undermine the reliability and security of the marketplace.

## Decision: Lazy Loading in 500-Extension Batches

-   **Rationale**: This batch size balances memory footprint and throughput, and aligns with our performance targets. It also allows for incremental streaming to the CLI UI.
-   **Alternatives Considered**:
    -   Fetching the entire catalog upfront was rejected due to long startup latency and potential memory issues.
    -   Smaller batches were rejected because they would result in too many network round trips for large catalogs.

## Decision: Offline-First Cache with Configurable TTL

-   **Rationale**: Users often rely on the Gemini CLI in constrained environments. A configurable TTL ensures data freshness while still allowing for offline access.
-   **Alternatives Considered**:
    -   No cache was rejected because the CLI would be unusable offline and would incur repeated API costs.
    -   A fixed TTL was rejected because it would not be flexible enough for different teams.

## Decision: Dual-Mode Observability

-   **Rationale**: This enables both interactive troubleshooting and automated ingestion into telemetry pipelines.
-   **Alternatives Considered**:
    -   Human-readable logs only were rejected because they would not allow for automated monitoring.
    -   Metrics-only was rejected because it would make local debugging more difficult.

## Decision: Token & Compute Budget Logging

-   **Rationale**: This complies with Constitution Principle VI, which requires that every heavy command log selective reruns, CPU/GPU minutes, and instrumentation choices so that auditors can confirm stewardship.
-   **Alternatives Considered**:
    -   Silent adherence was rejected because it would not provide an auditable trail.
    -   Post-hoc summaries only were rejected because they would miss per-command evidence and increase the risk of rework.

## Decision: Workspace & MCP Split

-   **Rationale**: Shipping a Cargo workspace with `marketplace-core`, `marketplace-mcp-server`, and `marketplace-mcp-cli` lets Gemini consume a purpose-built MCP stdio server while developers keep a familiar CLI harness for local validation. This split enforces a single source of truth (the server) and avoids duplicating logic in other languages.
-   **Alternatives Considered**:
    -   A single CLI binary that conditionally runs as an MCP server was rejected because it would be harder to package separately for Gemini and would not provide a standalone test harness.
    -   Building the MCP server in Node.js was rejected because it would duplicate business logic in another stack and make it harder to maintain parity with the Rust core.
