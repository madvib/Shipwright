---
name: writing-docs
stable-id: writing-docs
description: Use when creating or updating reference documentation for a skill. Covers Markdoc format, frontmatter conventions, progressive disclosure, and multi-page structure.
tags: [skills, documentation, authoring]
authors: [ship]
---

# Writing Docs

Write documentation in `references/docs/` inside a skill directory. Every doc page serves two audiences: humans browsing the docs site and agents retrieving context on demand.

## Where docs live

```
.ship/skills/{id}/
  SKILL.md                    <- always in agent context (concise)
  references/docs/
    index.md                  <- landing page (always create first)
    commands.md               <- one concern per page
    patterns.md
```

SKILL.md is the pointer. Keep it under 100 lines. Reference docs are the library — retrieved when the agent needs depth.

## Frontmatter

Every doc page starts with YAML frontmatter:

```yaml
---
title: Command Reference
description: Complete list of commands and options.
audience: public
section: reference
order: 2
---
```

Required: `title`. Recommended: `description`. Optional: `audience`, `section`, `order`.

### Audience values

- `public` (default) — rendered on docs site, readable by agents
- `internal` — agents can read, hidden from public site
- `agent-only` — never on site, only for agent filesystem reads

### Section values

- `guide` — how-to walkthroughs
- `reference` — tables, parameters, options
- `tutorial` — step-by-step with expected outcomes
- `concepts` — background, architecture, rationale

## Writing for two audiences

Write content that works for both a human scanning a docs page and an agent retrieving specific context.

- Use tables for reference data (commands, parameters, options)
- Use code blocks for examples — agents parse these directly
- Use headings as retrieval anchors — agents search by heading
- Keep pages focused — one concern per page, not one page per skill
- Index page summarizes the skill and links to sections

## File format

Use `.md` (Markdown) or `.mdoc` (Markdoc) extension. Markdoc supports custom tags rendered by Starlight at build time.

Retrieve the full frontmatter reference from `references/docs/index.md` when you need field details.
