# Data Model — Gemini Marketplace Extension

## Extension

| Field | Type | Required | Description | Constraints |
| --- | --- | --- | --- | --- |
| `id` | string | Yes | Namespaced identifier (`<source>/<extension>`) | Unique across all sources; slug safe |
| `name` | string | Yes | Extension's own name from manifest | Original casing preserved |
| `description` | string | Yes | Short summary for listings | Trimmed to CLI width, markdown stripped for tables |
| `repository_url` | string (URL) | Yes | Install URL surfaced to users | Must be HTTPS git URL |
| `version` | string | Yes | Semantic version reported by manifest | Valid SemVer; parsed for comparisons |
| `author` | string | Yes | Maintainer or publisher | Free-form |
| `source` | string | Yes | Marketplace source identifier | FK → `MarketplaceSource.name` |
| `categories` | array\<string> | No | Tags/categories used for filtering | Case-insensitive; deduplicated |
| `compatibility` | array\<string> | No | Gemini CLI or platform compatibility notes | Present when manifest declares constraints |
| `install_status` | enum | Yes | Installation detection result | `installed` / `not_installed` / `unknown` |
| `last_seen` | datetime | Yes | Timestamp from latest sync batch | UTC; updates on each successful fetch |
| `validation_summary` | object | Yes | Schema + semantic validation outcome | See ManifestValidation |
| `manifest_path` | string | Yes | Path within source repo (for monorepos) | Relative path; `.` for repo root |
| `readme_excerpt` | string | No | Cached README/usage snippet | Markdown preserved for detailed view |

**Identity & Relationships**
- Primary key: `id`.
- Relationship: `Extension` belongs to `MarketplaceSource`.
- Validation summary references `ManifestValidation` structure.

**Lifecycle**
1. `discovered` – during sync when manifest passes schema checks.
2. `stale` – TTL exceeded but still cached; flagged for refresh.
3. `archived` – source removal or manifest missing; retained until cache purge.

## ManifestValidation

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `schema_status` | enum | Yes | `passed` / `warning` / `failed` |
| `semantic_status` | enum | Yes | `pending` / `passed` / `failed` |
| `errors` | array\<ValidationError> | No | Detailed validation issues |
| `last_checked` | datetime | Yes | Timestamp of most recent semantic validation |

### ValidationError

| Field | Type | Description |
| --- | --- | --- |
| `code` | string | Stable identifier (e.g., `invalid_semver`) |
| `message` | string | User-facing explanation |
| `path` | string | JSON pointer to offending field |

## MarketplaceSource

| Field | Type | Required | Description | Constraints |
| --- | --- | --- | --- | --- |
| `name` | string | Yes | Human-friendly source key | Unique; kebab-case recommendation |
| `url` | string (URL/path) | Yes | GitHub repo, git URL, or local path | Validated before add |
| `type` | enum | Yes | `github_repo` / `git` / `local_path` | Derived from URL scheme |
| `enabled` | bool | Yes | Source active flag | Default `true` |
| `added_at` | datetime | Yes | When user configured source | UTC |
| `last_synced_at` | datetime | No | Last successful sync | Null until first sync |
| `recursion_depth` | int | Yes | Effective scan depth | Default 5; >=1 |
| `etag` | string | No | Optional cache token for conditional requests | Mirrors upstream headers |
| `sync_state` | enum | Yes | `idle` / `fetching` / `queued_retry` / `rate_limited` | Drives CLI status reporting |

**Relationships**
- `MarketplaceSource` has many `Extension`.
- `sync_state` transitions to `queued_retry` when errors occur and `rate_limited` while countdown active.

## CacheEntry

| Field | Type | Required | Description | Constraints |
| --- | --- | --- | --- | --- |
| `source` | string | Yes | Source identifier | FK → `MarketplaceSource.name` |
| `batch_index` | int | Yes | Zero-based index for 500-extension chunks | Sequential with no gaps |
| `fetched_at` | datetime | Yes | Retrieval time | UTC |
| `expires_at` | datetime | Yes | TTL deadline | `expires_at = fetched_at + ttl` |
| `extension_ids` | array\<string> | Yes | Extensions contained in batch | IDs must exist in `Extension` |
| `checksum` | string | No | Hash of serialized payload | Used to detect drift |

**Lifecycle**
- `fresh` → `stale` when current time > `expires_at`.
- `stale` → `refreshing` when manual refresh or background retry triggers; resets on completion.
- Purged when source removed or TTL exceeded + refresh failure beyond retention window.

## RetryJob

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `job_id` | string | Yes | Deterministic key (source + batch) |
| `source` | string | Yes | Associated source |
| `scheduled_for` | datetime | Yes | Next retry time |
| `attempt` | int | Yes | Retry count |
| `reason` | string | Yes | Error summary or rate-limit note |

**State Transitions**
- `scheduled` → `in_progress` when worker executes.
- `in_progress` → `completed` on success (job removed).
- `in_progress` → `scheduled` with backoff when failure persists.

## RateLimitWindow

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `source` | string | Yes | Source experiencing rate limiting |
| `resets_at` | datetime | Yes | API-provided reset timestamp |
| `remaining` | int | No | Remaining tokens if provided |

**Usage**
- Drives CLI countdown display.
- Cleared when reset time reached or manual override invoked.
