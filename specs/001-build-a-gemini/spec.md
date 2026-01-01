# Feature Specification: Gemini CLI Extension Marketplace

**Feature Branch**: `001-build-a-gemini`
**Created**: 2025-10-09
**Status**: Draft
Input: User requested a Rust-based Gemini CLI extension to address the current lack of a centralized marketplace for Gemini CLI extensions. Currently, extensions are installed directly from GitHub repositories (e.g., BigQuery Data Analytics, Conversational Analytics). Google plans to streamline this with simpler installation commands and a future 'clearinghouse' for ratified extensions. This project aims to create a marketplace-like experience, similar to Claude Code's `/plugin marketplace`, allowing users to list and browse third-party extensions. This is distinct from IDE-specific marketplaces like Visual Studio Marketplace (e.g., 'Gemini Code Assist').

## Clarifications

### Architecture

-   **Workspace & MCP Architecture**: The marketplace is a Cargo workspace that ships three deliverables:
    1.  `marketplace-core`: The existing shared library and CLI logic.
    2.  `marketplace-mcp-server`: A dedicated MCP stdio server binary that Gemini invokes during extension installation and at runtime.
    3.  `marketplace-mcp-cli`: A lightweight test CLI for developers that automatically spawns and communicates with the MCP server to validate flows without Gemini.
-   **Default Curated Source**: The default curated marketplace source at launch will be `https://github.com/athola/gemini-marketplace`, which is maintained under our control for stable test data.

### Data Handling

-   **Extension Naming and Conflict Resolution**: To prevent naming conflicts, extensions are identified using a `source-name/extension-name` namespace.
-   **Installation Detection**: The system uses a best-effort approach to detect installed Gemini CLI extensions, first checking the registry and then performing a file system scan.
-   **GitHub API Rate Limiting**: When encountering GitHub API rate limits, the system queues requests and defers refreshes until the rate limit resets, displaying a countdown timer to the user.
-   **Cache Validity and Refresh**: Cached marketplace data has a user-configurable Time-To-Live (TTL), with a recommended default of 24 hours.
-   **Metadata Manifest**: The marketplace parses `gemini-extension.json` from each extension repository for details and validation. If a root manifest is missing or fails validation, the extension is skipped, processing continues, and a warning is surfaced.
-   **Private Source Authentication**: For custom sources that require private Git access, the marketplace relies on existing Git credential helpers or environment variables and does not store credentials.
-   **Progressive Validation**: Validation of `gemini-extension.json` manifests is progressive: a basic schema check on fetch, followed by full semantic validation when the user views extension details.
-   **Scalability for Catalogs**: To efficiently handle large extension catalogs, the system lazy-loads extensions in 500-item increments.
-   **Monorepo Discovery**: For monorepos that contain multiple extensions, the system recursively searches for `gemini-extension.json` files in subdirectories, with a configurable recursion limit, to discover individual extensions.
-   **Network Failure Retry Strategy**: In cases of network errors, timeouts, or invalid responses during data fetching, the system uses a background retry strategy, queuing failed requests while continuing to serve cached data.
-   **Monorepo Scan Depth**: To balance discovering deeply nested extensions and preventing excessive traversal, a default maximum recursion depth of 5 levels is enforced, with user configurability.

### User Experience

-   **CLI Interaction Model**: The marketplace extension list in the CLI uses paginated output with navigation commands (next/prev) for an improved user experience.
-   **Default List Behavior**: `gemini marketplace list` defaults to stateless execution, requiring an explicit `--interactive` flag for interactive pagination prompts. This design prioritizes scriptability and predictability for automated workflows.
-   **Command Structure**: The marketplace extension exposes a top-level `gemini marketplace` command with a clear subcommand structure: `list`, `show <id>`, `search`, `sources add/list/remove`, `cache refresh`, and `cache ttl set <hours>`.
-   **Source Aliasing**: When adding a custom marketplace source, the user is prompted for a human-friendly alias, which defaults to the sanitized repository slug (e.g., `org-repo`).

