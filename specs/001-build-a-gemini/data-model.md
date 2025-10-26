# Phase 1 Data Model – Gemini CLI Extension Marketplace

## Entities

### Extension
- **id**: `String` (format `source-slug/extension-slug`)
- **source_slug**: `String`
- **extension_slug**: `String`
- **base_name**: `String`
- **description**: `String`
- **repository_url**: `Url`
- **version**: `semver::Version`
- **author**: `String`
- **categories**: `Vec<String>`
- **tags**: `Vec<String>`
- **readme_excerpt**: `Option<String>`
- **compatibility**: `Vec<String>` (CLI versions/features)
- **install_status**: enum `{ Installed, NotInstalled, Unknown }`
- **warnings**: `Vec<Warning>` (schema skip, validation issues, missing metadata)
- **manifest_path**: `PathBuf`
- **last_seen**: `DateTime<Utc>`
- **cache_freshness**: enum `{ Fresh, Stale, Expired }`

**Relationships**
- Belongs to one `MarketplaceSource`.
- Has many `ValidationIssue` records surfaced during detail lookup.

**Validation Rules**
- `repository_url` MUST be https.
- `version` MUST parse as semantic version.
- `description` trimmed length <= 500 chars for list view; full stored for detail.
- `readme_excerpt` sanitized to plain text.

### MarketplaceSource
- **slug**: `String` (unique, kebab-case)
- **display_name**: `String`
- **source_type**: enum `{ GitHubRepo, GitUrl, LocalPath }`
- **url**: `Url`
- **enabled**: `bool`
- **requires_auth**: `bool`
- **default_recursion_depth**: `u8`
- **last_synced_at**: `Option<DateTime<Utc>>`
- **sync_status**: enum `{ Idle, Syncing, Error }`
- **error_message**: `Option<String>`
- **extensions_count**: `usize`

**Validation Rules**
- `slug` MUST be unique and immutable after creation.
- `url` MUST be reachable (HEAD/GET) during add flow.
- `requires_auth` sources trust external credential helpers.

### CacheManifest
- **source_slug**: `String`
- **batch_size**: `usize` (default 500)
- **etag**: `Option<String>`
- **last_fetched**: `DateTime<Utc>`
- **ttl_hours**: `u32`
- **stale_after**: `DateTime<Utc>`
- **extensions**: `Vec<ExtensionSummary>`
- **queued_jobs**: `Vec<RefreshJobSummary>`

**Validation Rules**
- `ttl_hours` MUST match user preferences.
- `last_fetched + ttl_hours` defines `stale_after`.
- Stored under `$GEMINI_CONFIG/extensions/marketplace/{source}/manifest.json`.

### RefreshJob
- **job_id**: `Uuid`
- **source_slug**: `String`
- **requested_at**: `DateTime<Utc>`
- **status**: enum `{ Pending, Running, DeferredRateLimit, Completed, Failed }`
- **retry_after**: `Option<DateTime<Utc>>`
- **attempts**: `u8`
- **last_error**: `Option<String>`

**State Transitions**
- `Pending → Running`: worker dequeues job.
- `Running → DeferredRateLimit`: GitHub 429 encountered; sets `retry_after`.
- `Running → Completed`: successful fetch + cache write.
- `Running → Failed`: irrecoverable error (bad manifest, auth failure).
- `DeferredRateLimit → Pending`: when countdown expires.
- `Failed → Pending`: manual retry via `cache refresh`.

### UserPreferences
- **cache_ttl_hours**: `u32` (default 24)
- **pre_fetch_search**: `bool`
- **max_recursion_depth**: `u8` (default 5)
- **json_output_default**: `bool`
- **telemetry_opt_in**: `bool`

Stored as `config/preferences.json` beneath `$GEMINI_CONFIG/extensions/marketplace/`.

## Supporting Types

### Warning
- **kind**: enum `{ MissingManifest, SchemaInvalid, SemanticError, SourceUnavailable }`
- **message**: `String`
- **timestamp**: `DateTime<Utc>`

### ValidationIssue
- **field**: `String`
- **severity**: enum `{ Error, Warning }`
- **detail**: `String`

### ExtensionSummary
- **id**: `String`
- **name**: `String`
- **description**: `String`
- **categories**: `Vec<String>`
- **version**: `String`
- **install_status**: enum
- **warnings_present**: `bool`

Used by `CacheManifest` to avoid storing full detail data for listing.

## Derived Views

- **CatalogView**: Aggregates `ExtensionSummary` from all enabled sources, annotated with
  installation status and cache freshness, sorted by name.
- **DetailView**: Merges `Extension`, `ValidationIssue`, and README excerpt when user runs
  `show`.
- **SourceStatusView**: Combines `MarketplaceSource`, `CacheManifest`, and `RefreshJob` for
  `sources list` and observability metrics.
