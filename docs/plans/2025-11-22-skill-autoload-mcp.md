# SKILL.md Autoload via MCP Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Provide MCP-based SKILL.md autoloading for Codex using a local server that scans `~/.codex` (and mirrors `~/.claude`), plus a hook script to inject skill text on every prompt submit.

**Architecture:** Node-based MCP server (`codex-skills`) launched by Codex via stdio; resources expose SKILL.md files; tools handle sync and autoload snippet; a small CLI emits hook JSON for UserPromptSubmit. Hook script in `~/.codex/hooks/codex` calls the CLI. Config updates add the new MCP server.

**Tech Stack:** Node 18+, @modelcontextprotocol/sdk, zod, gray-matter, builtin fs/path. Files live under `~/.codex/skills-server`.

---

### Task 1: Scaffold server project
**Files:**
- Create: `~/.codex/skills-server/package.json`
- Create: `~/.codex/skills-server/index.js`
- Create: `~/.codex/skills-server/bin/codex-skills`
- Create: `~/.codex/skills-server/README.md`
- (Optional) Create: `~/.codex/skills-server/.gitignore`

**Step 1: Initialize package metadata**
- Fill name `codex-skills`, version, type=module, bin mapping, scripts for `start` and `emit-autoload`.

**Step 2: Add dependencies**
- Install `@modelcontextprotocol/sdk`, `zod`, `gray-matter`.

**Step 3: Ignore node_modules**
- Add `.gitignore` entry if missing.

### Task 2: Implement skill scanning + sync utilities
**Files:**
- Modify: `~/.codex/skills-server/index.js`

**Step 1: Implement `findSkillFiles`**
- Recursive fs walk over `~/.codex` and optional `~/.claude` roots, skipping large/hidden dirs; collect SKILL.md paths with metadata (source, mtime).

**Step 2: Implement `loadSkill`**
- Read file, parse frontmatter via gray-matter, capture title/description/body, default name from dir.

**Step 3: Implement `syncFromClaude`**
- Copy newer SKILL.md files from `~/.claude` into `~/.codex/skills-mirrored/<relative>`; preserve mtime.

### Task 3: Expose MCP resources and tools
**Files:**
- Modify: `~/.codex/skills-server/index.js`

**Step 1: Register resources**
- Resource URI `skill://<source>/<relative>`; name and description from frontmatter; content is skill markdown.

**Step 2: Register tools**
- `list-skills` → list metadata
- `sync-from-claude` → runs copy util
- `autoload-snippet` → returns concatenated skills honoring filters (`names`, `autoload` flag) and max length

**Step 3: Start stdio transport**
- Use `McpServer` + `StdioServerTransport`; wire graceful shutdown.

### Task 4: CLI for hooks and manual use
**Files:**
- Modify: `~/.codex/skills-server/bin/codex-skills`

**Step 1: Implement command parser**
- `server` (default) runs MCP server
- `list` prints JSON list
- `sync` copies from `~/.claude`
- `emit-autoload` prints `hookSpecificOutput` JSON for UserPromptSubmit

**Step 2: Share code**
- Reuse scanning/sync helpers from index.js via exports.

### Task 5: Codex integration (config + hook)
**Files:**
- Modify: `~/.codex/config.toml`
- Create: `~/.codex/hooks/codex/prompt.on_user_prompt_submit`

**Step 1: Add MCP server entry**
- Name `codex-skills`, command `node`, args `['~/.codex/skills-server/index.js']`.

**Step 2: Add hook script**
- Shell script that runs `~/.codex/skills-server/bin/codex-skills emit-autoload` and echoes JSON if present.

**Step 3: Permissions**
- Ensure hook is executable.

### Task 6: Verification
**Files:**
- None (runtime commands)

**Step 1: Run MCP server sanity**
- `node ~/.codex/skills-server/index.js --once list-skills`

**Step 2: Run hook manually**
- `~/.codex/skills-server/bin/codex-skills emit-autoload | jq .` (verify non-empty)

**Step 3: Check config**
- `codex mcp list` or inspect session start logs for `codex-skills` connection.

### Task 7: Documentation update
**Files:**
- Modify: `~/.codex/skills-server/README.md`
- Modify: `docs/plans/2025-11-22-skill-autoload-mcp.md` (this file if needed)

**Step 1: Document usage**
- How to sync, list, and troubleshoot; note hook path and env knobs.

**Step 2: Next steps**
- Outline future work (error handling, cache, throttling, tests).

