---
title: Writing Docs — Reference Guide
description: Complete reference for writing skill documentation in Markdoc format with audience-aware frontmatter.
---

# Writing Docs Reference

Ship skill documentation lives in `references/docs/` inside each skill directory. The docs site at docs.getship.dev collects these files and renders them as browsable pages. Agents read the same files from the filesystem for on-demand context.

## Progressive Disclosure Model

```
SKILL.md              Always loaded in agent context. Concise, imperative.
                      Points to docs when depth is needed.

references/docs/
  index.md            Retrieved on demand. Human landing page + agent overview.
  commands.md         Retrieved on demand. Specific reference section.
  patterns.md         Retrieved on demand. Usage examples.
```

**Rule**: If the agent needs it every time, put it in SKILL.md. If the agent needs it sometimes, put it in a doc page. If only humans need it, put it in a doc page with `audience: public`.

## Frontmatter Fields

Every `.md` or `.mdoc` file in `references/docs/` should have YAML frontmatter.

| Field | Required | Type | Default | Description |
|-------|----------|------|---------|-------------|
| `title` | yes | string | — | Page title. Shown in sidebar and page header. |
| `description` | no | string | — | One-line summary. Shown in search results and meta tags. |
| `audience` | no | enum | `public` | Controls where the page is visible. See Audience Values. |
| `section` | no | enum | — | Groups pages in the docs site sidebar. See Section Values. |
| `order` | no | number | — | Sort position within the skill's section. Lower = higher. |

### Audience Values

| Value | Docs site | Agent reads | Use for |
|-------|-----------|-------------|---------|
| `public` | yes | yes | User-facing docs, tutorials, references |
| `internal` | no | yes | Team conventions, internal tooling notes |
| `agent-only` | no | yes | Prompts, calibration data, context the agent needs but humans don't browse |

When `audience` is absent, defaults to `public`.

### Section Values

| Value | Sidebar icon | Use for |
|-------|-------------|---------|
| `guide` | book | How-to walkthroughs ("How to deploy") |
| `reference` | table | Parameter tables, command lists, API specs |
| `tutorial` | steps | Step-by-step with expected outcomes |
| `concepts` | lightbulb | Background, architecture, design rationale |

When `section` is absent, the page appears ungrouped in the sidebar.

## Multi-Page Structure

### Minimal (single page)

```
references/docs/
  index.md
```

For simple skills. One page covers everything.

### Standard (3-5 pages)

```
references/docs/
  index.md              <- order: 1, section: guide
  commands.md           <- order: 2, section: reference
  patterns.md           <- order: 3, section: guide
  troubleshooting.md    <- order: 4, section: guide
```

### Comprehensive

```
references/docs/
  index.md              <- overview
  getting-started.md    <- section: tutorial
  commands.md           <- section: reference
  api.md                <- section: reference
  architecture.md       <- section: concepts
  patterns.md           <- section: guide
  migration.md          <- section: guide
  internal-notes.md     <- audience: internal
```

## Writing Guidelines

### For the index page

The index page is the entry point for both humans and agents. It should:

1. Explain what the skill does in 2-3 sentences
2. List the key capabilities
3. Show one quick example
4. Link to other doc pages for depth

### For reference pages

Use tables. Agents parse tables efficiently. Humans scan them quickly.

```markdown
| Command | Description | Example |
|---------|-------------|---------|
| `goto <url>` | Navigate to URL | `$B goto https://app.com` |
| `click <sel>` | Click element | `$B click @e3` |
```

### For guide pages

Use numbered steps with code blocks. Show what to type and what to expect.

```markdown
## Deploy to staging

1. Build the project:
   ```bash
   ship build --target staging
   ```

2. Verify the output:
   ```bash
   ls dist/
   ```

3. Deploy:
   ```bash
   ship deploy staging
   ```
```

### For concept pages

Explain the "why" — architecture decisions, trade-offs, design rationale. These pages are less structured but should still use headings as retrieval anchors.

## Example: Complete Doc Page

```markdown
---
title: Variable Reference
description: All configuration variables for the deploy skill.
audience: public
section: reference
order: 2
---

# Variables

The deploy skill uses variables to configure target environments
and deployment behavior.

| Variable | Type | Scope | Default | Description |
|----------|------|-------|---------|-------------|
| `target` | enum | project | `staging` | Deploy target |
| `dry_run` | bool | global | `false` | Preview without deploying |
| `notify` | array | project | `[]` | Slack channels to notify |

## Setting variables

\`\`\`bash
ship vars set deploy target production
ship vars set deploy dry_run true
ship vars append deploy notify "#releases"
\`\`\`

## Scope meanings

- **global** — follows you across all projects
- **project** — shared with the team via `.ship/`
- **local** — personal override, not shared
```
