---
title: Skill Authoring Guide
description: Complete reference for creating smart skills with typed variables, documentation, and evals.
---

# Skill Authoring Guide

Smart skills are agent instructions that adapt to their context. A single skill produces different output for each user, project, and configuration — same skill, personalized behavior.

## How Smart Skills Work

```
SKILL.md (MiniJinja template)
assets/vars.json (schema + defaults)      ->  ship compile  ->  resolved provider output
platform.db KV (user state)
```

1. You declare variables in `assets/vars.json` with types, defaults, and storage scopes.
2. You reference them in `SKILL.md` using `{{ var }}`, `{% if %}`, `{% for %}`.
3. Users set values with `ship vars set`, through Studio, or by asking the agent.
4. `ship use` / `ship compile` merges state and resolves the template before writing provider outputs.

## Variable Resolution

Ship merges variable state from four layers. Last wins.

| Layer | Source | Wins over |
|-------|--------|-----------|
| defaults | `assets/vars.json` `default` field | nothing |
| global | `platform.db` KV `skill_vars:{id}` | defaults |
| local | `platform.db` KV `skill_vars.local:{ctx}:{id}` | global |
| project | `platform.db` KV `skill_vars.project:{ctx}:{id}` | local |

`{ctx}` is a stable hex token derived from the project path. It scopes local/project state without exposing the path in storage keys.

## CLI Reference

```bash
# Read merged state for a skill
ship vars get <skill-id>

# Read a single variable
ship vars get <skill-id> <key>

# Set a variable (routes to correct scope based on storage-hint)
ship vars set <skill-id> <key> <value>

# Append to an array variable
ship vars append <skill-id> <key> '<json-value>'

# Reset all state to defaults
ship vars reset <skill-id>
```

Type validation happens on `set`. Enum values are checked against the `values` list. Bool accepts `true`/`false`.

## MiniJinja Template Reference

Ship uses MiniJinja (Jinja2-compatible) for template resolution. No file loader, no custom functions.

### Substitution

```
{{ variable_name }}
{{ object.field }}
{{ deeply.nested.path }}
```

Undefined variables render as empty string (chainable undefined behavior).

### Conditionals

```
{% if flag %}
Content when flag is truthy.
{% endif %}

{% if style == "gitmoji" %}
Use emoji prefixes.
{% elif style == "conventional" %}
Use type(scope): prefixes.
{% else %}
Use default format.
{% endif %}
```

### Loops

```
{% for item in items %}
- {{ item }}
{% endfor %}

{% for person in team %}
- {{ person.name }} ({{ person.role }})
{% endfor %}
```

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

## vars.json Schema

Every key in `vars.json` (except `$schema`) defines one variable.

```json
{
  "$schema": "https://getship.dev/schemas/vars.schema.json",
  "variable_name": {
    "type": "enum",
    "default": "value",
    "storage-hint": "global",
    "values": ["value", "other"],
    "label": "Human Name",
    "description": "What this controls and why someone would change it."
  }
}
```

### Required fields

None are technically required. A bare `{}` entry creates a string variable with no default.

### All fields

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | `string` (default), `bool`, `enum`, `array`, `object` |
| `default` | any | Value used when no state is set. Type must match `type`. |
| `storage-hint` | string | `global` (default), `local`, `project` |
| `values` | array | Enum only. Allowed values. Validated on set and compile. |
| `label` | string | Human-readable name. Shown in Studio UI and docs. |
| `description` | string | Longer explanation. Shown in Studio hover and docs site. |

## Evals Specification

Evals live in `evals/evals.json`. Each eval case is a simulated user interaction.

```json
{
  "evals": [
    {
      "id": "eval-kebab-case-name",
      "prompt": "What a user would actually type",
      "expected": "Description of what good output looks like",
      "assertions": [
        "Specific verifiable claim about the output"
      ],
      "vars": { "optional": "override values for this eval" },
      "input_files": ["optional/paths/relative/to/workspace"]
    }
  ]
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | yes | Unique kebab-case identifier |
| `prompt` | yes | Realistic user message |
| `expected` | yes | Human-readable success description |
| `assertions` | no | Verifiable statements for grading |
| `vars` | no | Variable overrides for this eval case |
| `input_files` | no | Files the skill needs in the eval workspace |

## Content Hashing

When publishing, Ship hashes the skill directory tree (SKILL.md, assets/, references/, evals/). User variable state in `platform.db` KV is never included in the hash. This means:

- Changing SKILL.md content changes the publish hash (new version)
- Changing vars.json schema changes the publish hash (new version)
- A user setting their vars does NOT change the hash (not a new version)

## stable-id

Add `stable-id` to frontmatter to decouple the storage key from the directory name:

```yaml
---
stable-id: my-skill
---
```

All KV state is keyed by `stable-id`. If you rename the directory, state follows. Must match `[a-z0-9][a-z0-9\-]*`.
