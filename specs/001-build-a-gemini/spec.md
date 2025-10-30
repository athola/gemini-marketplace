# Feature Specification: Gemini CLI Extension Marketplace

**Feature Branch**: `001-build-a-gemini`
**Created**: 2025-10-09
**Status**: Draft
**Input**: User description: "build a gemini CLI extension with Rust using the reference here: https://blog.google/technology/developers/gemini-cli-extensions/. the caveat is that gemini CLI extensions does not expose a plugin marketplace similar to Claude Code. refer to this: The Gemini CLI, an open-source AI agent, supports extensions to enhance its functionality. While there isn't a centralized "marketplace" in the traditional sense for these extensions currently, the method for installing them involves directly referencing their GitHub repositories. Google has expressed plans to streamline this process in the future, aiming for a simpler, one-line command installation experience for extensions, potentially through a command like Gemini extensions install <GitHub_URL>. Additionally, there is anticipation of a future "clearinghouse or repository" where Google-ratified extensions will be readily available for easy installation. For now, users typically install Gemini CLI extensions by obtaining them from their respective GitHub repositories. For example, extensions for BigQuery Data Analytics and Conversational Analytics are available as part of the Gemini CLI ecosystem. It is important to distinguish this from the Visual Studio Marketplace or other IDE-specific marketplaces, where extensions like "Gemini Code Assist" are available for integration within those development environments. While these offer Gemini-related functionality, they are distinct from the extensions designed specifically for the Gemini CLI itself.. we want to add an extension that lets us to list and peruse 3rd party exensions similar to how Claude Code implements `/plugin marketplace`: https://docs.claude.com/en/docs/claude-code/plugin-marketplaces"

## Clarifications

### Session 2025-10-09

- Q: When two marketplace sources provide extensions with the same name, how should the system resolve this conflict? → A: Namespace by source - Extensions shown as "source-name/extension-name"
- Q: How should the system detect whether a Gemini CLI extension is currently installed? → A: Best effort both - Try registry first, then a file system scan
- Q: When the system encounters GitHub API rate limiting, what should it do? → A: Queue requests - Defer refresh until rate limit resets, show countdown timer
- Q: Should search/filter by keyword happen before or after fetching marketplace data? → A: Both supported - Users can fetch all extensions from all sources OR optionally use search-before-fetch as a performance optimization to filter API requests and reduce data transfer
- Q: How long should cached marketplace data remain valid before requiring a refresh? → A: User-controlled - Allow users to configure cache TTL with recommended default of 24 hours
- Q: Which metadata manifest should the marketplace parse from each extension repository to populate details and run validations? → A: gemini-extension.json
- Q: How should the marketplace handle an extension repository when its root gemini-extension.json manifest is missing or fails validation? → A: Skip the extension, continue processing, and surface a warning in results
- Q: How should the marketplace authenticate when a custom source requires private Git access? → A: Rely on existing Git credential helpers or environment variables without storing credentials
- Q: What level of observability should the marketplace extension guarantee for troubleshooting and monitoring? → A: Dual-mode logs plus structured metrics (e.g., counters for cache hits, rate-limit waits)
- Q: Which repository should ship as the default curated marketplace source at launch? → A: https://github.com/athola/gemini-marketplace (kept under our control so test data stays stable)

### Session 2025-10-11

- Q: When displaying the marketplace extension list in the CLI, what interaction model should be used? → A: Paginated output with navigation commands (next/prev)
- Q: When validating an extension repository's `gemini-extension.json` manifest, what depth of validation should be performed? → A: Progressive validation - Schema on fetch, full semantic when user views details
- Q: What is the expected maximum scale for marketplace extension catalogs that the system must handle efficiently? → A: Lazy load in 500-extension increments (supporting larger catalogs by fetching in batches of 500)
- Q: When a marketplace source repository is structured as a monorepo containing multiple extensions, how should the system discover individual extensions? → A: Directory scan - Recursively search for gemini-extension.json files in subdirectories, with recursion limit
- Q: When fetching marketplace data fails (network errors, timeouts, invalid responses), what retry strategy should the system employ? → A: Background retry - Queue for background retry while serving cached data
- Q: When recursively scanning marketplace source repositories for extension manifests (FR-013a), what maximum recursion depth should the system enforce to balance between discovering deeply nested extensions and preventing excessive traversal? → A: set default to 5 levels deep but allow it to be configurable

