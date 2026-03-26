---
title: Ship Docs
description: Documentation for Ship skills — declarative AI agent configuration that compiles to provider-specific outputs.
---

# Ship skills

Ship is a compiler and runtime for AI agent configuration. You write a declarative `.ship/` directory; Ship compiles it into provider-specific outputs (CLAUDE.md, `.cursor/`, `.mcp.json`, and more).

**Skills** are the building block. Each skill is a focused set of agent instructions for one task — committing code, writing tests, reviewing PRs. Skills are Markdown files with optional MiniJinja templating, so they adapt to each user and project without forking.

## Smart Skills

A Smart Skill has typed, scoped configuration variables declared in `assets/vars.json`. Variables are set once per user or project and resolved at compile time:

```
SKILL.md (MiniJinja template)
assets/vars.json (schema + defaults)   →   ship compile   →   provider output
platform.db KV (user state)
```

The same skill can produce a conventional-commits commit message for one user and a gitmoji commit message for another — no separate skill needed.

## How to install a skill

```bash
ship install <skill-id>
ship use
```

`ship use` compiles your `.ship/` directory and writes provider configs. Re-run it whenever you change variables or add skills.

## How to configure a skill

```bash
# See available variables and current values
ship vars get <skill-id>

# Set a variable
ship vars set <skill-id> <variable> <value>

# Append to an array variable
ship vars append <skill-id> <variable> <value>

# Reset to defaults
ship vars reset <skill-id>
```

## Skills in this docs site

The pages under **Skills** are generated automatically from each skill's `references/docs/` directory. The canonical source for each page is the skill itself — the docs site is a read-only view.
