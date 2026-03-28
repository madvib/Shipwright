---
name: ship-skills
stable-id: ship-skills
description: Use when creating, configuring, or understanding Ship skills — directory structure, SKILL.md format, typed variables, MiniJinja templates, progressive disclosure docs, publishing, and the getship.dev specification.
tags: [ship, skills, smart-skills, authoring, guide]
authors: [ship]
---

# Smart Skills

A smart skill is a skill with typed configuration variables that resolve into content at compile time. Same skill, different output for every user and project.

## How it works

1. Declare variables in `assets/vars.json` with types, defaults, and storage scopes.
2. Reference them in `SKILL.md` using `{{ var }}`, `{% if %}`, `{% for %}`.
3. Users set values via `ship vars set`, the Studio Skills IDE, or by asking an agent.
4. `ship use` merges state (defaults, global, local, project) and resolves the template.

## Directory layout

```
.ship/skills/{id}/
  SKILL.md              <- agent instructions (MiniJinja template)
  assets/
    vars.json           <- variable schema and defaults
  scripts/              <- helper scripts referenced in SKILL.md
  references/
    docs/               <- human + agent documentation (Markdown/Markdoc)
  evals/
    evals.json          <- eval test cases (planned tooling)
```

## Variables

Define each variable in `assets/vars.json` with a type (`string`, `bool`, `enum`, `array`, `object`), a `storage-hint` scope (`global`, `local`, `project`), and optional `default`, `label`, `description`.

Merge order: defaults, then global, then local, then project. Last wins.

Use `ship vars get <skill-id>` to read merged state. Use `ship vars set <skill-id> <key> <value>` to write. Enum values are validated on set.

## Templates

SKILL.md is a MiniJinja template. Use `{{ var }}` for substitution, `{% if var %}` for conditionals, `{% for item in list %}` for loops. Undefined variables render as empty string. Template errors fall back to original content with a warning.

## Reference docs

Put detailed documentation in `references/docs/`. The `index.md` page is the landing page. Additional pages cover specific concerns. Each page has frontmatter with `group`, `title`, `section`, `order`, and optional `audience`. The `group` field sets the docs site sidebar collection.

SKILL.md stays concise (under 100 lines). Reference docs are retrieved on demand when depth is needed.

## MCP tools

Agents read and write vars through these MCP tools:
- `get_skill_vars` -- merged variable state for a skill
- `set_skill_var` -- write a single variable value
- `list_skill_vars` -- list all skills with configured variables
- `write_skill_file` -- write a file into a skill directory
- `delete_skill_file` -- delete a file from a skill directory

## stable-id

Add `stable-id` to SKILL.md frontmatter to preserve variable state across directory renames. Must match `[a-z0-9][a-z0-9\-]*`. All KV state is keyed by `stable-id`.

## Content hashing

Skill content (SKILL.md, assets/, references/, evals/) is hashed for publishing. User variable state in platform.db is never included in the hash. Changing your vars does not create a new version.

## What is not yet stable

Eval tooling (`ship skill eval`) is planned but not implemented. The `evals/evals.json` structure is defined but cannot be run automatically. Declarative var migrations, computed/dynamic vars, WASM audit sandbox, `min-runtime-version`, and structured `allowed-tools` are planned for future releases.

For full details, read `references/docs/` in this skill directory.