### Session 2025-10-12

- Q: Should `gemini marketplace list` keep users in an interactive pagination prompt by default or require an explicit flag? → A: Default to stateless execution with an optional `--interactive` flag for next/prev prompts

### Session 2025-10-12

- Q: Which command structure should the marketplace extension expose for listing extensions, viewing details, managing sources, refreshing data, and adjusting cache TTL? → A: Top-level `gemini marketplace` command with subcommands: `list`, `show <id>`, `search`, `sources add/list/remove`, `cache refresh`, `cache ttl set <hours>`

### Session 2025-10-26

- Q: How should “common keywords” be defined for SC-005 search relevance measurement? → A: Use live telemetry to identify the top searched terms each release cycle and validate relevance against that rotating list

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse Available Extensions (Priority: P1)

As a Gemini CLI user, I want to discover what extensions are available so that I can enhance my CLI experience without manually searching GitHub repositories.

**Why this priority**: This is the core feature. It solves the main problem of users having to manually search for extensions.

**Independent Test**: Can be tested by launching the marketplace and verifying that a list of extensions is displayed.

**Acceptance Scenarios**:

1. **Given** I am using Gemini CLI, **When** I run `gemini marketplace list`, **Then** I see a paginated list of available extensions with their names, descriptions, and source repositories
2. **Given** the extension list is displayed, **When** I view the list, **Then** I can see key metadata for each extension including version information and installation status
3. **Given** there are multiple extensions available, **When** I browse the list, **Then** each row displays the columns `namespace`, `name`, `version`, `categories`, `install status`, `source`, `warnings`, and `cache freshness`, with category values rendered as badges inline with the row
4. **Given** the extension list contains more items than fit on one page, **When** I launch `gemini marketplace list --interactive` and use navigation commands (next/prev), **Then** I can navigate forward and backward through pages of results without re-invoking the command
5. **Given** a marketplace source contains thousands of extensions, **When** I browse the list, **Then** the system lazily loads extensions in 500-extension increments, providing responsive performance without fetching the entire catalog upfront

---

### User Story 2 - View Extension Details (Priority: P2)

As a Gemini CLI user, I want to view detailed information about a specific extension so that I can make an informed decision about whether to install it.

**Why this priority**: Users need details to evaluate extensions after discovering them. This includes documentation, compatibility, and usage examples.

**Independent Test**: Can be tested by selecting an extension and verifying that its detailed information is displayed.

**Acceptance Scenarios**:

1. **Given** I am viewing the extension list, **When** I run `gemini marketplace show <source-name/extension-name>`, **Then** I see its description, author, repository URL, version, and compatibility information
2. **Given** I am viewing extension details, **When** the extension has documentation or README content, **Then** I can access that documentation directly
3. **Given** I am viewing extension details, **When** I want to install the extension, **Then** I can see clear installation instructions with the exact GitHub URL to use
4. **Given** I am viewing extension details, **When** the system performs full semantic validation, **Then** any manifest validation errors (invalid semver, malformed URLs, type mismatches) are displayed clearly to inform installation decisions

---

### User Story 3 - Search and Filter Extensions (Priority: P3)

As a Gemini CLI user, I want to search for extensions by keyword or filter by category so that I can quickly find relevant extensions.

**Why this priority**: Improves discovery as the number of extensions grows.

**Independent Test**: Can be tested by searching or filtering and verifying the list updates correctly.

**Acceptance Scenarios**:

1. **Given** I am viewing the marketplace, **When** I run `gemini marketplace search <keyword>`, **Then** the extension list filters to show only extensions matching that term in their name or description
2. **Given** extensions have categories or tags, **When** I run `gemini marketplace search --category <tag>`, **Then** only extensions in that category are displayed
3. **Given** I have applied filters, **When** I run `gemini marketplace list` without search parameters, **Then** the full extension list is displayed again

