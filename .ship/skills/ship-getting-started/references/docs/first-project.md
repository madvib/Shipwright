---
group: Getting Started
title: Your First Project
section: tutorial
order: 3
---

# Your First Project

## Initialize

Navigate to your project and run:

```bash
cd your-project
ship init
```

This creates a `.ship/` directory with a project manifest and default configuration.

## Open Studio

The fastest way to get going:

```bash
ship studio --open
```

Studio opens in your browser. From there you can:

- Browse and install skills from the registry
- Create and configure agent profiles
- Set skill variables (preferences that personalize behavior)
- See everything update in real time

## Or use the CLI

If you prefer the terminal:

```bash
# Install a skill from the registry
ship add github.com/some/skill-pack

# Create an agent profile
ship agents create my-agent

# Activate it
ship use my-agent
```

When you run `ship use`, Ship compiles your agent's full configuration — skills, MCP servers, permissions, rules — into whatever format your AI tool expects. Claude Code gets CLAUDE.md, Cursor gets .cursor/rules, and so on.

## Add skills

Skills teach your agent how to do specific things. Install them from the registry or create your own:

```bash
# Install from the registry
ship add github.com/some/tdd-skill

# Or create a local skill
ship skills create code-review
```

Each skill can have typed variables that you personalize:

```bash
ship vars set tdd test_runner vitest
ship vars set tdd commit_at_green false
```

These preferences are stored per-user and per-project. The same skill produces different output for different people.

## What happens under the hood

When you run `ship use`, Ship reads your `.ship/` directory, resolves everything, and writes provider-native config files. These output files are gitignored — they're build artifacts, not source.

You don't need to think about this most of the time. Install skills, configure agents, and let Ship handle the rest.

## Next steps

- [Agents](/reference/agents/) — create and configure agent profiles
- [Smart Skills](/reference/smart-skills/) — understand how skills and variables work
- [Registry](/reference/registry/) — browse and publish packages
