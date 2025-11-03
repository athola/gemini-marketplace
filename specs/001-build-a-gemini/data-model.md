# Data Model: Gemini CLI Extension Marketplace

## Entities

### Extension
- **Primary Identifier**: `namespace` (`<source>/<extension-name>`), stored as `String`
- **Key Fields**: `name`, `version`, `description`, `repository_url`, `categories: Vec<String>`, `compatibility: Vec<String>`, `install_status`, `warnings: Vec<String>`, `cache_freshness`
- **Relationships**: Belongs to exactly one `MarketplaceSource`; may reference zero or more `TelemetryEvent` records (search interactions)
- **Validation Rules**:
  - `version` MUST parse via `semver::Version`
  - `repository_url` MUST be HTTPS
  - `categories` limited to 10 entries, each ≤32 chars
  - `install_status` is enum: `Installed`, `NotInstalled`, `Unknown`
- **State Transitions**:
  - `NotInstalled` → `Installed` after registry/file-system detection succeeds
  - `Installed` → `NotInstalled` if both registry and file-system checks fail
  - `Unknown` only during initialization before detection runs

### MarketplaceSource
- **Primary Identifier**: `name` (`String`), unique per user profile
- **Key Fields**: `url`, `source_type` (`GitHubRepo`, `GitUrl`, `LocalPath`), `enabled`, `last_synced_at`, `recursion_limit`
- **Relationships**: Owns many `Extension` records; tracked by `Preferences`
- **Validation Rules**:
  - `url` MUST be valid URL or absolute path (for `LocalPath`)
  - `recursion_limit` default 5, range 1–10
- **State Transitions**:
  - `enabled` toggled via CLI `sources add/remove`
  - `last_synced_at` updated after successful fetch

### CacheEntry
- **Primary Identifier**: Composite (`marketplace_source`, `payload_type`)
- **Key Fields**: `payload_hash`, `stored_at`, `expires_at`, `serialization_format`
- **Relationships**: Linked to `MarketplaceSource` for data provenance
- **Validation Rules**:
  - `expires_at` = `stored_at` + TTL (24h default or user override)
  - `payload_hash` verified against manifest checksums
- **State Transitions**:
  - Fresh → Stale when `now >= expires_at`
  - Stale → Refreshing when queued for background refresh
  - Refreshing → Fresh on successful fetch; remains Stale on failure

### RefreshJob
- **Primary Identifier**: `job_id` (`Uuid`)
- **Key Fields**: `source_name`, `created_at`, `next_attempt_at`, `status`, `error_context`
- **Relationships**: References `MarketplaceSource` and optionally `TelemetryEvent` (if triggered by search)
- **Validation Rules**:
  - `status` enum: `Queued`, `Running`, `Deferred`, `Completed`, `Failed`
  - `next_attempt_at` MUST be ≥ `created_at`
- **State Transitions**:
  - `Queued` → `Running` when worker picks up job
  - `Running` → `Completed` on success
  - `Running` → `Deferred` when rate limited; `next_attempt_at` set to API reset time
  - `Deferred` → `Running` when window opens
  - Any state → `Failed` after retry budget exhausted (surface warning)

### Preferences
- **Primary Identifier**: single record per user (`preferences.json`)
- **Key Fields**: `default_ttl_hours`, `enabled_sources: Vec<String>`, `metrics_opt_in`
- **Relationships**: Tracks enabled `MarketplaceSource` names and influences caching rules
- **Validation Rules**:
  - `default_ttl_hours` range 1–168
  - `enabled_sources` MUST reference existing sources during load

### TelemetryEvent
- **Primary Identifier**: auto-increment or UUID in memory (persisted across runs unless the user opts out)
- **Key Fields**: `event_type` (`Search`, `List`, `Show`), `timestamp`, `payload`
- **Relationships**: Aggregated per release for SC-005; linked to `Extension` when event references a specific namespace
- **Validation Rules**:
  - `payload` sanitized to exclude PII
  - Persisted only when the user has not opted out of telemetry collection

## Derived Relationships & Aggregations

- `MarketplaceSource` 1—N `Extension`
- `MarketplaceSource` 1—N `CacheEntry`
- `RefreshJob` triggered per `MarketplaceSource` (optional per `Extension` detail view)
- Telemetry aggregation produces `top_search_terms` dataset each release cycle, bounded to 50 entries

## Data Lifecycle

1. Source fetch populates `CacheEntry` (Fresh) and updates `Extension` records.
2. TTL expiration marks entries Stale; background refresh enqueues `RefreshJob`.
3. Refresh success updates cache and `last_synced_at`; failure keeps stale data with warnings.
4. Preferences updates propagate immediate effect on TTL defaults and enabled sources.
5. Telemetry counters reset each release cycle after success criteria evaluation.
