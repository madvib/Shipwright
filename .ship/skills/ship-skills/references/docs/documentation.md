---
group: Smart Skills
title: Skill Documentation
description: How references/docs/ works, frontmatter fields, audience values, and writing for humans and agents.
audience: public
section: reference
order: 4
---

# Skill Documentation

Documentation lives in `references/docs/` inside the skill directory. It serves two audiences from the same source: humans browsing a docs site and agents retrieving context on demand through filesystem reads.

## Progressive disclosure

Smart skills use two tiers:

1. **SKILL.md** -- always loaded into the agent's context window. Concise working instructions. Under 100 lines.
2. **references/docs/** -- retrieved on demand. Detailed reference material, examples, troubleshooting.

This keeps the context window lean while making deep information available. The agent reads specific doc pages only when it needs depth.

## Directory layout

```
references/docs/
  index.md              <- landing page
  commands.md           <- one concern per page
  patterns.md
  troubleshooting.md
```

`index.md` is the entry point. It summarizes the skill and links to other pages. Additional pages each cover a single concern.

## File format

Use `.md` (Markdown) or `.mdoc` (Markdoc). Markdoc files support custom tags rendered at docs site build time. For most skills, plain Markdown is sufficient.

## Frontmatter

Every doc page starts with YAML frontmatter:

```yaml
---
group: My Skill Group
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
| `group` | string | yes | -- | Sidebar group in the docs site. All pages in a skill typically share the same group value. |
| `title` | string | yes | -- | Page title. Displayed in sidebar and as page heading. |
| `description` | string | no | -- | One-line summary for meta tags and search. |
| `audience` | string | no | `"public"` | Controls visibility. See audience values below. |
| `section` | string | no | -- | Grouping hint for sidebar organization. |
| `order` | number | no | -- | Sort position within the group. Lower numbers appear first. |

### Audience values

| Value | Docs site | Agent reads | Use for |
|-------|-----------|-------------|---------|
| `public` | visible | yes | General documentation for everyone |
| `internal` | hidden | yes | Notes for agents, not for human readers |
| `agent-only` | hidden | yes | Context meaningful only during agent execution |

The default is `public`. Most pages should be public. Use `internal` for implementation notes that help agents but would confuse humans. Use `agent-only` for context that is only meaningful during agent work.

### Section values

Sections group pages in the docs site sidebar:

| Section | Use for |
|---------|---------|
| `concepts` | Background, architecture, rationale |
| `guide` | How-to walkthroughs for common tasks |
| `reference` | Tables, parameters, CLI commands, API surfaces |
| `tutorial` | Step-by-step instructions with expected outcomes |

## The docs site convention

The Ship docs site (Astro/Starlight) collects `references/docs/` pages from all installed skills at build time. The `group` frontmatter field determines the sidebar section. Pages within a group are sorted by `order`.

This means skill documentation is not a separate authoring step -- the same files that agents read from the filesystem are the pages humans see on the site. Write once, serve both audiences.

## Writing for two audiences

### Use tables for reference data

Agents parse tables reliably. Humans scan them quickly. Use tables for commands, parameters, options, and field definitions.

### Use code blocks for examples

Both audiences benefit from concrete examples. Agents follow examples more reliably than abstract descriptions. Show the command, the file content, the expected output.

### Use headings as retrieval anchors

Agents search doc pages by heading. Make headings specific: "Storage scopes" over "Details", "CLI commands" over "Usage".

### Keep pages focused

One concern per page. A page about commands should not also cover architecture. A page about variables should not also cover eval writing.

## What belongs where

| Content | Location |
|---------|----------|
| Direct instructions the agent follows every time | SKILL.md |
| Quick variable summary (3-5 lines) | SKILL.md |
| Links to reference pages | SKILL.md |
| Variable reference tables | references/docs/ |
| Setup and installation guides | references/docs/ |
| Detailed examples and walkthroughs | references/docs/ |
| Background context and rationale | references/docs/ |
| Troubleshooting guides | references/docs/ |

## Index page structure

The index page should:

1. State what the skill does in one sentence.
2. Explain the key concept or approach.
3. List major topics with brief descriptions.
4. Link to relevant reference pages.
