---
name: ship-skills
stable-id: ship-skills
description: Use when creating, configuring, or understanding Ship skills — directory structure, SKILL.md format, typed variables, artifact types, progressive disclosure docs, publishing, and the getship.dev specification.
tags: [ship, skills, smart-skills, authoring, guide]
authors: [ship]
---

# Smart Skills

Ship's smart skill specification is a superset of plain agent skills. A smart skill can have typed variables, declared artifact types, visual design tokens, progressive documentation, and eval test cases. Same skill, different output for every user and project.

## Directory layout

```
.ship/skills/{id}/
  SKILL.md              <- agent instructions (MiniJinja template)
  assets/
    vars.json           <- variable schema and defaults
    ui/                 <- layout specs, design tokens (static only)
  app/                  <- optional custom frontend (future)
  scripts/              <- helper scripts referenced in SKILL.md
  references/
    docs/               <- human + agent documentation
  evals/
    evals.json          <- eval test cases
```

## Frontmatter

| Field | Required | Description |
|-------|----------|-------------|
| `name` | yes | Human-readable name. Lowercase, hyphens, digits. |
| `stable-id` | no | Storage key. Pattern: `[a-z0-9][a-z0-9\-]*`. Defaults to directory name. |
| `description` | yes | One sentence starting with "Use when..." for trigger matching. |
| `artifacts` | no | Array of artifact types this skill produces: `html`, `pdf`, `markdown`, `image`, `adr`, `note`, `json`, `url`. |
| `tags` | no | Category labels for discovery. |
| `authors` | no | Who wrote the skill. |
| `allowed-tools` | no | Space-delimited MCP tool names the skill requires. |

## Artifacts

Skills declare what they produce via the `artifacts` frontmatter field. The platform infers interaction capabilities from artifact types:

- `html` — renders in iframe, supports annotation + feedback events
- `pdf` — renders in PDF viewer, supports selection + feedback events
- `markdown` — renders with syntax highlighting, supports feedback events
- `image` — renders with zoom, supports annotation events
- `adr` — structured architecture decision record with schema, syncs to cloud docs API
- `note` — structured note with schema, syncs to cloud docs API
- `url` — iframes a local server URL (e.g. `vitest --ui`, `storybook`)
- `json` — renders as formatted JSON viewer

Typed artifacts like `adr` and `note` have schemas the platform knows. Studio renders purpose-built viewers. The cloud docs API ingests structured data, not text blobs.

## Variables (smart skills)

Define variables in `assets/vars.json` with a type (`string`, `bool`, `enum`, `array`, `object`), a `storage-hint` scope (`global`, `local`, `project`), and optional `default`, `label`, `description`.

Merge order: defaults, then global, then local, then project. Last wins.

SKILL.md is a MiniJinja template. Use `{{ var }}` for substitution, `{% if var %}` for conditionals, `{% for item in list %}` for loops. Undefined variables render as empty string.

## Design tokens (assets/ui/)

Static assets for visual consistency: layout specs, color palettes, typography rules, design tokens. Agents use these when generating artifacts so outputs from the same skill look consistent. HTML, CSS, and JSON only — no JavaScript in published skills.

## Reference docs

Put detailed documentation in `references/docs/`. SKILL.md stays concise (under 100 lines). Reference docs are retrieved on demand. Each page has frontmatter with `group`, `title`, `section`, `order`.

## What is not yet stable

Eval tooling, `app/` frontend serving, Ship SDK npm package, computed/dynamic vars, WASM audit sandbox, typed artifact schemas (adr, note).

For full details, read `references/docs/` in this skill directory.
