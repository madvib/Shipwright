---
group: Smart Skills
title: Directory Structure
description: Complete layout of a smart skill directory with every file explained.
audience: public
section: reference
order: 2
---

# Directory Structure

Every skill is a directory under `.ship/skills/`. The directory name is the skill id. Additional skill directories can be configured in `ship.jsonc` via `project.skill_paths` (relative to project root). The default `.ship/skills/` is always included. This allows top-level skill directories like `docs/` for documentation skills.

## Two kinds of skills

### Agent skill layout

Most skills. Instructs agents how to do something.

```
.ship/skills/{skill-id}/
  SKILL.md
  assets/
    vars.json
    ui/               <- design tokens, layout specs (static only)
    templates/
  app/                <- optional custom frontend (future)
  scripts/
  evals/
    evals.json
```

### Doc-skill layout

Documentation that also loads as a skill. Lives under `docs/` in the project root. The `references/docs/` directory is collected by the site generator and rendered on the documentation website.

```
docs/{skill-id}/
  SKILL.md
  references/
    docs/             <- SITE GENERATOR INPUT — docs website pages only
      index.md
      {topic}.md
    api/
```

`references/docs/` is a **reserved namespace owned by the site generator**. Do not add it to agent skills. The site generator scans all `references/docs/` directories across the project and builds documentation pages from them. Putting agent skill content there pollutes the website and misleads agents into thinking the pattern is universal.

## SKILL.md

The skill's agent instructions. Always loaded into the agent's context window at startup. This is the only required file.

Starts with YAML frontmatter delimited by `---`. Body is Markdown, optionally with MiniJinja template syntax when `assets/vars.json` is present. Keep under 100 lines.

SKILL.md is the table of contents for the skill. Keep it focused. Long is fine if the content belongs there — do not move content out of SKILL.md into `references/docs/` just because it is long.

### Frontmatter fields

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Human-readable name. Lowercase, hyphens, digits, 1-64 chars. |
| `stable-id` | no | Storage key for variable state. Pattern: `[a-z0-9][a-z0-9\-]*`. Defaults to directory name. Survives renames. |
| `description` | yes | One sentence starting with "Use when..." for trigger matching. |
| `tags` | no | Category labels for discovery. YAML list. |
| `authors` | no | Who wrote the skill. YAML list. |
| `license` | no | SPDX identifier (e.g., `MIT`, `Apache-2.0`). |
| `compatibility` | no | Comma-separated provider names. Omit for universal. |
| `artifacts` | no | Array of artifact types: `html`, `pdf`, `markdown`, `image`, `adr`, `note`, `json`, `url`. |
| `allowed-tools` | no | Space-delimited MCP tool names the skill requires. |

## assets/

Bundled resources the skill depends on.

### assets/vars.json

Declares typed configuration variables with defaults and storage scopes. The presence of this file activates MiniJinja template resolution for SKILL.md. See the [Variables](variables.md) reference for the complete schema.

### assets/ui/

Static design assets for visual consistency: layout specs (`layout.json`), color palettes, typography rules, CSS custom properties. Agents reference these when generating artifacts so outputs from the same skill look consistent. HTML, CSS, and JSON only — no JavaScript in published skills.

### assets/templates/

Reusable config snippets or boilerplate files. Referenced from SKILL.md instructions. Not processed by the template engine -- these are files the agent copies or adapts during work.

## app/

Optional custom frontend served by the Ship runtime. Studio renders it in an iframe. Any framework works — the runtime serves static files. The only requirement is an `index.html` entry point.

If the app wants to communicate with agents, it includes the Ship SDK (`<script src="/_ship/sdk.js"></script>`) and uses `ship.action()` to emit events and `ship.on()` to receive them. Without the SDK, the app still renders — it just cannot interact with the event bus.

The skill author is responsible for building their app. Ship does not run build tools. Ship a `dist/` or pre-built output as `app/`. Same as deploying to any static host.

## scripts/

Helper scripts that SKILL.md instructs the agent to run. Shell scripts, Python scripts, or any executable. Referenced by relative path from SKILL.md. The skill directory is self-contained.

## references/ (doc-skills only)

`references/` is only present in doc-skills — skills that live under `docs/` and whose purpose is documentation rendered on the website.

### references/docs/

**Reserved namespace. Site generator input. Do not add to agent skills.**

Markdown (`.md`) or Markdoc (`.mdoc`) pages collected at build time and rendered on the documentation website. Also readable by agents via filesystem when they need reference depth.

- `index.md` is the landing page.
- Additional pages cover one concern each.
- Each page has YAML frontmatter with `title`, `order`, and optional `group`, `description`, `audience`, `section`.

If you are writing an agent skill and feel like you need `references/docs/`, put the content in SKILL.md instead.

### references/api/

API tables, external specs, and machine-readable reference data. Only in doc-skills that document external APIs or protocols.

## evals/evals.json

Eval test cases that measure whether the skill improves agent output. Each case defines a prompt, expected outcome, and optional assertions.

| Field | Required | Description |
|-------|----------|-------------|
| `id` | yes | Unique kebab-case identifier. |
| `prompt` | yes | Realistic user message. |
| `expected` | yes | Human-readable description of success. |
| `assertions` | no | Verifiable statements for grading. |
| `input_files` | no | Files needed in the eval workspace (relative paths). |

**Note:** The `evals/evals.json` structure is defined, but `ship skill eval` tooling to run evals automatically is not yet implemented.

## Minimal skill

The absolute minimum is a directory with SKILL.md:

```
.ship/skills/my-skill/
  SKILL.md
```

Add `assets/vars.json` when the skill needs user configuration. Add `scripts/` for helper scripts the agent runs. Add `evals/` when the skill produces verifiable output. Do not add `references/docs/` — that is for doc-skills only.

## Naming rules

Skill ids (directory names) must be lowercase alphanumeric with hyphens, 1-64 characters. No leading, trailing, or consecutive hyphens. Pattern: `[a-z0-9][a-z0-9\-]*`.

## Skill discovery

The runtime resolves skill directories from `project.skill_paths` in `ship.jsonc`. Paths are relative to `.ship/`. Absolute paths are rejected. If the field is absent or empty, the default `skills/` is used.

When multiple skill paths are configured, the first directory containing a given skill id wins. This allows layering project-specific skills over shared ones.
