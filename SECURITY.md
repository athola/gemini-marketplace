# Security Overview

## Architecture
- The marketplace extension runs entirely inside the Gemini CLI process.
- No background HTTP server or daemon is started; all actions are in-process CLI commands.
- Outbound network requests are limited to fetching marketplace manifests over HTTPS via `reqwest`.

## Key Controls

-   **Input Validation**: CLI arguments are validated for length and allowed characters to prevent command injection vulnerabilities.
-   **Data Integrity**: Marketplace data is hashed with SHA-256 before caching to detect and reject any unauthorized tampering or corruption.
-   **Resource Management**: Cache TTL enforcement and intelligent refresh queue management prevent tight retry loops, which safeguards against denial-of-service scenarios.
-   **Data Isolation**: User preferences and marketplace configurations are stored in `$GEMINI_CONFIG/extensions/marketplace/` to ensure that sensitive user data is scoped within Gemini's dedicated configuration directory.

With the standalone server removed, the extension's attack surface is limited to CLI usage and outbound HTTPS requests.
