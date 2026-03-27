---
title: Skill Documentation
description: How to write progressive disclosure documentation for smart skills using references/docs/.
audience: public
section: reference
order: 4
---

# Skill Documentation

Skill documentation lives in `references/docs/` inside the skill directory. It serves two audiences from the same source: humans browsing a docs site and agents retrieving context on demand through filesystem reads.

## The progressive disclosure model

Smart skills use a two-tier information architecture:

1. **SKILL.md** -- always loaded into the agent's context window. Concise working instructions. Under 100 lines.
2. **references/docs/** -- retrieved on demand when the agent needs depth. Detailed reference material, examples, troubleshooting.

This keeps the agent's context window lean while making deep information available. The agent gets its instructions automatically; when it needs more, it reads specific doc pages without permanently consuming context.

## Directory structure

```
references/docs/
  index.md              <- landing page (always create first)
  commands.md           <- one concern per page
  patterns.md
  troubleshooting.md
```

The `index.md` page is the entry point. It summarizes the skill and links to other pages. Additional pages each cover a single concern.

## File format

Use `.md` (Markdown) or `.mdoc` (Markdoc) extension. Markdoc files support custom tags that are rendered by Starlight at docs site build time. For most skills, plain Markdown is sufficient.

## Frontmatter

Every doc page starts with YAML frontmatter:

```yaml
---
title: Command Reference
description: Complete list of commands and their options.
audience: public
section: reference
order: 2
---
```

### Frontmatter fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `title` | string | yes | -- | Page title. Displayed in docs site sidebar and as page heading. |
| `description` | string | no | -- | One-line summary. Used in docs site meta tags and search results. |
| `audience` | string | no | `"public"` | Controls visibility. See audience values below. |
| `section` | string | no | -- | Grouping hint for the docs site sidebar. See section values below. |
| `order` | number | no | -- | Sort position within the skill's doc section. Lower numbers appear first. |

### Audience values

| Value | Docs site | Agent reads | Use for |
|-------|-----------|-------------|---------|
| `public` | yes | yes | General documentation visible to everyone |
| `internal` | no | yes | Internal notes for agents, hidden from the public site |
| `agent-only` | no | yes | Context exclusively for agent consumption, never rendered on site |

The default is `public`. Most doc pages should be public. Use `internal` for implementation notes that help agents but would confuse human readers. Use `agent-only` for context that is only meaningful during agent execution.

### Section values

Sections group pages in the docs site sidebar:

| Section | Use for |
|---------|---------|
| `concepts` | Background, architecture, rationale, "how it works" explanations |
| `guide` | How-to walkthroughs for common tasks |
| `reference` | Tables, parameters, options, CLI commands, API surfaces |
| `tutorial` | Step-by-step instructions with expected outcomes at each step |

## Writing for two audiences

Every doc page is read by both humans and agents. Write content that works for both.

### Use tables for reference data

Agents parse tables reliably. Humans scan them quickly. Use tables for commands, parameters, options, and field definitions.

### Use code blocks for examples

Both audiences benefit from concrete examples. Agents follow examples more reliably than abstract descriptions. Show the command, the file content, the expected output.

### Use headings as retrieval anchors

Agents search doc pages by heading. Make headings specific and descriptive. "Storage scopes" is better than "Details." "CLI commands" is better than "Usage."

### Keep pages focused

One concern per page, not one page per skill. A page about commands should not also cover architecture. A page about variables should not also cover eval writing.

### Index page structure

The index page should:

1. State what the skill does in one sentence.
2. Explain the key concept or innovation.
3. List the major topics with brief descriptions.
4. Link to the relevant reference pages.

## Multi-page organization

For skills with substantial documentation, organize pages by concern:

| Page | Content |
|------|---------|
| `index.md` | Overview, key concepts, navigation |
| `directory-structure.md` | File layout reference |
| `variables.md` | Variable schema, types, scopes, CLI |
| `documentation.md` | How to write docs (this pattern) |
| `commands.md` | CLI command reference |
| `troubleshooting.md` | Common issues and solutions |

Use the `order` frontmatter field to control sidebar sort order. Number sequentially starting from 1.

## What belongs in docs vs SKILL.md

| Content | Where it goes |
|---------|--------------|
| Direct instructions the agent follows every time | SKILL.md |
| Variable reference tables | references/docs/ |
| Setup and installation guides | references/docs/ |
| Detailed examples and walkthroughs | references/docs/ |
| Background context and rationale | references/docs/ |
| Troubleshooting guides | references/docs/ |
| Quick variable summary (3-5 lines) | SKILL.md |
| Links to reference pages | SKILL.md |
