# Feature Specification: Gemini CLI Extension Marketplace

**Feature Branch**: `001-build-a-gemini`
**Created**: 2025-10-09
**Status**: Draft
**Input**: User description: "build a gemini CLI extension with Rust using the reference here: https://blog.google/technology/developers/gemini-cli-extensions/. the caveat is that gemini CLI extensions does not expose a plugin marketplace similar to Claude Code. refer to this: The Gemini CLI, an open-source AI agent, supports extensions to enhance its functionality. While there isn't a centralized "marketplace" in the traditional sense for these extensions currently, the method for installing them involves directly referencing their GitHub repositories. Google has expressed plans to streamline this process in the future, aiming for a simpler, one-line command installation experience for extensions, potentially through a command like Gemini extensions install <GitHub_URL>. Additionally, there is anticipation of a future "clearinghouse or repository" where Google-ratified extensions will be readily available for easy installation. For now, users typically install Gemini CLI extensions by obtaining them from their respective GitHub repositories. For example, extensions for BigQuery Data Analytics and Conversational Analytics are available as part of the Gemini CLI ecosystem. It is important to distinguish this from the Visual Studio Marketplace or other IDE-specific marketplaces, where extensions like "Gemini Code Assist" are available for integration within those development environments. While these offer Gemini-related functionality, they are distinct from the extensions designed specifically for the Gemini CLI itself.. we want to add an extension that lets us to list and peruse 3rd party exensions similar to how Claude Code implements `/plugin marketplace`: https://docs.claude.com/en/docs/claude-code/plugin-marketplaces"

## Clarifications

### Session 2025-10-09

- Q: When two marketplace sources provide extensions with the same name, how should the system resolve this conflict? → A: Namespace by source - Extensions shown as "source-name/extension-name"
- Q: How should the system detect whether a Gemini CLI extension is currently installed? → A: Best effort both - Try registry first, fall back to file system scan
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

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse Available Extensions (Priority: P1)

As a Gemini CLI user, I want to discover what extensions are available so that I can enhance my CLI experience without manually searching GitHub repositories.

**Why this priority**: This is the core value proposition - enabling discovery. Without the ability to browse extensions, users must rely on word-of-mouth or manual GitHub searches, which is the current friction point this feature addresses.

**Independent Test**: Can be fully tested by launching the marketplace interface and verifying that a list of available extensions is displayed with their basic metadata (name, description, repository URL). Delivers immediate value by solving the discovery problem.

**Acceptance Scenarios**:

1. **Given** I am using Gemini CLI, **When** I run `gemini marketplace list`, **Then** I see a paginated list of available extensions with their names, descriptions, and source repositories
2. **Given** the extension list is displayed, **When** I view the list, **Then** I can see key metadata for each extension including version information and installation status
3. **Given** there are multiple extensions available, **When** I browse the list, **Then** extensions are organized in a readable format with clear categorization
4. **Given** the extension list contains more items than fit on one page, **When** I launch `gemini marketplace list --interactive` and use navigation commands (next/prev), **Then** I can navigate forward and backward through pages of results without re-invoking the command
5. **Given** a marketplace source contains thousands of extensions, **When** I browse the list, **Then** the system lazily loads extensions in 500-extension increments, providing responsive performance without fetching the entire catalog upfront

---

### User Story 2 - View Extension Details (Priority: P2)

As a Gemini CLI user, I want to view detailed information about a specific extension so that I can make informed decisions about whether to install it.

**Why this priority**: Once users can discover extensions (P1), they need detailed information to evaluate them. This includes documentation, compatibility, and usage examples.

**Independent Test**: Can be tested by selecting any extension from the list and verifying that detailed information is displayed, including full description, installation instructions, requirements, and repository details.

**Acceptance Scenarios**:

