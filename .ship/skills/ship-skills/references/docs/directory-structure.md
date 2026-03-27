---
title: Skill Directory Structure
description: Complete reference for every file and directory in a Ship smart skill.
audience: public
section: reference
order: 2
---

# Skill Directory Structure

Every smart skill is a directory under `.ship/skills/`. The directory name is the skill id. The canonical layout:

```
.ship/skills/{skill-id}/
  SKILL.md
  assets/
    vars.json
    templates/
  scripts/
  references/
    docs/
      index.md
      {topic}.md
    api/
  evals/
    evals.json
```

## File-by-file reference

### SKILL.md

The skill's agent instructions. This file is always loaded into the agent's context window at startup. It is the only required file.

- Starts with YAML frontmatter (delimited by `---`).
- Body is Markdown, optionally with MiniJinja template syntax.
- Keep under 100 lines. Agents lose focus in long prompts.

#### Frontmatter fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Human-readable skill name. Must match the directory name. Lowercase, hyphens, digits, 1-64 chars. |
| `stable-id` | no | Storage key for variable state. Must match `[a-z0-9][a-z0-9\-]*`. Defaults to directory name if omitted. Survives directory renames. |
| `description` | yes | One sentence starting with "Use when..." to help trigger matching. Agents scan this to decide relevance. |
| `tags` | no | Category labels for discovery. Bracket-delimited YAML list. |
| `authors` | no | Who wrote the skill. Bracket-delimited YAML list. |
| `license` | no | SPDX license identifier (e.g., `MIT`, `Apache-2.0`). |
| `compatibility` | no | Comma-separated provider names. Omit for universal compatibility. |
| `allowed-tools` | no | Space-delimited MCP tool names the skill requires. Used for permission auditing. |

### assets/

Contains bundled resources the skill depends on.

#### assets/vars.json

Declares typed configuration variables with defaults and storage scopes. The presence of this file activates MiniJinja template resolution for SKILL.md.

```json
{
  "$schema": "https://getship.dev/schemas/vars.schema.json",
  "variable_name": {
    "type": "enum",
    "default": "value",
    "storage-hint": "global",
    "values": ["value", "other"],
    "label": "Human Name",
    "description": "What this controls."
  }
}
```

See the [Variables](variables.md) reference for the complete schema.

#### assets/templates/

Reusable config snippets or boilerplate files. Referenced from SKILL.md instructions. Not processed by the template engine -- these are files the agent copies or adapts during its work.

### scripts/

Helper scripts that SKILL.md instructs the agent to run. These are executable files (shell scripts, Python scripts, etc.) that support the skill's workflow.

Scripts are referenced by relative path from SKILL.md. The skill directory is self-contained -- everything the skill needs is co-located.

### references/

Supporting material not loaded into the agent's context by default.

#### references/docs/

Documentation pages in Markdown (`.md`) or Markdoc (`.mdoc`) format. Serves two audiences: humans browsing a docs site and agents retrieving context on demand.

- `index.md` is the landing page. Always create this first.
- Additional pages cover specific concerns (commands, patterns, troubleshooting).
- Each page has YAML frontmatter with `title`, `section`, `order`, and optional `audience` and `description`.

See the [Skill Documentation](documentation.md) reference for frontmatter details.

#### references/api/

API tables, external specs, and machine-readable reference data. Used for skills that document external APIs or protocols.

### evals/

#### evals/evals.json

Eval test cases that measure whether the skill reliably improves agent output. Each case defines a prompt, expected outcome, and optional assertions.

```json
{
  "evals": [
    {
      "id": "eval-descriptive-name",
      "prompt": "Realistic user message",
      "expected": "Description of successful output",
      "assertions": ["Verifiable statement about the output"]
    }
  ]
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `id` | yes | Unique kebab-case identifier. |
| `prompt` | yes | Realistic user message. |
| `expected` | yes | Human-readable description of success. |
| `assertions` | no | Verifiable statements for grading. |
| `vars` | no | Variable overrides for this eval case. |
| `input_files` | no | Files needed in the eval workspace (relative paths). |

**Planned -- not yet available in stable releases.** The `evals/evals.json` structure is defined, but `ship skill eval` tooling to run evals automatically is not yet implemented. Until then, evals are run manually or with a subagent per case.

## Minimal skill

The absolute minimum is a directory with SKILL.md:

```
.ship/skills/my-skill/
  SKILL.md
```

Add `assets/vars.json` when the skill needs user configuration. Add `references/docs/` when content exceeds what fits in SKILL.md. Add `evals/` when the skill produces verifiable output.

## Naming rules

Skill ids (directory names) must be:

- Lowercase alphanumeric with hyphens
- 1-64 characters
- No leading or trailing hyphens
- No consecutive hyphens
- Pattern: `[a-z0-9][a-z0-9\-]*`
