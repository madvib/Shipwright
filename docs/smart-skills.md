# Smart Skills

Smart Skills are skills with typed, scoped configuration variables that resolve into content at compile time. A smart skill adapts its instructions to the user and context — same skill, different output for everyone.

## How it works

```
SKILL.md (MiniJinja template)
assets/vars.json (schema + defaults)   →   ship compile   →   resolved provider output
platform.db KV (user state)
```

1. Declare variables in `assets/vars.json`.
2. Reference them in `SKILL.md` with `{{ var }}`, `{% if %}`, `{% for %}`.
3. Users set values with `ship vars set`, through Studio, or by asking the agent.
4. `ship use` / `ship compile` merges state and resolves the template before writing provider outputs.

---

## Full directory layout

```
.ship/skills/my-skill/
  SKILL.md              ← agent instructions (MiniJinja template)
  assets/
    vars.json           ← variable schema and defaults
    templates/          ← reusable config snippets
  scripts/              ← helper scripts referenced in SKILL.md
  references/
    docs/               ← human + agent-readable documentation (.mdoc, Markdoc)
    api/                ← API tables, external specs
  evals/
    evals.json          ← eval test cases (prompts, expected outputs, assertions)
```

---

## assets/vars.json schema

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
| `default` | no | Value used when no state exists |
| `storage-hint` | no | `global` (default), `local`, or `project` |
| `values` | enum only | Allowed values; CLI and compile-time validation enforced |
| `label` | no | Human-readable name (Studio, `ship vars get`) |
| `description` | no | Longer explanation (Studio, docs site) |

### Storage hints

All state lives in `platform.db` KV. No files.

| Hint | KV namespace | Semantics |
|------|-------------|-----------|
| `global` | `skill_vars:{id}` | Machine-wide. Follows the user across all contexts. |
| `local` | `skill_vars.local:{ctx}:{id}` | This context only, not shared. Personal override. |
| `project` | `skill_vars.project:{ctx}:{id}` | This context, intended to be shared with the team. |

`{ctx}` is a stable hex token derived from the project path.

**Merge order:** defaults → global → local → project (last wins).

---

## stable-id

Add `stable-id` to `SKILL.md` frontmatter to preserve state across skill renames:

```yaml
---
name: My Skill
stable-id: commit
---
```

The `stable-id` is used as the storage key. Must be `[a-z0-9][a-z0-9\-]*`.

---

## Template syntax (MiniJinja)

```
Write commit messages in {{ commit_style }} format.

{% if commit_style == "gitmoji" %}
Start every message with the appropriate emoji.
{% endif %}

{% for author in co_authors %}
Co-Authored-By: {{ author }}
{% endfor %}
```

Undefined variables render as empty string. Template errors fall back to original content with a warning to stderr.

---

## CLI

```bash
ship vars get commit                          # merged state (defaults + user overrides)
ship vars get commit commit_style             # single var
ship vars set commit commit_style gitmoji     # set (validates type + allowed values)
ship vars append commit co_authors '"Alice <alice@example.com>"'
ship vars reset commit                        # clear all state, revert to defaults
```

---

## references/docs/

Rich documentation lives in `references/docs/` as Markdoc (`.mdoc`) files. The main page is `index.mdoc`.

- **Human-readable**: rendered by the Ship documentation site
- **Agent-discoverable**: exposed as MCP resources, retrieved on demand without consuming context window

This keeps `SKILL.md` focused on concise agent instructions. Richer explanations and examples live in docs where they can be retrieved when needed.

---

## evals/evals.json

Every skill should have an eval suite. Evals measure whether the skill produces reliably better outputs than no skill, and give a feedback loop for iterating.

See the full evaluation methodology at [agentskills.io/skill-creation/evaluating-skills](https://agentskills.io/skill-creation/evaluating-skills).

### Format

```json
{
  "evals": [
    {
      "id": "eval-basic-conventional",
      "prompt": "I fixed a null pointer crash in the auth module",
      "expected": "A conventional commit message: fix(auth): handle null pointer in ...",
      "assertions": [
        "Message starts with 'fix(' or 'fix:'",
        "Message is one line under 72 characters",
        "No period at the end of the subject"
      ]
    },
    {
      "id": "eval-edge-breaking-change",
      "prompt": "I renamed the login() function to authenticate() across the whole codebase",
      "expected": "A commit message that signals a breaking change",
      "assertions": [
        "Message includes 'BREAKING CHANGE' footer or '!' after type",
        "Message describes what changed, not just that it changed"
      ]
    }
  ]
}
```

### Fields

| Field | Required | Description |
|-------|----------|-------------|
| `id` | yes | Unique identifier for the eval case. Kebab-case. |
| `prompt` | yes | Realistic user message — the kind of thing someone would actually type. |
| `expected` | yes | Human-readable description of what success looks like. |
| `assertions` | no | Verifiable statements about the output. Add after seeing first results. |
| `input_files` | no | Files the skill needs to work with (paths relative to eval workspace). |

### Workspace structure

Eval results live outside the skill directory, in `{skill-id}-workspace/iteration-N/`:

```
commit-workspace/
  iteration-1/
    eval-basic-conventional/
      with_skill/
        outputs/          ← files produced by the run
        timing.json       ← { total_tokens, duration_ms }
        grading.json      ← { assertion_id: "PASS"|"FAIL", evidence: "..." }
      without_skill/
        outputs/
        timing.json
        grading.json
    benchmark.json        ← aggregated pass rates, token/time deltas
```

### Running evals

Each run starts with a clean context (no state from previous runs). Run each eval twice — once with the skill, once without — to get a baseline delta. `benchmark.json` captures the delta: what the skill costs (tokens, time) versus what it buys (pass rate improvement).

`ship skill eval` tooling is planned. Until then, run manually or with a subagent per eval case.
