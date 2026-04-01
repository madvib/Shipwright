---
group: Smart Skills
title: Variables
description: vars.json schema, types, storage scopes, merge order, template syntax, CLI, and MCP tools.
audience: public
section: reference
order: 3
---

# Variables

Variables make the same skill produce different output for different users and projects. Declared in `assets/vars.json`, stored in `platform.db` KV, resolved into SKILL.md at compile time via MiniJinja.

## vars.json schema

Every key in `vars.json` (except `$schema`) defines one variable. The JSON Schema is published at `https://getship.dev/schemas/vars.schema.json`.

```json
{
  "$schema": "https://getship.dev/schemas/vars.schema.json",
  "commit_style": {
    "type": "enum",
    "default": "conventional",
    "storage-hint": "global",
    "values": ["conventional", "gitmoji", "angular"],
    "label": "Commit style",
    "description": "Format applied to every commit message"
  }
}
```

### Variable fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | string | no | `"string"` | Variable type. One of: `string`, `bool`, `enum`, `array`, `object`. |
| `default` | any | no | none | Value used when no user state exists. Must match the declared type. |
| `storage-hint` | string | no | `"global"` | Where user values are stored: `global`, `local`, or `project`. |
| `values` | string[] | enum only | -- | Allowed values for enum type. Validated on `ship vars set`. |
| `label` | string | no | -- | Human-readable name. Shown in Studio and CLI output. |
| `description` | string | no | -- | Longer explanation. Shown in Studio tooltips and docs site. |

No additional properties are allowed beyond these six fields. A bare `{}` entry creates a string variable with no default.

## Variable types

| Type | JSON value | Use for | Example |
|------|-----------|---------|---------|
| `string` | `"text"` | Paths, names, free-form config | `"src/lib"` |
| `bool` | `true`/`false` | Feature toggles | `true` |
| `enum` | `"value"` | Fixed choices (must declare `values`) | `"conventional"` |
| `array` | `["a", "b"]` | Lists (co-authors, ignore patterns) | `["Alice", "Bob"]` |
| `object` | `{...}` | Structured config (rarely needed) | `{"key": "val"}` |

## Storage scopes

All variable state lives in `platform.db` as key-value pairs. The `storage-hint` field determines which KV namespace receives user state.

| Scope | KV namespace | Semantics | Use for |
|-------|-------------|-----------|---------|
| `global` | `skill_vars:{id}` | Machine-wide, all projects | Personal preferences (style, format) |
| `local` | `skill_vars.local:{ctx}:{id}` | This project, personal only | Individual overrides |
| `project` | `skill_vars.project:{ctx}:{id}` | This project, team-shared | Team conventions |

- `{id}` is the skill's `stable-id` (or directory name if no stable-id is set).
- `{ctx}` is a stable 16-character hex token derived from the `.ship/` directory path.

Each variable is read from and written to exactly the namespace matching its `storage-hint`. A global variable is only stored in the global namespace, a local variable only in the local namespace, and so on.

### Merge order

The runtime merges four layers when reading variable state. Last wins.

```
defaults (vars.json "default" field)
  -> global (machine-wide KV)
    -> local (project-personal KV)
      -> project (project-team KV)
```

A project-scope value overrides everything. A global value overrides only the default. Variables without a default and without user state are absent from the merged result.

## Template syntax

SKILL.md is rendered as a MiniJinja template (Jinja2-compatible). The engine runs in Rust with chainable undefined behavior -- no file loader, no custom functions.

**Substitution.** Wrap a variable name in double curly braces to insert its value. Dot-path access works for object variables (e.g., `object_var.field`).

**Conditionals.** Use `if`/`elif`/`else`/`endif` blocks to conditionally include content. Supports boolean checks (render content when a variable is truthy) and equality checks (render content when a variable matches a specific string).

**Loops.** Use `for`/`endfor` to iterate array variables. Each iteration has access to the current element.

**Note:** Template syntax uses double curly braces for substitution and percent-brace pairs for control flow. These are standard Jinja2 constructs processed by MiniJinja. Do not use these patterns in Markdoc files -- Markdoc will try to parse them.

### Truthiness

| Value | Truthy? |
|-------|---------|
| `true` | yes |
| `false` | no |
| `""` (empty string) | no |
| `"anything"` | yes |
| `[]` (empty array) | no |
| `[1, 2]` | yes |
| `null` / undefined | no |

### Error handling

Undefined variables render as empty string. The engine uses MiniJinja's chainable undefined behavior, so accessing a field on an undefined value also returns empty rather than erroring. Template syntax errors cause the engine to fall back to the original unresolved content with a warning to stderr.

## Runtime variables

In addition to user-configured vars, the compiler automatically injects a `runtime` object into every skill template at compile time. These are read-only — derived from project state, not user-settable, and do not appear in `vars.json`.

| Key | Type | Description |
|-----|------|-------------|
| `runtime.agents` | `[{id, name, description}]` | Agent profiles from `.ship/agents/*.toml` |
| `runtime.providers` | `string[]` | Active provider IDs, e.g. `["claude", "cursor"]` |
| `runtime.model` | `string` | Configured model override; empty string if not set |
| `runtime.skills` | `[{id, name, description}]` | All skills active in this compile |

`description` fields can be `null` — guard before rendering:

```
{% for a in runtime.agents %}
- **{{ a.id }}**{% if a.description %} — {{ a.description }}{% endif %}
{% endfor %}

{% if runtime.model %}
Model in use: {{ runtime.model }}
{% endif %}
```

Runtime variables follow the same template rules as user vars: undefined keys render as empty string, dot-path access on null returns empty rather than erroring.

## CLI commands

The `ship vars` CLI reads and writes variable state with type validation.

| Command | Description |
|---------|-------------|
| `ship vars get <skill-id>` | Show merged state for all variables |
| `ship vars get <skill-id> <key>` | Show merged value for a single variable |
| `ship vars set <skill-id> <key> <value>` | Set a variable. Routes to correct scope. Validates type and enum constraints. |
| `ship vars append <skill-id> <key> '<json>'` | Append to an array variable |
| `ship vars reset <skill-id>` | Clear all user state for the skill across all three scopes, reverting to defaults |

## MCP tools

Agents interact with variables through these MCP tools.

| Tool | Parameters | Description |
|------|-----------|-------------|
| `get_skill_vars` | `skill_id` | Returns merged variable state for a skill |
| `set_skill_var` | `skill_id`, `key`, `value` | Writes a single variable value |
| `list_skill_vars` | (none) | Lists all skills that have `assets/vars.json` with their merged state |

Two additional tools manage skill files on disk:

| Tool | Parameters | Description |
|------|-----------|-------------|
| `write_skill_file` | `skill_id`, `path`, `content` | Write a file into a skill directory |
| `delete_skill_file` | `skill_id`, `path` | Delete a file from a skill directory (cannot delete SKILL.md) |

## Content hashing

When publishing, Ship hashes the skill directory tree (SKILL.md, assets/, references/, evals/). User variable state in `platform.db` is excluded from the hash. Changing SKILL.md or vars.json schema produces a new version. Changing user variable values does not.