---

### User Story 4 - Manage Marketplace Sources (Priority: P4)

As a Gemini CLI user, I want to add custom marketplace sources so that I can access organization-specific or private extension repositories.

**Why this priority**: Supports team and enterprise use cases.

**Independent Test**: Can be tested by adding a custom source and verifying that its extensions appear in the marketplace.

**Acceptance Scenarios**:

1. **Given** I want to add a custom marketplace source, **When** I run `gemini marketplace sources add <url>`, **Then** that source is added to my marketplace configuration
2. **Given** I have multiple marketplace sources configured, **When** I browse extensions, **Then** I can see which source each extension comes from
3. **Given** I have added custom sources, **When** I run `gemini marketplace sources remove <source-name>`, **Then** that source is removed and extensions from that source no longer appear
4. **Given** I add a marketplace source structured as a monorepo with multiple extensions, **When** the system scans the repository, **Then** it recursively discovers all `gemini-extension.json` manifests in subdirectories (up to the recursion limit) and treats each as an independent extension

---

### Edge Cases

- Unreachable or invalid marketplace sources MUST surface a warning, preserve cached listings, and queue the request for background retry.
- Network errors during data fetch MUST trigger a background retry, with status visible to the user, while serving cached data.
- Extensions with missing or invalid `gemini-extension.json` metadata MUST be skipped and reported in a warning.
- Extensions failing full semantic validation MUST be listed but display validation errors on detail view.
- Deleted or moved extension repositories MUST be marked as unavailable, retaining cached metadata until expiration.
- Offline or no-network scenarios MUST display cached data with a staleness warning and queue a refresh for when connectivity resumes.
- Metadata revisions MUST be validated progressively (schema on fetch, semantics on detail view), with incompatibilities flagged.
- Recursive scanning for manifests MUST enforce a configurable depth limit (default: 5) to prevent excessive traversal, logging a warning if the limit is reached.
- Each discovered `gemini-extension.json` in a monorepo MUST be treated as an independent, namespaced extension.


## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: MUST retrieve and display a curated list of extensions from configured marketplace sources.
- **FR-001a**: MUST present extension lists using paginated output with navigation commands (next/prev).
- **FR-001b**: `gemini marketplace list` MUST execute as a single-shot command by default; an `--interactive` flag enables an interactive prompt for navigation.
- **FR-002**: MUST display extension metadata including name, description, repository URL, version, and author.
- **FR-002a**: MUST namespace extension identifiers as `source-name/extension-name` to handle naming conflicts.
- **FR-002b**: `gemini marketplace list` output MUST include `namespace`, `name`, `version`, `categories`, `install_status`, `source`, `warnings`, and `cache_freshness`.
- **FR-003**: MUST allow users to view detailed information for any listed extension.
- **FR-004**: MUST provide the GitHub repository URL for each extension to enable manual installation.
- **FR-005**: MUST support at least one default marketplace source.
- **FR-005a**: The default curated marketplace source MUST be `https://github.com/athola/gemini-marketplace` and enabled by default.
- **FR-006**: MUST support keyword search in name or description, with two modes: local filtering after fetching all extensions, or optional pre-filtering of API requests.
- **FR-007**: MUST allow users to add custom marketplace sources via GitHub repository or git URLs.
- **FR-008**: MUST allow users to list all configured marketplace sources.
- **FR-009**: MUST allow users to remove previously added marketplace sources.
- **FR-010**: MUST cache marketplace data locally with a configurable TTL (default: 24 hours).
- **FR-010a**: MUST allow users to configure the cache TTL.
- **FR-010b**: MUST implement lazy loading, fetching and caching data in increments of 500 extensions.
- **FR-010c**: MUST treat cached records as stale when `stored_at + TTL <= now`, label them as such, and enqueue a background refresh.
- **FR-011**: MUST provide a mechanism to manually refresh marketplace data, bypassing the cache.
- **FR-012**: MUST handle network errors gracefully by queuing failed requests for background retry and serving cached data.
- **FR-012a**: MUST handle GitHub API rate limiting by queuing refresh requests until the rate limit resets and displaying a countdown timer.
- **FR-012b**: MUST implement background retry for failed marketplace data fetches.
- **FR-013**: MUST validate marketplace source URLs and metadata format.
- **FR-013a**: MUST discover extensions by recursively scanning for `gemini-extension.json` files (up to a configurable depth, default: 5) and perform progressive validation (basic schema on fetch, full semantic on detail view).
- **FR-013b**: MUST omit extensions with invalid or missing `gemini-extension.json` manifests and emit a warning.
- **FR-013c**: MUST display full semantic validation errors on the extension detail view.
- **FR-013d**: MUST rely on existing Git credential helpers for authenticating private sources and not store credentials.
- **FR-013e**: MUST allow users to configure the maximum recursion depth for monorepo scanning.
- **FR-014**: MUST distinguish between installed and not-installed extensions by checking the Gemini CLI extension registry first, then the file system.
- **FR-014a**: MUST persist preferences, sources, and cache configuration in `$GEMINI_CONFIG/extensions/marketplace/preferences.json`.
- **FR-015**: MUST support filtering extensions by category or tags.
- **FR-016**: MUST expose a `gemini marketplace` command with subcommands: `list`, `show <id>`, `search`, `sources add/list/remove`, `cache refresh`, and `cache ttl set <hours>`.
- **FR-017**: The embedded HTTP API MUST be scoped for local CLI use only.

