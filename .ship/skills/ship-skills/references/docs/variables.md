---
group: Smart Skills
title: Variables
description: Complete reference for skill variables — vars.json schema, types, storage scopes, merge order, templates, CLI, and MCP tools.
audience: public
section: reference
order: 3
---

# Variables

Variables make the same skill produce different output for different users and projects. They are declared in `assets/vars.json`, stored in `platform.db` KV, and resolved into SKILL.md at compile time via MiniJinja.

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
| `type` | string | no | `"string"` | Variable type. Determines validation and UI control. |
| `default` | any | no | none | Value used when no user state exists. Must match the declared type. |
| `storage-hint` | string | no | `"global"` | Where user values are stored: `global`, `local`, or `project`. |
| `values` | string[] | enum only | -- | Allowed values for enum type. Validated on set and at compile time. |
| `label` | string | no | -- | Human-readable name. Shown in Studio UI and CLI output. |
| `description` | string | no | -- | Longer explanation. Shown in Studio tooltips and the docs site. |

No fields are technically required. A bare `{}` entry creates a string variable with no default.

No additional properties are allowed beyond these six fields.

## Variable types

| Type | JSON value | Use for | Example |
|------|-----------|---------|---------|
| `string` | `"text"` | Paths, names, free-form config | `"src/lib"` |
| `bool` | `true`/`false` | Feature toggles | `true` |
| `enum` | `"value"` | Fixed choices (must declare `values`) | `"conventional"` |
| `array` | `["a", "b"]` | Lists (co-authors, ignore patterns) | `["Alice", "Bob"]` |
| `object` | `{...}` | Structured config (rarely needed) | `{"key": "val"}` |

## Storage scopes

All variable state lives in `platform.db` as key-value pairs. The `storage-hint` field determines which KV namespace a variable's user state is written to.

| Scope | KV namespace | Semantics | Use for |
|-------|-------------|-----------|---------|
| `global` | `skill_vars:{id}` | Machine-wide, all projects | Personal preferences (style, format) |
| `local` | `skill_vars.local:{ctx}:{id}` | This project, personal only | Individual overrides |
| `project` | `skill_vars.project:{ctx}:{id}` | This project, team-shared | Team conventions, tool config |

- `{id}` is the skill's `stable-id` (or directory name if no stable-id is set).
- `{ctx}` is a stable 16-character hex token derived from the project path.

### Merge order

Variable state is merged from four layers. Last wins.

```
defaults (vars.json "default" field)
  -> global (machine-wide KV)
    -> local (project-personal KV)
      -> project (project-team KV)
```

A project-scope value overrides everything. A global value overrides only the default.

## Template syntax (MiniJinja)

SKILL.md is rendered as a MiniJinja (Jinja2-compatible) template. The engine is pure WASM with no file loader and no custom functions.

### Substitution

Use double braces for variable substitution: `{{ variable_name }}` and `{{ object_var.field }}`.

### Conditionals

Use `if`/`elif`/`else`/`endif` blocks to branch on variable values. Example: `if sign_commits` renders signing instructions only when the variable is true. Equality checks like `commit_style == "gitmoji"` select format-specific content.

### Loops

Use `for`/`endfor` to iterate arrays. Example: `for author in co_authors` renders a `Co-Authored-By` trailer for each entry.

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

Undefined variables render as empty string (chainable undefined behavior). Template syntax errors cause the engine to fall back to the original unresolved content. A warning is printed to stderr.

## CLI commands

The `ship vars` CLI reads and writes variable state with type validation.

| Command | Description |
|---------|-------------|
| `ship vars get <skill-id>` | Show merged state for all variables (defaults + user overrides) |
| `ship vars get <skill-id> <key>` | Show merged value for a single variable |
| `ship vars set <skill-id> <key> <value>` | Set a variable value. Routes to the correct scope based on `storage-hint`. Validates type and enum constraints. |
| `ship vars append <skill-id> <key> '<json>'` | Append a value to an array variable |
| `ship vars reset <skill-id>` | Clear all user state for a skill, reverting to defaults |

Type validation happens on `set`. Enum values are checked against the `values` list. Bool accepts `true`/`false`.

## MCP tools

Agents interact with variables through these MCP tools. They use the same validation and storage paths as the CLI.

| Tool | Parameters | Description |
|------|-----------|-------------|
| `get_skill_vars` | `skill_id` | Returns merged variable state for a skill |
| `set_skill_var` | `skill_id`, `key`, `value` | Writes a single variable value |
| `list_skill_vars` | (none) | Lists all skills that have configured variables |

Two additional tools manage skill files on disk:

| Tool | Parameters | Description |
|------|-----------|-------------|
| `write_skill_file` | `skill_id`, `path`, `content` | Write a file into a skill directory |
| `delete_skill_file` | `skill_id`, `path` | Delete a file from a skill directory |

## Content hashing

When publishing, Ship hashes the skill directory tree (SKILL.md, assets/, references/, evals/). User variable state in `platform.db` KV is explicitly excluded from the hash. This means:

- Changing SKILL.md content or vars.json schema changes the publish hash (new version).
- A user setting their variable values does NOT change the hash (not a new version).

## Planned features

**Planned -- not yet available in stable releases.**

- **Declarative migrations**: A `migrations.json` file in skill assets with JSON ops (rename, set_default, delete, change_type) applied on `ship install`/`ship update`.
- **Computed/dynamic vars**: Environment variable injection (`{{ env.ANTHROPIC_MODEL }}`), git context (`{{ git.branch }}`), agent-written state via MCP.
- **Compile-time enum validation**: Currently enum values are only validated on `ship vars set`. Compile-time validation is planned.