### Search

-   **Search and Filter Behavior**: Both local filtering (after fetching all extensions) and optional pre-fetch filtering are supported for keyword searches.
-   **Search Relevance Measurement**: "Common keywords" are defined by using live telemetry to identify the top searched terms each release cycle. Search relevance is then validated against this rotating list.

## User Scenarios & Testing

### User Story 1: Browse Available Extensions (P1)

As a Gemini CLI user, I want to discover available extensions to enhance my CLI experience without manually searching GitHub repositories.

**Acceptance Scenarios**:

1.  **Given** I am a Gemini CLI user, **When** I run `gemini marketplace list`, **Then** I see a paginated list of available extensions with their names, descriptions, and source repositories.
2.  **Given** the extension list is displayed, **When** I view the list, **Then** I can see key metadata for each extension, including version information and installation status.
3.  **Given** there are multiple extensions available, **When** I browse the list, **Then** each row displays the columns `namespace`, `name`, `version`, `categories`, `install status`, `source`, `warnings`, and `cache freshness`. Category values are rendered as badges inline with the row.
4.  **Given** the extension list contains more items than fit on one page, **When** I launch `gemini marketplace list --interactive` and use the navigation commands (next/prev), **Then** I can navigate forward and backward through pages of results without re-invoking the command.
5.  **Given** a marketplace source contains thousands of extensions, **When** I browse the list, **Then** the system lazy-loads extensions in 500-extension increments, providing responsive performance without fetching the entire catalog upfront.

---

### User Story 2: View Extension Details (P2)

As a Gemini CLI user, I want to view detailed information about a specific extension to make an informed decision about whether to install it.

**Acceptance Scenarios**:

1.  **Given** I am viewing the extension list, **When** I run `gemini marketplace show <source-name/extension-name>`, **Then** I see its description, author, repository URL, version, and compatibility information.
2.  **Given** I am viewing extension details, **When** the extension has documentation or README content, **Then** I can access that documentation directly.
3.  **Given** I am viewing extension details, **When** I want to install the extension, **Then** I can see clear installation instructions with the exact GitHub URL to use.
4.  **Given** I am viewing extension details, **When** the system performs full semantic validation, **Then** any manifest validation errors are displayed clearly.

---

### User Story 3: Search and Filter Extensions (P3)

As a Gemini CLI user, I want to search for extensions by keyword or filter by category to quickly find relevant extensions.

**Acceptance Scenarios**:

1.  **Given** I am viewing the marketplace, **When** I run `gemini marketplace search <keyword>`, **Then** the extension list filters to show only extensions that match the term in their name or description.
2.  **Given** extensions have categories or tags, **When** I run `gemini marketplace search --category <tag>`, **Then** only extensions in that category are displayed.
3.  **Given** I have applied filters, **When** I run `gemini marketplace list` without search parameters, **Then** the full extension list is displayed again.

---

### User Story 4: Manage Marketplace Sources (P4)

As a Gemini CLI user, I want to add custom marketplace sources to access organization-specific or private extension repositories.

**Acceptance Scenarios**:

1.  **Given** I want to add a custom marketplace source, **When** I run `gemini marketplace sources add <url>`, **Then** that source is added to my marketplace configuration.
2.  **Given** I have multiple marketplace sources configured, **When** I browse extensions, **Then** I can see which source each extension comes from.
3.  **Given** I have added custom sources, **When** I run `gemini marketplace sources remove <source-name>`, **Then** that source is removed, and extensions from that source no longer appear.
4.  **Given** I add a marketplace source that is structured as a monorepo with multiple extensions, **When** the system scans the repository, **Then** it recursively discovers all `gemini-extension.json` manifests in subdirectories (up to the recursion limit) and treats each as an independent extension.

---

### Edge Cases

