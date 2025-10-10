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

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Browse Available Extensions (Priority: P1)

As a Gemini CLI user, I want to discover what extensions are available so that I can enhance my CLI experience without manually searching GitHub repositories.

**Why this priority**: This is the core value proposition - enabling discovery. Without the ability to browse extensions, users must rely on word-of-mouth or manual GitHub searches, which is the current friction point this feature addresses.

**Independent Test**: Can be fully tested by launching the marketplace interface and verifying that a list of available extensions is displayed with their basic metadata (name, description, repository URL). Delivers immediate value by solving the discovery problem.

**Acceptance Scenarios**:

1. **Given** I am using Gemini CLI, **When** I execute a marketplace command to list extensions, **Then** I see a list of available extensions with their names, descriptions, and source repositories
2. **Given** the extension list is displayed, **When** I view the list, **Then** I can see key metadata for each extension including version information and installation status
3. **Given** there are multiple extensions available, **When** I browse the list, **Then** extensions are organized in a readable format with clear categorization

---

### User Story 2 - View Extension Details (Priority: P2)

As a Gemini CLI user, I want to view detailed information about a specific extension so that I can make informed decisions about whether to install it.

**Why this priority**: Once users can discover extensions (P1), they need detailed information to evaluate them. This includes documentation, compatibility, and usage examples.

**Independent Test**: Can be tested by selecting any extension from the list and verifying that detailed information is displayed, including full description, installation instructions, requirements, and repository details.

**Acceptance Scenarios**:

1. **Given** I am viewing the extension list, **When** I select a specific extension, **Then** I see comprehensive details including description, author, repository URL, version, and compatibility information
2. **Given** I am viewing extension details, **When** the extension has documentation or README content, **Then** I can access that documentation directly
3. **Given** I am viewing extension details, **When** I want to install the extension, **Then** I can see clear installation instructions with the exact GitHub URL to use

---

### User Story 3 - Search and Filter Extensions (Priority: P3)

As a Gemini CLI user, I want to search for extensions by keyword or filter by category so that I can quickly find extensions relevant to my needs.

**Why this priority**: Enhances discoverability once the basic marketplace is established. As the number of available extensions grows, search becomes increasingly valuable.

**Independent Test**: Can be tested by entering search terms or applying filters and verifying that the displayed list updates to show only matching extensions.

**Acceptance Scenarios**:

1. **Given** I am viewing the marketplace, **When** I enter a search term, **Then** the extension list filters to show only extensions matching that term in their name or description
2. **Given** extensions have categories or tags, **When** I apply a category filter, **Then** only extensions in that category are displayed
3. **Given** I have applied filters, **When** I clear the filters, **Then** the full extension list is displayed again

---

### User Story 4 - Manage Marketplace Sources (Priority: P4)

As a Gemini CLI user, I want to add custom marketplace sources so that I can access organization-specific or private extension repositories in addition to public ones.

**Why this priority**: Supports team and enterprise use cases where organizations may maintain private extension catalogs. Less critical for individual users but important for broader adoption.

**Independent Test**: Can be tested by adding a custom marketplace source URL and verifying that extensions from that source appear in the marketplace alongside public extensions.

**Acceptance Scenarios**:

1. **Given** I want to add a custom marketplace source, **When** I provide a valid GitHub repository or URL, **Then** that source is added to my marketplace configuration
2. **Given** I have multiple marketplace sources configured, **When** I browse extensions, **Then** I can see which source each extension comes from
3. **Given** I have added custom sources, **When** I want to remove a source, **Then** I can remove it and extensions from that source no longer appear

---

### Edge Cases

- Marketplace sources that are unreachable or return invalid data MUST surface a warning while preserving previously cached listings
- Extensions with missing or invalid `gemini-extension.json` metadata MUST be skipped and reported in the warning output
- Extension repositories that are deleted or moved MUST be indicated as unavailable while retaining cached metadata until expiration
- When the user is offline or has no network connectivity, the system MUST continue displaying cached data and inform the user that results may be stale
- Extension metadata revisions that change format or version MUST be validated against the current schema, and incompatible manifests flagged with the same warning mechanism

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST retrieve and display a curated list of available Gemini CLI extensions from configured marketplace sources
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
- **FR-011**: System MUST provide a mechanism to manually refresh/update marketplace data from sources, bypassing the cache
- **FR-012**: System MUST handle network errors gracefully and inform users when marketplace data cannot be retrieved
- **FR-012a**: System MUST handle GitHub API rate limiting by queuing refresh requests until the rate limit resets, displaying a countdown timer to inform users when the request will be retried
- **FR-013**: System MUST validate marketplace source URLs and metadata format before accepting them
- **FR-013a**: System MUST parse the canonical `gemini-extension.json` manifest at the repository root to populate and validate extension metadata per Gemini CLI guidelines
- **FR-013b**: System MUST omit extensions lacking a valid `gemini-extension.json` manifest and emit a user-visible warning identifying the skipped repository
- **FR-013c**: System MUST rely on existing Git credential helpers or environment variables for authenticating private sources and must not store credentials itself
- **FR-014**: System MUST distinguish between installed and not-installed extensions in the display by checking Gemini CLI's extension registry first, then falling back to file system scans of known extension directories if registry is unavailable
- **FR-015**: System MUST support filtering extensions by category or tags when provided in extension metadata

### Non-Functional Requirements

- **NFR-001**: System MUST provide dual-mode logging (human-readable by default, JSON when explicitly requested) and expose structured metrics tracking cache usage, rate-limit delays, and source sync outcomes to support troubleshooting

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
- **SC-006**: The system handles network failures gracefully with informative error messages, allowing users to continue browsing cached data
- **SC-007**: Users can configure cache expiration settings to balance between data freshness and API usage according to their needs

## Assumptions

- Extension metadata is defined by the root-level `gemini-extension.json` manifest documented in the Gemini CLI extension release guide
- The default marketplace source will be a GitHub repository maintained by the community or project maintainers containing a curated list of extensions
- Extensions follow Gemini CLI's existing installation pattern via GitHub URLs
- Users have network connectivity for initial marketplace data retrieval, but can browse cached data offline
- Extension versioning follows semantic versioning principles
- The marketplace extension itself will be installed using the standard Gemini CLI extension installation method
- The default cache TTL of 24 hours provides a reasonable balance between data freshness and API usage for most users, but is configurable for specific needs

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