1. **Given** I am viewing the extension list, **When** I run `gemini marketplace show <source-name/extension-name>`, **Then** I see comprehensive details including description, author, repository URL, version, and compatibility information
2. **Given** I am viewing extension details, **When** the extension has documentation or README content, **Then** I can access that documentation directly
3. **Given** I am viewing extension details, **When** I want to install the extension, **Then** I can see clear installation instructions with the exact GitHub URL to use
4. **Given** I am viewing extension details, **When** the system performs full semantic validation, **Then** any manifest validation errors (invalid semver, malformed URLs, type mismatches) are displayed clearly to inform installation decisions

---

### User Story 3 - Search and Filter Extensions (Priority: P3)

As a Gemini CLI user, I want to search for extensions by keyword or filter by category so that I can quickly find extensions relevant to my needs.

**Why this priority**: Enhances discoverability once the basic marketplace is established. As the number of available extensions grows, search becomes increasingly valuable.

**Independent Test**: Can be tested by entering search terms or applying filters and verifying that the displayed list updates to show only matching extensions.

**Acceptance Scenarios**:

1. **Given** I am viewing the marketplace, **When** I run `gemini marketplace search <keyword>`, **Then** the extension list filters to show only extensions matching that term in their name or description
2. **Given** extensions have categories or tags, **When** I run `gemini marketplace search --category <tag>`, **Then** only extensions in that category are displayed
3. **Given** I have applied filters, **When** I run `gemini marketplace list` without search parameters, **Then** the full extension list is displayed again

---

### User Story 4 - Manage Marketplace Sources (Priority: P4)

As a Gemini CLI user, I want to add custom marketplace sources so that I can access organization-specific or private extension repositories in addition to public ones.

**Why this priority**: Supports team and enterprise use cases where organizations may maintain private extension catalogs. Less critical for individual users but important for broader adoption.

**Independent Test**: Can be tested by adding a custom marketplace source URL and verifying that extensions from that source appear in the marketplace alongside public extensions.

**Acceptance Scenarios**:

1. **Given** I want to add a custom marketplace source, **When** I run `gemini marketplace sources add <url>`, **Then** that source is added to my marketplace configuration
2. **Given** I have multiple marketplace sources configured, **When** I browse extensions, **Then** I can see which source each extension comes from
3. **Given** I have added custom sources, **When** I run `gemini marketplace sources remove <source-name>`, **Then** that source is removed and extensions from that source no longer appear
4. **Given** I add a marketplace source structured as a monorepo with multiple extensions, **When** the system scans the repository, **Then** it recursively discovers all `gemini-extension.json` manifests in subdirectories (up to the recursion limit) and treats each as an independent extension

---

### Edge Cases

