---
name: ship-skills
stable-id: ship-skills
description: Use when creating, configuring, or understanding Ship skills — directory structure, SKILL.md format, typed variables, events, interactive skills, progressive disclosure docs, publishing, and the getship.dev specification.
tags: [ship, skills, smart-skills, authoring, guide]
authors: [ship]
---

# Smart Skills

Ship's smart skill specification is a superset of plain agent skills. A smart skill can have typed variables, declared events, custom frontends, progressive documentation, and eval test cases. Same skill, different output for every user and project. Same events, reactive communication between agents and humans.

## Directory layout

```
.ship/skills/{id}/
  SKILL.md              <- agent instructions (MiniJinja template)
  assets/
    vars.json           <- variable schema and defaults
    events.json         <- event declarations (interactive skills)
  app/                  <- optional custom frontend
  scripts/              <- helper scripts referenced in SKILL.md
  references/
    docs/               <- human + agent documentation
  evals/
    evals.json          <- eval test cases
```

## Variables (smart skills)

Define variables in `assets/vars.json` with a type (`string`, `bool`, `enum`, `array`, `object`), a `storage-hint` scope (`global`, `local`, `project`), and optional `default`, `label`, `description`.

Merge order: defaults, then global, then local, then project. Last wins.

SKILL.md is a MiniJinja template. Use `{{ var }}` for substitution, `{% if var %}` for conditionals, `{% for item in list %}` for loops. Undefined variables render as empty string.

## Events (interactive skills)

Declare events in `assets/events.json`. Reference Ship built-in events and declare custom events in the skill's namespace.

```json
{
  "$schema": "https://getship.dev/schemas/events.schema.json",
  "ship": ["annotation", "feedback"],
  "custom": [
    { "id": "page_created", "direction": "out", "schema": {...} }
  ]
}
```

Ship built-in events: `annotation`, `feedback`, `selection`, `artifact_created`, `artifact_deleted`. Custom events become `{stable-id}.{id}` at runtime. Direction: `in` (human to agent), `out` (agent to human), `both`.

Events route to agents that have the skill, not to the skill itself. Skills define, agents react.

## Reference docs

Put detailed documentation in `references/docs/`. SKILL.md stays concise (under 100 lines). Reference docs are retrieved on demand. Each page has frontmatter with `group`, `title`, `section`, `order`.

## MCP tools

- `get_skill_vars` -- merged variable state for a skill
- `set_skill_var` -- write a single variable value
- `list_skill_vars` -- list all skills with configured variables
- `write_skill_file` -- write a file into a skill directory
- `delete_skill_file` -- delete a file from a skill directory

## stable-id

Add `stable-id` to SKILL.md frontmatter to preserve state across directory renames. Must match `[a-z0-9][a-z0-9\-]*`. All KV and event state is keyed by `stable-id`.

## What is not yet stable

Eval tooling, declarative var/event schema migrations, `app/` frontend serving, Ship SDK npm package, computed/dynamic vars, WASM audit sandbox.

For full details, read `references/docs/` in this skill directory.