### Non-Functional Requirements

- **NFR-001**: MUST provide dual-mode logging (human-readable and JSON) and structured metrics for cache usage, rate-limit delays, refresh queue depth, and top searched terms.
- **NFR-002**: MUST handle large extension catalogs efficiently by implementing lazy loading in 500-extension increments.
- **NFR-003**: Telemetry collection MUST be opt-out by default and store aggregated, non-identifying data.

### Key Entities

- **Extension**: A Gemini CLI extension, with a namespaced unique identifier (`source-name/extension-name`), name, description, repository URL, version, author, source, categories, compatibility, and installation status.
- **Marketplace Source**: A repository or catalog of extensions, with a name, URL, type (GitHub, git, local), last updated timestamp, and enabled/disabled status.
- **Extension Metadata**: Information about an extension from its `gemini-extension.json` manifest, including README, installation instructions, and dependencies.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can discover extensions without leaving the CLI.
- **SC-002**: Users can view extension details and get installation instructions in under 30 seconds.
- **SC-003**: The marketplace displays extension lists within 2 seconds when using cached data.
- **SC-004**: Users can add custom sources and see their extensions within 5 seconds.
- **SC-005**: Search returns relevant results for at least 90% of top searched terms.
- **SC-006**: The system handles network failures gracefully, serving cached data while retrying in the background.
- **SC-007**: Users can configure cache expiration.

## Assumptions

- Extension metadata is defined by `gemini-extension.json` manifests.
- The default marketplace source is a GitHub repository.
- Extensions are installed via GitHub URLs.
- Users have network connectivity for initial data retrieval, but can browse cached data offline.
- Extensions follow semantic versioning.
- The marketplace extension is installed using the standard Gemini CLI method.
- The default 24-hour cache TTL is a reasonable balance, but is configurable.
- Marketplace catalogs may contain thousands of extensions, requiring lazy loading.
- Marketplace sources may be monorepos, requiring recursive scanning.
- Telemetry is opt-in and stores aggregated, non-identifying data.
- Preferences, cache, and source configurations are stored in `$GEMINI_CONFIG/extensions/marketplace/`.

## Dependencies

- Access to GitHub API or raw GitHub content.
- A supported Gemini CLI extension architecture.
- An established extension metadata standard.
- Access to the Gemini CLI extension registry or configuration for installation status.
- Knowledge of Gemini CLI extension installation directories for secondary checks.
- Availability of Git credential helpers for private sources.

## Out of Scope

- Automated extension installation.
- Extension verification, security scanning, or safety guarantees.
- Extension ratings, reviews, or feedback mechanisms.
- Extension dependency resolution or compatibility checking beyond displaying metadata.
- Extension update notifications or version management.
- Creating or publishing extensions.
- Hosting marketplace infrastructure.