-   **Unreachable or invalid sources**: Must surface a warning, preserve cached listings, and queue the request for a background retry.
-   **Network errors**: Must trigger a background retry, with the status visible to the user, while serving cached data.
-   **Missing or invalid metadata**: Must stop the command with a clear error that references the offending source and manifest path.
-   **Semantic validation failures**: Must stop the command with a clear error and diagnostics so the user can correct the source before retrying.
-   **Deleted or moved repositories**: Must be marked as unavailable, retaining cached metadata until expiration.
-   **Offline or no-network scenarios**: Must display cached data with a staleness warning and queue a refresh for when connectivity resumes.
-   **Metadata revisions**: Must be validated progressively (schema on fetch, semantics on detail view), with incompatibilities flagged.
-   **Recursive scanning depth**: Must enforce a configurable depth limit (default: 5) to prevent excessive traversal and log a warning if the limit is reached.
-   **Monorepo discovery**: Each discovered `gemini-extension.json` in a monorepo must be treated as an independent, namespaced extension.
-   **Checksum mismatches**: Must halt the command with guidance to refresh or inspect the source before retrying.


## Requirements

### Functional Requirements

-   **FR-001**: Must retrieve and display a curated list of extensions from configured marketplace sources.
    -   **FR-001a**: Must present extension lists using paginated output with navigation commands (next/prev).
    -   **FR-001b**: `gemini marketplace list` must execute as a single-shot command by default; an `--interactive` flag enables an interactive prompt for navigation.
-   **FR-002**: Must display extension metadata, including name, description, repository URL, version, and author.
    -   **FR-002a**: Must namespace extension identifiers as `source-name/extension-name` to handle naming conflicts.
    -   **FR-002b**: The `gemini marketplace list` output must include `namespace`, `name`, `version`, `categories`, `install_status`, `source`, `warnings`, and `cache_freshness`.
-   **FR-003**: Must allow users to view detailed information for any listed extension.
-   **FR-004**: Must provide the GitHub repository URL for each extension to enable manual installation.
-   **FR-005**: Must support at least one default marketplace source.
    -   **FR-005a**: The default curated marketplace source must be `https://github.com/athola/gemini-marketplace` and must be enabled by default.
-   **FR-006**: Must support keyword search in the name or description, with two modes: local filtering after fetching all extensions, or optional pre-filtering of API requests.
-   **FR-007**: Must allow users to add custom marketplace sources via GitHub repository or git URLs.
    -   **FR-007a**: `gemini marketplace sources add` must prompt for a source alias, defaulting to the sanitized repository slug if the user accepts the default.
-   **FR-008**: Must allow users to list all configured marketplace sources.
-   **FR-009**: Must allow users to remove previously added marketplace sources.
-   **FR-010**: Must cache marketplace data locally with a configurable TTL (default: 24 hours).
    -   **FR-010a**: Must allow users to configure the cache TTL.
    -   **FR-010b**: Must implement lazy loading, fetching and caching data in increments of 500 extensions.
    -   **FR-010c**: Must treat cached records as stale when `stored_at + TTL <= now`, label them as such, and enqueue a background refresh.
-   **FR-011**: Must provide a mechanism to manually refresh marketplace data, bypassing the cache.
-   **FR-012**: Must handle network errors gracefully by queuing failed requests for background retry and serving cached data.
    -   **FR-012a**: Must handle GitHub API rate limiting by queuing refresh requests until the rate limit resets and displaying a countdown timer.
    -   **FR-012b**: Must implement a background retry for failed marketplace data fetches.
-   **FR-013**: Must validate marketplace source URLs and metadata format.
    -   **FR-013a**: Must discover extensions by recursively scanning for `gemini-extension.json` files (up to a configurable depth, default: 5) and perform progressive validation (basic schema on fetch, full semantic on detail view).
    -   **FR-013b**: Must halt command execution when encountering invalid or missing `gemini-extension.json` manifests, emitting actionable error diagnostics.
    -   **FR-013c**: Must display full semantic validation errors on the extension detail view.
    -   **FR-013d**: Must rely on existing Git credential helpers for authenticating private sources and must not store credentials.
    -   **FR-013e**: Must allow users to configure the maximum recursion depth for monorepo scanning.
    -   **FR-013f**: Must verify manifest checksums and semantic version metadata using `sha2` and `semver`; if verification fails, abort with an error and do not cache the entry.
