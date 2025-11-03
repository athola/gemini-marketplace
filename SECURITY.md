# Security Overview

## Architecture
- The marketplace extension runs entirely inside the Gemini CLI process.
- No background HTTP server or daemon is started; all actions are in-process CLI commands.
- Outbound network requests are limited to fetching marketplace manifests over HTTPS via `reqwest`.

## Key Controls
To mitigate common security risks, the marketplace extension implements several key controls:
- **Input Validation:** CLI arguments are rigorously validated for length and allowed characters. This prevents command injection vulnerabilities and ensures that only well-formed inputs are processed, protecting against unexpected behavior or malicious commands.
- **Data Integrity:** Marketplace data is hashed with SHA-256 before caching. This cryptographic checksum ensures the integrity of cached data, allowing the system to detect and reject any unauthorized tampering or corruption.
- **Resource Management:** Cache TTL (Time-To-Live) enforcement and intelligent refresh queue management prevent tight retry loops. This safeguards against denial-of-service scenarios by limiting repeated requests to external sources and managing resource consumption effectively.
- **Data Isolation:** User preferences and marketplace configurations are stored under `$GEMINI_CONFIG/extensions/marketplace/`. This ensures that sensitive user data is scoped within Gemini's dedicated configuration directory, providing clear data isolation and preventing unauthorized access or modification.

With the standalone server removed, the extension's attack surface is limited to CLI usage and outbound HTTPS requests.
