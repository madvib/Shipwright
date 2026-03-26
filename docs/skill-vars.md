# Stateful Skills — Variable Schema

Skill variables let skill authors define named parameters that users can customize without forking the skill. Values are stored in `platform.db` KV and resolved into skill content at compile time.

## How it works

```
assets/vars.json    +   KV state          →   compiler   →   resolved SKILL.md
(schema + defaults)     (user overrides)
```

1. Declare variables in `assets/vars.json` alongside `SKILL.md`.
2. Users set values with `ship vars set` (CLI), through the Studio UI, or by asking the agent.
3. At compile time (`ship use` / `ship compile`), Ship merges state and resolves MiniJinja template markers in `SKILL.md` before writing provider outputs.

Variables are only supported for directory-format skills. Flat-format `.ship/skills/{id}.md` files do not support vars.

---

## Directory layout

```
.ship/skills/my-skill/
  SKILL.md              ← agent instructions (MiniJinja template)
  assets/
    vars.json           ← variable schema and defaults
  references/
    docs/               ← human + agent-readable documentation (.mdoc)
```

---

## assets/vars.json

```json
{
  "$schema": "https://agentskills.io/schemas/vars/v1.json",
  "commit_style": {
    "type": "enum",
    "default": "conventional",
    "storage-hint": "global",
    "values": ["conventional", "gitmoji", "angular"],
    "label": "Commit style",
    "description": "Format applied to every commit message"
  },
  "sign_commits": {
    "type": "bool",
    "default": false,
    "storage-hint": "project",
    "label": "Sign commits"
  },
  "co_authors": {
    "type": "array",
    "storage-hint": "local",
    "label": "Co-authors"
  }
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | no | `string` (default), `bool`, `enum`, `array`, `object` |
| `default` | no | Default value when no state exists |
| `storage-hint` | no | `global` (default), `local`, or `project` |
| `values` | enum only | Allowed values; `ship vars set` rejects anything not in this list |
| `label` | no | Human-readable name shown in Studio and `ship vars get` |
| `description` | no | Longer explanation shown in Studio and docs |

The `$schema` key is ignored at parse time.

---

## Storage

All state lives in `platform.db` KV. No files. Three scopes:

| Hint | KV namespace | Semantics |
|------|-------------|-----------|
| `global` | `skill_vars:{skill_id}` | Machine-wide. Follows the user across all contexts. Use for personal preferences: commit style, terminal choice, preferred language. |
| `local` | `skill_vars.local:{ctx}:{skill_id}` | This context only, not shared. Personal overrides within a project. |
| `project` | `skill_vars.project:{ctx}:{skill_id}` | This context, intended to be shared with the team. |

`{ctx}` is a stable hex token derived from the project path, scoping local and project state to a specific context without embedding the path in the key.

### Merge order

```
1. defaults      (assets/vars.json)
2. global state  (platform.db, machine-wide)
3. local state   (platform.db, this context)
4. project state (platform.db, this context)
```

Later layers win.

---

## stable-id

Skills are identified by their directory name by default. If a skill is renamed, stored state is orphaned. Add `stable-id` to `SKILL.md` frontmatter to preserve state across renames:

```yaml
---
name: My Skill
stable-id: commit
---
```

The `stable-id` is used as the storage key instead of the directory name. Must be lowercase letters, digits, and hyphens only.

---

## Template syntax

Uses MiniJinja (Jinja2-compatible). Write `{{ var }}` markers in `SKILL.md`.

```
Write commit messages in {{ commit_style }} format.

{% if commit_style == "gitmoji" %}
Start every message with the appropriate emoji.
{% endif %}

{% if sign_commits %}
Sign every commit with -S.
{% endif %}

{% for author in co_authors %}
Co-Authored-By: {{ author }}
{% endfor %}
```

Undefined variables render as empty string. Template syntax errors fall back to the original content with a warning to stderr.

---

## CLI reference

```bash
ship vars get commit                    # show all vars (merged state)
ship vars get commit commit_style       # single var
ship vars set commit commit_style gitmoji
ship vars append commit co_authors '"Alice <alice@example.com>"'
ship vars reset commit                  # clear all state, revert to defaults
```

`ship vars set` validates type and, for enum vars, the values list.

---

## Documentation (`references/docs/`)

Rich documentation for a skill lives in `references/docs/` as Markdoc (`.mdoc`) files. This content is:

- **Human-readable** — rendered by the Ship documentation site
- **Agent-discoverable** — exposed as MCP resources, retrieved on demand without consuming context window

The main page is `references/docs/index.mdoc`. Additional pages (`examples.mdoc`, `reference.mdoc`, etc.) are linked from it.

This keeps `SKILL.md` focused on concise agent instructions while richer explanations and examples live where they can be retrieved when actually needed.

---

## Gotchas

**Compile-time only.** Values are baked in at `ship use` / `ship compile`. Changes take effect on next compile.

**Flat-format skills have no vars.** `{{ var }}` in `.ship/skills/{id}.md` is never resolved.

**Enum validation at CLI and compile time.** `ship vars set` rejects unknown values. At compile time, invalid enum values warn to stderr but still compile.

**Global scope is machine-wide.** `storage-hint: global` means the same value applies across all projects on this machine. Use `local` or `project` for per-context values.