-   **FR-014**: Must distinguish between installed and not-installed extensions by checking the Gemini CLI extension registry first, then the file system.
    -   **FR-014a**: Must persist preferences, sources, and cache configuration in `$GEMINI_CONFIG/extensions/marketplace/preferences.json`.
-   **FR-015**: Must support filtering extensions by category or tags.
-   **FR-016**: Must expose a `gemini marketplace` command with the following subcommands: `list`, `show <id>`, `search`, `sources add/list/remove`, `cache refresh`, and `cache ttl set <hours>`.
-   **FR-017**: Must ship as a Cargo workspace that includes (a) the existing CLI/library crate for reuse, (b) an MCP stdio server binary that Gemini can launch independently of the CLI, and (c) a developer-only MCP test CLI that auto-spawns and communicates with the same server to mirror Gemini behavior.
### Non-Functional Requirements

-   **NFR-001**: Provides dual-mode logging (human-readable and JSON) and structured metrics for cache usage, rate-limit delays, refresh queue depth, and top searched terms.
-   **NFR-002**: Handles large extension catalogs efficiently by implementing lazy loading in 500-extension increments.
-   **NFR-003**: Telemetry collection is opt-out by default and stores aggregated, non-identifying data.

### Key Entities

-   **Extension**: A Gemini CLI extension with a namespaced unique identifier (`source-name/extension-name`), name, description, repository URL, version, author, source, categories, compatibility, and installation status.
-   **Marketplace Source**: A repository or catalog of extensions with a name, URL, type (GitHub, git, local), last updated timestamp, and enabled/disabled status.
-   **Extension Metadata**: Information about an extension from its `gemini-extension.json` manifest, including the README, installation instructions, and dependencies.

## Success Criteria

-   **SC-001**: Users can discover extensions without leaving the CLI.
-   **SC-002**: Users can view extension details and get installation instructions in under 30 seconds.
-   **SC-003**: The marketplace displays extension lists within 2 seconds when using cached data.
-   **SC-004**: Users can add custom sources and see their extensions within 5 seconds.
-   **SC-005**: Search returns relevant results for at least 90% of the top searched terms.
-   **SC-006**: The system handles network failures gracefully, serving cached data while retrying in the background.
-   **SC-007**: Users can configure the cache expiration.

## Assumptions

-   Extension metadata is defined by `gemini-extension.json` manifests.
-   The default marketplace source is a GitHub repository.
-   Extensions are installed via GitHub URLs.
-   Users have network connectivity for initial data retrieval but can browse cached data offline.
-   Extensions follow semantic versioning.
-   The marketplace extension is installed using the standard Gemini CLI method.
-   The default 24-hour cache TTL is a reasonable balance but is configurable.
-   Marketplace catalogs may contain thousands of extensions, requiring lazy loading.
-   Marketplace sources may be monorepos, requiring recursive scanning.
-   Telemetry is opt-out by default and stores aggregated, non-identifying data.
-   Preferences, cache, and source configurations are stored in `$GEMINI_CONFIG/extensions/marketplace/`.

## Dependencies

-   Access to the GitHub API or raw GitHub content.
-   A supported Gemini CLI extension architecture.
-   An established extension metadata standard.
-   Access to the Gemini CLI extension registry or configuration for installation status.
-   Knowledge of Gemini CLI extension installation directories for secondary checks.
-   Availability of Git credential helpers for private sources.
-   HTTP client and serialization crates (`reqwest`, `serde`/`serde_json`) for fetching and decoding marketplace manifests.

## Out of Scope

-   Automated extension installation.
-   Extension verification, security scanning, or safety guarantees.
-   Extension ratings, reviews, or feedback mechanisms.
-   Extension dependency resolution or compatibility checking beyond displaying metadata.
-   Extension update notifications or version management.
-   Creating or publishing extensions.
-   Hosting marketplace infrastructure.
