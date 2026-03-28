---
title: "Permissions"
sidebar:
  label: "Permissions"
  order: 3
---
Permissions control what tools an agent can use without asking for confirmation. Four built-in presets form a strict-to-loose continuum, defined in `.ship/permissions.jsonc`.

Presets are templates, not security boundaries. Real isolation comes from worktree separation and gate reviews.

## Preset overview

| Preset | Default Mode | Use Case |
|--------|-------------|----------|
| `ship-readonly` | `plan` | Reviewers, gate agents, auditors |
| `ship-standard` | `default` | Interactive sessions, human-paired work |
| `ship-autonomous` | `dontAsk` | Specialist agents in worktrees |
| `ship-elevated` | `dontAsk` | CI agents, release automation |

The `default_mode` maps to provider permission modes. `plan` means the agent proposes but cannot act. `default` means the provider decides when to prompt. `dontAsk` means all allowed tools run without confirmation.

## ship-readonly

Read-only analysis. The agent can read files, search content, run Ship CLI commands, and call MCP read/list/search tools. It cannot write files or run destructive shell commands.

```jsonc
{
  "default_mode": "plan",
  "tools_allow": [
    "Read", "Glob", "Grep",
    "Bash(ship *)",
    "mcp__ship__*",
    "mcp__*__read*", "mcp__*__list*", "mcp__*__search*"
  ],
  "tools_deny": [
    "Write(*)", "Edit(*)", "Bash(rm*)"
  ]
}
```

## ship-standard

The default for interactive work. The agent can read, write, and use Ship tools. The provider prompts for confirmation on operations it deems risky. Migration SQL files are protected from direct edits.

```jsonc
{
  "default_mode": "default",
  "tools_allow": [
    "Read", "Glob", "Grep",
    "Bash(ship *)",
    "mcp__ship__*"
  ],
  "tools_deny": [
    "Write(**/migrations/**/*.sql)",
    "Edit(**/migrations/**/*.sql)"
  ]
}
```

## ship-autonomous

For agents dispatched to work independently in isolated worktrees. All file operations and most shell commands are pre-approved. Destructive operations, publishing, and pushes are denied. Migration SQL files remain protected.

```jsonc
{
  "default_mode": "dontAsk",
  "tools_allow": [
    "Read", "Write", "Edit", "Glob", "Grep",
    "Bash(*)",
    "mcp__ship__*"
  ],
  "tools_deny": [
    "Bash(rm -rf *)",
    "Bash(git reset --hard*)",
    "Bash(git push*)",
    "Bash(cargo publish*)",
    "Bash(npm publish*)",
    "Write(**/migrations/**/*.sql)",
    "Edit(**/migrations/**/*.sql)"
  ]
}
```

## ship-elevated

For deploy and release automation. Like `ship-autonomous` but unlocks `git push` and publish commands. Only `rm -rf`, `git reset --hard`, and `git push --force` remain denied.

```jsonc
{
  "default_mode": "dontAsk",
  "tools_allow": [
    "Read", "Write", "Edit", "Glob", "Grep",
    "Bash(*)",
    "mcp__ship__*"
  ],
  "tools_deny": [
    "Bash(rm -rf *)",
    "Bash(git reset --hard*)",
    "Bash(git push --force*)",
    "Write(**/migrations/**/*.sql)",
    "Edit(**/migrations/**/*.sql)"
  ]
}
```

## Per-agent layering

Agents select a preset and layer additional rules on top:

```jsonc
{
  "permissions": {
    "preset": "ship-autonomous",
    "tools_allow": ["Bash(docker *)"],
    "tools_deny": ["Bash(rm -rf *)", "Bash(git reset --hard*)"],
    "tools_ask": ["Bash(git push --force*)"]
  }
}
```

| Field | Effect |
|-------|--------|
| `tools_allow` | Merged into the preset's allow list |
| `tools_deny` | Merged into the preset's deny list |
| `tools_ask` | Tools that require confirmation regardless of mode |

Overrides are additive (merged, not replaced). When a tool matches both allow and deny, deny wins.

## Escalation between presets

The key differences between each step up:

**readonly to standard** -- Gains write access to files and broader shell commands. Still prompts for confirmation on risky operations.

**standard to autonomous** -- Switches to `dontAsk` mode. Gains pre-approved `Bash(*)`. Blocks push and publish.

**autonomous to elevated** -- Unlocks `git push` and `cargo/npm publish`. Only force-push and destructive resets remain blocked.

## Custom presets

Add your own presets by adding keys to `.ship/permissions.jsonc`:

```jsonc
{
  "my-ci-preset": {
    "default_mode": "dontAsk",
    "tools_allow": ["Read", "Glob", "Grep", "Bash(pnpm *)"],
    "tools_deny": ["Write(*)", "Edit(*)"]
  }
}
```

Reference it in an agent with `"preset": "my-ci-preset"`.
