---
name: skill-authoring
stable-id: skill-authoring
description: Use when creating or improving agent skills. Guides smart skill structure, vars.json schema, MiniJinja templating, reference docs, and evals.
tags: [skills, authoring, meta]
authors: [ship]
---

# Skill Authoring

Build skills that adapt to their user. A smart skill is a MiniJinja template with typed variables, reference documentation, and eval test cases.

## Skill Structure

{% if output_format == "directory" %}
Every publishable skill is a directory:

```
.ship/skills/{id}/
  SKILL.md              <- agent instructions (MiniJinja template)
  assets/
    vars.json           <- variable schema and defaults
  references/
    docs/
      index.md          <- human + agent readable documentation
  evals/
    evals.json          <- test cases (prompt, expected, assertions)
```

Create all four files. `SKILL.md` is the agent-facing instructions. Everything else supports it.
{% elif output_format == "single-file" %}
Minimal skills are a single file:

```
.ship/skills/{id}/SKILL.md
```

Add `assets/vars.json` when the skill needs user configuration. Add `references/docs/` when the skill needs more context than fits in SKILL.md. Add `evals/` when the skill produces verifiable output.
{% endif %}

## SKILL.md Frontmatter

Every SKILL.md starts with YAML frontmatter:

```yaml
---
name: my-skill
stable-id: my-skill
description: One sentence. Starts with "Use when..." to help trigger matching.
tags: [category, subcategory]
authors: [you]
---
```

- `stable-id` is the storage key for vars. Must be `[a-z0-9][a-z0-9\-]*`. Survives renames.
- `description` is how the skill gets matched to user intent. Be specific about the trigger.
- `tags` help discovery. Use existing tags before inventing new ones.

## Writing Instructions

SKILL.md is what the agent reads. Write it as direct instructions, not documentation.

**Do:**
- Imperative voice: "Write the test first", not "Tests should be written first"
- Concrete examples over abstract rules
- One concern per skill — compose multiple skills, don't build monoliths
- Keep it under 200 lines — agents lose focus in long prompts

**Don't:**
- Repeat what the agent already knows (general coding practices)
- Include setup instructions the user runs once (put those in references/docs/)
- Add conditional sections for rare edge cases (handle in evals instead)

## Variables (assets/vars.json)

Variables make the same skill work differently for different users and projects.

```json
{
  "$schema": "https://getship.dev/schemas/vars.schema.json",
  "my_var": {
    "type": "enum",
    "default": "option_a",
    "storage-hint": "global",
    "values": ["option_a", "option_b"],
    "label": "Human-readable name",
    "description": "Shown in Studio and docs. Explain what this controls."
  }
}
```

### Types

| Type | JSON value | Use for |
|------|-----------|---------|
| `string` | `"text"` | Paths, names, free-form config |
| `bool` | `true`/`false` | Feature toggles |
| `enum` | `"value"` | Fixed choices (must declare `values`) |
| `array` | `["a", "b"]` | Lists (co-authors, ignore patterns) |
| `object` | `{...}` | Structured config (rarely needed) |

### Storage hints

| Hint | Scope | Use for |
|------|-------|---------|
| `global` | Machine-wide, all projects | Personal preferences (style, format) |
| `project` | This project, shareable | Team settings (conventions, tool config) |
| `local` | This project, personal only | Individual overrides |

Merge order: defaults -> global -> local -> project (last wins).

### Template syntax

Reference vars in SKILL.md with MiniJinja:

```
Use {{ my_var }} format.

{% if my_var == "option_a" %}
Specific instructions for option A.
{% endif %}

{% for item in my_list %}
- {{ item }}
{% endfor %}
```

Undefined vars render as empty string. Template errors fall back to original content.

## Reference Docs (references/docs/)

`references/docs/index.md` is the main documentation page. Additional pages cover specific concerns. Same source serves humans (docs site) and agents (filesystem reads).

A skill can have multiple doc pages:

```
references/docs/
  index.md              <- overview (always create this first)
  commands.md           <- command reference
  patterns.md           <- usage patterns
```

Put here what doesn't belong in SKILL.md:
- Setup and installation guides
- Detailed examples and walkthroughs
- Variable reference tables
- Background context and rationale

### Doc frontmatter

```yaml
---
title: Page Title
description: One-line summary.
audience: public
section: reference
order: 1
---
```

- `audience`: `public` (default) = docs site + agents. `internal` = agents only. `agent-only` = never on site.
- `section`: `guide`, `reference`, `tutorial`, `concepts` — groups pages in the sidebar.
- `order`: sort position within the skill's doc section.

{% if include_evals %}
## Evals (evals/evals.json)

Evals measure whether the skill reliably improves agent output.

```json
{
  "evals": [
    {
      "id": "eval-descriptive-name",
      "prompt": "Realistic user message — what someone would actually type",
      "expected": "Human-readable description of what success looks like",
      "assertions": [
        "Specific, verifiable statement about the output",
        "Another verifiable statement"
      ]
    }
  ]
}
```

### Writing good evals

- `prompt` is a realistic user message, not a test instruction
- `expected` describes the outcome, not the implementation
- `assertions` are specific enough to grade programmatically
- Include `vars` field to test skill behavior with different variable values
- Test the happy path AND the interesting edge cases
- 5-8 evals is a good starting set

### Evals with vars

Test that template branching works:

```json
{
  "id": "eval-with-custom-vars",
  "prompt": "Same prompt as default case",
  "expected": "Different expected output because vars changed behavior",
  "vars": { "my_var": "option_b" }
}
```
{% endif %}

## Publishing Checklist

Before `ship publish`:

1. `SKILL.md` has valid frontmatter with `stable-id` and `description`
2. `assets/vars.json` has `$schema`, every var has `label` and `description`
3. `references/docs/index.md` exists with title and description frontmatter
4. Evals cover the main use case and at least one var-driven branch
5. Run `ship use` and verify the resolved output reads correctly
6. Run evals and confirm pass rate > baseline (no skill)
