# Skill Variables

Skill variables let skill authors define named parameters that users can customize without forking the skill. Variable values are stored in typed JSON state files and resolved into skill content at compile time.

## How it works

```
vars.json           state files              compiler output
(schema + defaults) + (user overrides)  →   resolved SKILL.md
```

1. The skill author defines variables in `vars.json` alongside `SKILL.md`.
2. Users set values with `ship vars set` or by editing state files directly.
3. At compile time (`ship use` / `ship compile`), Ship reads the merged state and resolves template markers in `SKILL.md` before writing the provider-specific output.

Variables are only supported for directory-format skills (`.ship/skills/{id}/SKILL.md`). Flat-format `.ship/skills/{id}.md` files do not support vars.

---

## vars.json format

Place `vars.json` next to `SKILL.md` in the skill directory.

```json
{
  "$schema": "https://agentskills.io/schemas/vars/v1.json",
  "commit_style": {
    "type": "enum",
    "default": "conventional",
    "scope": "user",
    "values": ["conventional", "gitmoji", "angular"],
    "label": "Commit message format",
    "description": "Applied to every commit in this project"
  },
  "verbose_output": {
    "type": "bool",
    "default": false,
    "scope": "project"
  },
  "team_members": {
    "type": "array",
    "scope": "project"
  }
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `type` | no | `string` (default), `bool`, `enum`, `array`, `object` |
| `default` | no | Default value used when no state file exists |
| `scope` | no | `user` (default) or `project` — controls which state file stores this var |
| `values` | enum only | Allowed values; `ship vars set` rejects anything not in this list |
| `label` | no | Human-readable name shown in Studio and `ship vars get` |
| `description` | no | Longer explanation of what the var controls |

The `$schema` key is ignored at parse time and exists only for IDE tooling.

---

## Scope

Each variable has a scope declared in `vars.json`. The scope is set by the skill author.

### User scope (`scope: user`)

State stored at `~/.ship/state/skills/{skill-id}.json`.

Use for personal preferences that follow the developer across all projects: commit message style, preferred language, output verbosity.

### Project scope (`scope: project`)

State stored at `.ship/state/skills/{skill-id}.json` (relative to the `.ship/` directory).

Use for team-wide configuration: team member lists, project-specific conventions.

**Important:** Project-scoped state files are only shared if `.ship/state/` is committed to version control. Consider adding it explicitly when using project-scoped vars.

### Merge order

```
1. defaults (vars.json)
2. user state  (~/.ship/state/skills/{id}.json)
3. project state (.ship/state/skills/{id}.json)
```

---

## Template syntax

Uses standard Jinja2 syntax via MiniJinja. Write `{{ var }}` markers in `SKILL.md`. Resolution happens at compile time.

### Scalar substitution

```
Use {{ commit_style }} commit messages.
```

### Dot-path into object

```
Primary contact: {{ owner.name }} ({{ owner.email }})
```

### Conditional block

```
{% if verbose_output %}
Include full reasoning in every response.
{% endif %}
```

```
{% if commit_style == "gitmoji" %}
Start every commit message with an emoji.
{% endif %}
```

With an else branch:

```
{% if commit_style == "conventional" %}
Use type(scope): subject format.
{% else %}
Use the format preferred by this project.
{% endif %}
```

### Loop over array

```
{% for member in team_members %}
- {{ member }}
{% endfor %}
```

With object elements:

```
{% for member in team_members %}
- {{ member.name }} <{{ member.email }}>
{% endfor %}
```

Conditionals nest inside loops:

```
{% for member in team_members %}
- {{ member.name }}{% if member.lead %} (lead){% endif %}
{% endfor %}
```

### Undefined variables

A `{{ var }}` marker with no value in state renders as **empty string**. The skill still compiles. Always provide sensible defaults in `vars.json` to avoid silent holes in the output.

Template syntax errors (malformed `{% if %}` etc.) fall back to the original unrendered content with a warning to stderr.

---

## State file format

State files are JSON objects mapping variable name to value, with a reserved `_meta` block:

```json
{
  "commit_style": "gitmoji",
  "team_members": ["Alice", "Bob"],
  "_meta": {
    "v": 1,
    "skill": "commit",
    "migrations": []
  }
}
```

`_meta` is written by Ship and stripped before vars reach the template engine. Do not edit it by hand.

Files are created on first write. Missing files are treated as empty (defaults apply). Unknown keys are silently ignored at read time.

### Changes log

Every write via `ship vars set` or `ship vars append` appends an entry to `.ship/state/skills/{id}.changes.jsonl`:

```jsonl
{"ts":"2026-03-25T14:00:00Z","key":"commit_style","from":"conventional","to":"gitmoji","actor":"user:cli"}
```

---

## CLI reference

```bash
# Show all var values for a skill (merged: defaults + user + project)
ship vars get commit

# Show a single var
ship vars get commit commit_style

# Set a value
ship vars set commit commit_style gitmoji

# Append to an array var (value must be valid JSON)
ship vars append commit team_members '"Alice"'

# Open the project state file in $EDITOR
ship vars edit commit

# Delete all state for a skill (resets to defaults on next compile)
ship vars reset commit
```

`ship vars set` validates against the declared type and, for `enum` vars, against the `values` list.

---

## Gotchas

**Compile-time resolution only.** Variables are baked into the output at `ship use` / `ship compile` time. Changing a state file has no effect until the next compile.

**Flat-format skills have no vars support.** `{{ var }}` markers in `.ship/skills/{id}.md` (single-file format) are never resolved. Use the directory format to get vars.

**Project state is only shared if committed.** `.ship/state/` is not automatically added to version control. Teams using project-scoped vars should commit this directory.

**Enum validation is enforced at the CLI and at compile time.** `ship vars set` rejects values not in the `values` list. At compile time (`ship use` / `ship compile`), invalid enum values produce a warning to stderr — the skill still compiles, using the stored value as-is.

---

## Example: commit style skill

```
.ship/skills/commit/
  SKILL.md
  vars.json
```

**vars.json:**
```json
{
  "commit_style": {
    "type": "enum",
    "default": "conventional",
    "scope": "user",
    "values": ["conventional", "gitmoji", "angular"],
    "label": "Commit style"
  },
  "include_scope": {
    "type": "bool",
    "default": true,
    "scope": "user",
    "label": "Include scope in commit subject"
  }
}
```

**SKILL.md:**
```markdown
Write commit messages in {{ commit_style }} format.

{% if commit_style == "conventional" %}
Format: `type(scope): subject` where type is feat, fix, docs, etc.
{% if include_scope %}Always include the scope in parentheses.{% endif %}
{% endif %}

{% if commit_style == "gitmoji" %}
Start every commit message with the appropriate gitmoji.
{% endif %}

{% if commit_style == "angular" %}
Follow the Angular commit message convention strictly.
{% endif %}
```

A user who prefers gitmoji runs:

```bash
ship vars set commit commit_style gitmoji
ship use
```

Their compiled skill reads `Write commit messages in gitmoji format.` followed by the gitmoji-specific instructions. Teammates who haven't set a preference get `conventional` from the default.
