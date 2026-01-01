# Speckit + Superpowers Integration Plan

**Goal**: To ensure that every Speckit workflow in this repo auto-loads project-scoped superpowers skills while preserving productive "dual governance" between Speckit and Superpowers.

**Architecture**: A `speckit-meta` superpower skill will be introduced to orchestrate the existing skills and enforce collaboration rules with Speckit. The `AGENTS.md` file and each `/speckit:*` prompt will be updated to invoke the meta-skill, and documentation on conflict handling will be added.

**Tech Stack**: Codex prompts, superpowers skills, and markdown docs.

### Task 1: Add the Project-Scoped `speckit-meta` Skill

1.  **Create the skill file**: `.codex/superpowers/skills/speckit-meta/SKILL.md`
2.  **Write the skill definition**:
    *   State that it only applies inside `/home/alext/gemini-marketplace`.
    *   Load `using-superpowers` immediately, then instruct the agent to identify the active `/speckit:*` command and load the mapped skill(s).
    *   Define a checklist for confirming the repo path, running Speckit command prerequisites, logging any governance disagreements, and capturing the remediation owner.
    *   Describe the "productive conflict" rule: neither system blocks progress outright; disagreements must be logged and resolved before final verification.
3.  **Include instructions** for how to proceed if no `/speckit:*` command is active.

### Task 2: Require the Meta-Skill in `AGENTS.md`

1.  **Modify the `AGENTS.md` file**.
2.  **Instruct agents** to run the `speckit-meta` skill immediately after loading `AGENTS.md`.
3.  **Document** that Speckit and Superpowers should challenge each other constructively, logging variances rather than halting.
4.  **Mention** that this requirement is repo-specific to avoid accidental global usage.

### Task 3: Update the Speckit Prompt Templates

1.  **Modify the Speckit prompt templates**.
2.  **Add language** to the top of each prompt that ensures the `speckit-meta` skill is active.
3.  **Keep the wording consistent** to minimize confusion.

### Task 4: Document the Workflow and Validation Steps

1.  **Add a short section** to the `README.md` that describes how to run Speckit commands in this repo.
2.  **Explain the variance logging process** and how to resolve conflicts.
3.  **Document the verification guidance**.
4.  **Note any future follow-ups** in a "Next steps" subsection.
