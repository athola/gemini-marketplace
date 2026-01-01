# Catchup Prompt Integration Plan

**Goal**: Introduce a `catchup` superpowers skill and wire `/prompts:catchup` to call it so that repo-status reviews align with the superpowers workflow.

**Architecture**: The prompt will be stored in `.codex/prompts/catchup.md`, and the reusable skill will be in `.codex/superpowers/skills/catchup/SKILL.md`. The prompt will ensure that the agent runs the new skill, while the skill will contain the full checklist. After editing, these files will be synced into `~/.codex` so that the slash command and skill are immediately available.

**Tech Stack**: Codex prompts, superpowers skills, and git CLI commands.

### Task 1: Scaffold the Catchup Prompt

1.  **Create the prompt file**: `.codex/prompts/catchup.md`
2.  **Add metadata**: Add YAML frontmatter that describes the prompt.
3.  **Reference the skill**: Instruct the user to run the `catchup` skill and mention that the `TodoWrite` entries from the skill are mandatory.
4.  **Describe the fallback**: If the skill cannot be loaded, direct the user to follow the manual steps.
5.  **Save the file**.

### Task 2: Implement the Catchup Skill

1.  **Create the skill file**: `.codex/superpowers/skills/catchup/SKILL.md`
2.  **Write the header**: Follow the structure of other skills: name, description, and repo-specific guidance.
3.  **Create the checklist**: Provide a short checklist that requires `TodoWrite` items for repo verification, git status, branch/base review, diff summaries, file-by-file notes, and follow-up capture.
4.  **Provide command guidance**: Specify lightweight git commands and remind agents to open only important files.
5.  **Add integration hooks**: Reference compatible skills where helpful, but keep this skill narrowly focused on summarizing code changes.
6.  **Save the file**.

### Task 3: Update Documentation and Install the Prompt/Skill

1.  **Document the workflow**: Add a short section to the `README.md` that describes how `/prompts:catchup` now requires running the `catchup` skill.
2.  **Copy the assets to the Codex global directories**:
    ```bash
    cp ./.codex/prompts/catchup.md ~/.codex/prompts/
    cp -R ./.codex/superpowers/skills/catchup ~/.codex/superpowers/skills/
    ```
3.  **Verify**: Execute `/prompts:catchup` to confirm that the instructions reference the new skill.
4.  **Update the plan status**: Mark the plan tasks as complete once verification succeeds.
