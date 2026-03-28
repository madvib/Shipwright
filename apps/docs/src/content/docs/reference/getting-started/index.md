---
title: "Introduction"
sidebar:
  label: "Introduction"
  order: 1
---
Ship is a package manager and runtime for AI agents. Install skills and agents from the registry, customize them for your projects, and let Ship handle the rest.

## What Ship does

- **Skills** — install, publish, and personalize agent capabilities. Smart skills adapt to each user and project through typed variables.
- **Agents** — compose skills, tools, permissions, and rules into profiles that work across Claude Code, Cursor, Gemini, Codex, and OpenCode.
- **Studio** — a visual IDE connected to your local projects. Browse the registry, configure agents, edit skills, set preferences.
- **Registry** — share skills and agents publicly at getship.dev, or keep them private to your team.

## Getting started

I recommend starting with Studio:

```bash
curl -fsSL https://getship.dev/install | sh
cd your-project
ship init
ship studio --open
```

Studio connects to your local project and lets you browse the registry, install skills, create agents, and configure everything visually.

Alternatively, ask your AI agent to set things up. Ship's MCP tools let agents install skills and manage configuration on your behalf — just describe what you want.

The [CLI reference](/reference/cli/) documents the full command set for scripting and automation.

Continue to [Installation](./installation).
