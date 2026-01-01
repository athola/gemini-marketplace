# Data Model – Gemini CLI Extension Marketplace

## Entities

### Extension

| Field | Type | Description | Validation |
| --- | --- | --- | --- |
| `namespace` | string | `source-alias/extension-name` identifier. | Must be lowercase, unique, and match `[a-z0-9-]+/[a-z0-9-]+`. |
| `name` | string | Human-friendly name from the manifest. | 1–80 characters. |
| `description` | string | A summary that is surfaced in the list and detail views. | ≤ 512 characters; sanitized markdown. |
| `version` | semver string | The declared extension version. | Must be a valid semver and match the manifest checksum tuple. |
| `author` | string | The publisher label. | Required. |
| `categories` | array<string> | Tags used for filtering. | Each entry must be ≤ 32 characters. |
| `repository_url` | URL | The Git/GitHub location for manual installation. | HTTPS required. |
| `readme` | markdown blob | An optional README snippet. | Rendered on the `show` command. |
| `install_status` | enum | The derived installation status. | `installed`, `not_installed`, or `unknown`. |
| `source_alias` | string | The alias chosen during `sources add`. | References `MarketplaceSource.alias`. |
| `warnings` | array<string> | Validation or fetch warnings. | Each entry must be ≤ 120 characters. |
| `cache_freshness` | ISO timestamp | The last refresh timestamp. | Required. |

### MarketplaceSource

| Field | Type | Description | Validation |
| --- | --- | --- | --- |
| `alias` | string | A user-facing identifier. | Unique, `[a-z0-9-]+`. |
| `url` | URL | The Git repo or catalog root. | HTTPS or SSH git URL. |
| `type` | enum | The source type for the ingestion pipeline. | `github_repo`, `git_repo`, or `local_dir`. |
| `recursion_depth` | int | The maximum directory depth for manifest discovery. | Default 5; 1–10. |
| `enabled` | bool | Whether the source participates in refresh/list. | Default true. |
| `last_sync_at` | ISO timestamp | The last successful refresh per source. | Optional. |
| `error_state` | object | Stores the last failure reason and retry ETA. | Optional. |

### ManifestCacheEntry

| Field | Type | Description | Validation |
| --- | --- | --- | --- |
| `namespace` | string | A foreign key to `Extension.namespace`. | Required. |
| `stored_at` | ISO timestamp | When the manifest was cached. | Required. |
| `ttl_hours` | int | The TTL chosen by the user. | Default 24; 1–168. |
| `checksum` | sha256 | The digest of the manifest payload. | Required for trust. |
| `schema_valid` | bool | The result of the schema validation. | Required. |
| `semantic_valid` | bool | The result of the semantic validation. | Required. |
| `metadata` | json | The raw manifest blob for detail rendering. | |

### TelemetryRecord

| Field | Type | Description | Validation |
| --- | --- | --- | --- |
| `timestamp` | ISO timestamp | When the event was emitted. | Required. |
| `event_type` | enum | Observability events. | `cache_hit`, `cache_miss`, `rate_limit_wait`, `search`, `source_add`, or `source_remove`. |
| `attributes` | map | Structured metrics (e.g., `duration_ms`, `source_alias`, `keyword`). | Sensitive data is redacted. |

## Relationships

-   `MarketplaceSource.alias` to `Extension.source_alias` is a one-to-many relationship. Removing a source hides its dependent extensions.
-   `Extension.namespace` to `ManifestCacheEntry.namespace` is a one-to-one relationship. The cache entry holds the raw metadata for the rendered extension.
-   `TelemetryRecord` references `source_alias` and `namespace` via attributes but does not hold strong foreign key constraints.

## State Transitions

1.  **Source Lifecycle**:
    -   `unregistered` → `registered`: When a user runs `sources add`.
    -   `registered` → `syncing`: During fetch and validation.
    -   `syncing` → `healthy` or `degraded`: On success or failure, respectively.
2.  **Extension Lifecycle**:
    -   `discovered` → `validated`: After semantic checks pass.
    -   `validated` → `stale`: When the TTL expires.
    -   `stale` → `refreshed`: After a background or manual refresh.
3.  **ManifestCacheEntry**:
    -   On a checksum mismatch or validation failure, the entry transitions to `invalid` and is removed from the list.

## Validation & Rules

-   Namespaces are immutable once an extension is cached.
-   The recursion depth guard prevents runaway scans.
-   Cache entries that are older than their TTL must not influence derived telemetry without a `stale` flag.
-   Telemetry events omit PII and aggregate counts to respect the opt-out policy.
