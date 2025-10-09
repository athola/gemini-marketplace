# Data Model — Gemini CLI Extension Marketplace

## Overview
The marketplace extension persists a small set of structured records to present catalog data, manage sources, and honor caching + rate-limit requirements. All records are stored locally (JSON files) within the Gemini CLI configuration directory.

## Entities

### Extension
- **Identifier**: `String` — Namespaced identifier formatted as `<source_slug>/<extension_slug>` (primary key)
- **Fields**:
  - `source_slug: String` — References the owning `MarketplaceSource.slug`
  - `extension_slug: String` — Slugified manifest `name`
  - `display_name: String`
  - `summary: String`
  - `repository_url: Url`
  - `homepage_url: Option<Url>`
  - `documentation_url: Option<Url>`
  - `version: Version` — SemVer parsed from manifest
  - `author: String`
  - `license: Option<String>`
  - `categories: Vec<String>`
  - `tags: Vec<String>`
  - `compatibility: Vec<String>` — Gemini CLI versions or capability strings
  - `install_status: InstallStatus`
  - `manifest_checksum: String` — SHA256 of the retrieved `gemini-extension.json`
  - `readme_excerpt: Option<String>`
  - `last_synced_at: DateTime`
  - `cache_expires_at: DateTime`
- **Relationships**:
  - Belongs to one `MarketplaceSource`
  - May reference zero or more `CacheEntry` artifacts via `manifest_checksum`
- **Validation Rules**:
  - `display_name`, `repository_url`, `version`, and `author` MUST be present
  - `repository_url` MUST match the source host protocol (e.g., GitHub HTTPS)
  - `version` MUST obey SemVer format
  - `categories` values MUST be normalized to kebab-case
  - `install_status` transitions follow the state machine below
- **State Transitions** (`InstallStatus`):
  - `NotInstalled` → `Installed` when registry/file check confirms presence
  - `Installed` → `UpdateAvailable` when manifest version > installed version
  - `Installed` → `NotInstalled` when uninstall detected
  - `UpdateAvailable` → `Installed` after manual upgrade verification

### MarketplaceSource
- **Identifier**: `slug: String` (primary key)
- **Fields**:
  - `display_name: String`
  - `url: Url` — GitHub repository or git URL
  - `source_type: SourceType` (`GithubRepo` | `GitUrl` | `LocalPath`)
  - `default: bool`
  - `enabled: bool`
  - `requires_auth: bool`
  - `last_synced_at: Option<DateTime>`
  - `last_sync_status: SyncStatus`
  - `etag: Option<String>`
  - `poll_interval: Duration` — Optional per-source override (default 24h)
- **Relationships**:
  - Owns many `Extension` records
  - Links to zero or one `RateLimitWindow` (for GitHub-backed sources)
- **Validation Rules**:
  - `slug` MUST be unique and kebab-case
  - `url` MUST be reachable via git clone or HTTP fetch during validation
  - `requires_auth` MUST be true when the URL is private or non-public host
  - Disabling a source MUST cascade to mark its extensions as `Stale`
- **State Transitions** (`SyncStatus`):
  - `Idle` → `Syncing` when refresh triggered
  - `Syncing` → `Healthy` when data validated successfully
  - `Syncing` → `Warning` when recoverable issues (missing manifests) found
  - `Syncing` → `Error` when source unreachable; resolves to `Idle` after manual acknowledgment

### CacheEntry
- **Identifier**: Composite (`source_slug`, `manifest_checksum`)
- **Fields**:
  - `source_slug: String`
  - `payload_path: PathBuf`
  - `fetched_at: DateTime`
  - `expires_at: DateTime`
  - `extension_ids: Vec<String>` — References of extensions covered by this payload
  - `etag: Option<String>`
- **Relationships**:
  - Maps to many `Extension` records via `extension_ids`
- **Validation Rules**:
  - `expires_at` MUST equal `fetched_at + TTL`
  - Deleting a cache entry MUST clear `cache_expires_at` on associated extensions

### RateLimitWindow
- **Identifier**: `source_slug: String`
- **Fields**:
  - `source_slug: String`
  - `reset_at: Option<DateTime>`
  - `remaining_requests: Option<u32>`
  - `limit: Option<u32>`
- **Relationships**:
  - One-to-one with `MarketplaceSource` (GitHub-hosted sources only)
- **Validation Rules**:
  - `remaining_requests` MUST NOT exceed `limit`
  - `reset_at` MUST be in the future when `remaining_requests == 0`

### UserPreferences
- **Identifier**: Singleton stored alongside configuration
- **Fields**:
  - `cache_ttl_hours: u16` — Default 24
  - `auto_refresh_on_launch: bool`
  - `search_mode: SearchMode` (`LocalFilter` | `PreFetchFilter`)
  - `output_format: OutputFormat` (`Table` | `Json`)
- **Validation Rules**:
  - `cache_ttl_hours` MUST be between 1 and 168
  - `search_mode` defaults to `LocalFilter` when unspecified

## Derived Views
- **ExtensionListView**: Denormalized projection combining `Extension`, `MarketplaceSource`, and `RateLimitWindow` reminder for display.
- **SourceStatusView**: Presents `MarketplaceSource` with sync + rate-limit metadata for CLI status commands.

## Data Lifecycle
1. **Ingestion**: Sources are validated, manifests (`gemini-extension.json`) fetched, and new `Extension` records materialized.
2. **Caching**: Each sync writes/updates `CacheEntry` JSON files; stale entries pruned during refresh.
3. **Display**: CLI queries projection views sorted by category/tag, enriched with `InstallStatus`.
4. **Invalidation**: Manual refresh or expired TTL purges relevant cache entries and triggers re-fetch.
5. **Removal**: Disabling a source removes its extensions after user confirmation, leaving audit trail in logs.