- Marketplace sources that are unreachable or return invalid data MUST surface a warning while preserving previously cached listings and queue the failed request for background retry
- Network errors, timeouts, or invalid responses during marketplace data fetch MUST trigger background retry while users continue browsing cached data, with retry status visible to users
- Extensions with missing `gemini-extension.json` metadata or failing basic schema validation (missing required fields) MUST be skipped during fetch and reported in the warning output
- Extensions that pass basic schema validation but fail full semantic validation (invalid semver, malformed URLs, type mismatches) MUST be listed but display validation errors when users view details
- Extension repositories that are deleted or moved MUST be indicated as unavailable while retaining cached metadata until expiration
- When the user is offline or has no network connectivity, the system MUST continue displaying cached data, inform the user that results may be stale, and queue refresh for background retry when connectivity resumes
- Extension metadata revisions that change format or version MUST be validated progressively: basic schema on fetch, full semantics on detail view, with incompatible manifests flagged appropriately at each stage
- When recursively scanning marketplace source repositories for extension manifests, the system MUST enforce a configurable maximum recursion depth limit (default: 5 levels) to prevent excessive directory traversal, stopping the scan when the limit is reached and logging a warning if directories remain unscanned
- Monorepo marketplace sources containing multiple extensions MUST have each discovered `gemini-extension.json` treated as an independent extension, with each extension namespaced appropriately by the source name

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST retrieve and display a curated list of available Gemini CLI extensions from configured marketplace sources
- **FR-001a**: System MUST present extension lists using paginated output with navigation commands (next/prev) to allow users to browse results page by page
- **FR-001b**: `gemini marketplace list` MUST execute as a single-shot command by default; an explicit `--interactive` flag enables an interactive prompt that accepts `next` / `prev` / `quit` navigation without re-running the command
- **FR-002**: System MUST display extension metadata including name, description, repository URL, version, and author information
- **FR-002a**: System MUST namespace extension identifiers as "source-name/extension-name" to handle extensions with identical names from different marketplace sources
- **FR-003**: System MUST allow users to view detailed information for any listed extension
- **FR-004**: System MUST provide the GitHub repository URL for each extension to enable manual installation
- **FR-005**: System MUST support at least one default marketplace source containing publicly available Gemini CLI extensions
- **FR-005a**: The default curated marketplace source MUST be seeded from `https://github.com/athola/gemini-marketplace` and remain enabled unless the user explicitly disables it
- **FR-006**: System MUST allow users to search extensions by keyword (supporting partial or full matches) in name or description, with two modes: (1) fetch all extensions from all sources then filter locally, or (2) optionally apply search filters to marketplace API requests before fetching to minimize API calls and improve performance
- **FR-007**: System MUST allow users to add custom marketplace sources via GitHub repository URLs or git URLs
- **FR-008**: System MUST allow users to list all configured marketplace sources
- **FR-009**: System MUST allow users to remove previously added marketplace sources
- **FR-010**: System MUST cache marketplace data locally to reduce network requests and enable offline viewing of previously fetched data, with a configurable cache time-to-live (TTL) defaulting to 24 hours
- **FR-010a**: System MUST allow users to configure the cache TTL to control the trade-off between data freshness and API usage
- **FR-010b**: System MUST implement lazy loading, fetching and caching marketplace extension data in increments of 500 extensions at a time to support larger catalogs efficiently without loading all extensions upfront
- **FR-011**: System MUST provide a mechanism to manually refresh/update marketplace data from sources, bypassing the cache
- **FR-012**: System MUST handle network errors gracefully by queuing failed requests for background retry while serving cached data to users, informing users when marketplace data cannot be retrieved immediately and indicating that retry is in progress
- **FR-012a**: System MUST handle GitHub API rate limiting by queuing refresh requests until the rate limit resets, displaying a countdown timer to inform users when the request will be retried
- **FR-012b**: System MUST implement background retry for failed marketplace data fetches (network errors, timeouts, invalid responses), allowing users to continue browsing cached data while retries proceed asynchronously
- **FR-013**: System MUST validate marketplace source URLs and metadata format before accepting them
- **FR-013a**: System MUST discover extensions within marketplace source repositories by recursively scanning for `gemini-extension.json` manifest files in subdirectories up to a configurable maximum recursion depth (default: 5 levels deep) supporting monorepo structures containing multiple extensions, parsing each discovered manifest using progressive validation: basic schema validation (required fields: name, version, description, repository) during initial fetch to populate listings, and full semantic validation (type checking, semver format, URL validity, constraint validation) when users view extension details
- **FR-013b**: System MUST omit extensions lacking a valid `gemini-extension.json` manifest or failing basic schema validation during fetch, emitting a user-visible warning identifying the skipped repository
- **FR-013c**: System MUST display full semantic validation errors (invalid semver, malformed URLs, type mismatches) clearly when users view extension details, allowing informed installation decisions
- **FR-013d**: System MUST rely on existing Git credential helpers or environment variables for authenticating private sources and must not store credentials itself
- **FR-013e**: System MUST allow users to configure the maximum recursion depth for monorepo directory scanning to balance between extension discovery coverage and performance requirements
- **FR-014**: System MUST distinguish between installed and not-installed extensions in the display by checking Gemini CLI's extension registry first, then falling back to file system scans of known extension directories if registry is unavailable
- **FR-015**: System MUST support filtering extensions by category or tags when provided in extension metadata
- **FR-016**: System MUST expose a top-level `gemini marketplace` command with subcommands `list`, `show <id>`, `search`, `sources add`, `sources list`, `sources remove`, `cache refresh`, and `cache ttl set <hours>` to provide browsing, management, and cache controls through a consistent CLI surface

### Non-Functional Requirements

- **NFR-001**: System MUST provide dual-mode logging (human-readable by default, JSON when explicitly requested) and expose structured metrics tracking cache usage, rate-limit delays, and source sync outcomes to support troubleshooting
- **NFR-002**: System MUST efficiently handle marketplace catalogs containing thousands of extensions by implementing lazy loading in 500-extension increments, ensuring responsive performance regardless of total catalog size

### Key Entities

- **Extension**: Represents a Gemini CLI extension with attributes including unique identifier (namespaced as "source-name/extension-name"), base name (the extension's own name), description, repository URL, version number, author/maintainer, source marketplace name, categories/tags, compatibility requirements, and installation status
- **Marketplace Source**: Represents a repository or catalog of extensions with attributes including source name (used for namespacing), source URL, source type (GitHub repo, git URL, local path), last updated timestamp, and enabled/disabled status
- **Extension Metadata**: Information about an extension including README content, installation instructions, dependencies, and compatibility information extracted from the root-level `gemini-extension.json` manifest

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can discover available Gemini CLI extensions without leaving the CLI interface or manually searching GitHub
- **SC-002**: Users can view details for any extension and obtain installation instructions in under 30 seconds
- **SC-003**: The marketplace displays extension lists within 2 seconds when using cached data
- **SC-004**: Users can successfully add custom marketplace sources and see extensions from those sources within 5 seconds of adding
- **SC-005**: Search functionality returns relevant results for at least 90% of common keywords related to available extensions
- **SC-006**: The system handles network failures gracefully with informative error messages, allowing users to continue browsing cached data while failed requests are queued for background retry with visible retry status
- **SC-007**: Users can configure cache expiration settings to balance between data freshness and API usage according to their needs

## Assumptions

- Extension metadata is defined by individual `gemini-extension.json` manifests (either at repository root or in subdirectories for monorepo structures) documented in the Gemini CLI extension release guide
- The default marketplace source will be a GitHub repository maintained by the community or project maintainers containing a curated list of extensions
- Extensions follow Gemini CLI's existing installation pattern via GitHub URLs
- Users have network connectivity for initial marketplace data retrieval, but can browse cached data offline
- Extension versioning follows semantic versioning principles
- The marketplace extension itself will be installed using the standard Gemini CLI extension installation method
- The default cache TTL of 24 hours provides a reasonable balance between data freshness and API usage for most users, but is configurable for specific needs
- Marketplace catalogs may grow to thousands of extensions across all sources, requiring lazy loading (500-extension increments) to maintain responsive performance
- Marketplace sources may be structured as monorepos containing multiple extensions, requiring recursive directory scanning with enforced depth limits to discover all extension manifests

## Dependencies

- Access to GitHub API or raw GitHub content for fetching marketplace data
- Gemini CLI extension architecture must support this marketplace extension
- Extension metadata standard must be established or adopted for consistent data retrieval
- Access to Gemini CLI's extension registry or configuration system for installation status detection
- Knowledge of Gemini CLI's extension installation directories for file system fallback detection
- Availability of Git credential helpers or environment-based authentication when accessing private marketplace sources

## Out of Scope

- Automated extension installation (users will use the standard Gemini CLI installation commands with provided URLs)
- Extension verification, security scanning, or safety guarantees
- Extension ratings, reviews, or user feedback mechanisms
- Extension dependency resolution or compatibility checking beyond displaying metadata
- Extension update notifications or version management
- Creating or publishing extensions to marketplaces
- Hosting marketplace infrastructure (relies on existing GitHub/git hosting)
