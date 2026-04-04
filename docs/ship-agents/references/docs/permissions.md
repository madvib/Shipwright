---
group: Agents
title: Permissions
order: 3
---

# Permissions

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

## Base rules

The compiler injects these rules into every preset automatically. They cannot be removed by profile-level overrides.

```jsonc
// Always permitted — Ship's own tools must always work
"always_allow": ["mcp__ship__*", "Bash(ship *)"],

// Always blocked — dangerous operations require explicit elevation
"always_deny": [
  "Bash(sqlite3 ~/.ship/*)",      // direct DB access bypasses the runtime
  "Bash(git push*)",               // only ship-elevated unlocks this
  "Bash(*publish*)",               // only ship-elevated unlocks this
  "Read(.env*)", "Write(.env*)",   // secrets
  "Read(.dev.vars*)", "Write(.dev.vars*)",
  "Read(credentials*)", "Write(credentials*)",
  "Read(secrets/*)", "Write(secrets/*)"
],

// Always requires confirmation — even in dontAsk mode
"always_ask": ["Write(.ship/*)", "Edit(.ship/*)"],

// Ship is the memory layer — provider-native memories are always off
"autoMemoryEnabled": false
```

Key points:
- `always_deny` blocks `git push` and `publish` globally. Only `ship-elevated` reopens them via `tools_allow`.
- `always_ask` protects `.ship/` config files even in autonomous mode — an agent cannot silently rewrite its own configuration.
- Direct SQLite access is denied because all state must flow through Ship's runtime (MCP tools or CLI).

## Compilation

When you run `ship use <profile>`, the compiler:

1. Loads the base `Permissions` struct (safe defaults from the runtime).
2. Resolves the preset from `.ship/permissions.jsonc` by key name.
3. Applies agent overrides — `tools_deny` and `tools_ask` from the agent JSONC are merged on top. Agent-level `default_mode` wins over the preset value.
4. Emits provider-specific config. For Claude Code, this becomes `.claude/settings.json`. For Cursor, permissions go to `.cursor/cli.json`. For Gemini, they go to `.gemini/policies/`.

The JSONC is the single source — provider configs are ephemeral build artifacts.

## The compound command problem

Claude Code matches permission patterns against the full command string. Compound shell commands defeat simple pattern matching.

```bash
# This will NOT match: cd .target/bin && ship exec blah
tools_ask = ["Bash(ship exec*)"]
```

The pattern `Bash(ship exec*)` only matches when `ship exec` is at the start of the command. If the agent chains it with `cd` or `&&`, the pattern silently fails to match.

Rules for working around this:
- Do not try to allowlist compound commands with patterns. It will not work reliably.
- Use `default_mode` via presets to set the baseline. If the agent needs to run commands freely, use `ship-autonomous` — don't enumerate every allowed pattern.
- Use `tools_ask` only for specific destructive operations where the pattern is unambiguous (e.g. `Bash(rm -rf*)`).

```jsonc
// Wrong: trying to pattern-match compound commands
"tools_ask": ["Bash(ship exec*)"]   // misses: cd dir && ship exec

// Right: set mode via preset, guard only what's genuinely dangerous
"permissions": {
  "preset": "ship-autonomous",
  "tools_deny": ["Bash(rm -rf*)"]
}
```

## Security model

Ship does not rely on any single mechanism for safety. Three layers work together:

**Layer 1: Worktree isolation.** Specialist agents work in git worktrees — separate working directories branched from the main repo. An agent in a worktree cannot corrupt the main branch. If something goes wrong, `git worktree remove` deletes the mess.

**Layer 2: Permission presets.** The 4-tier system controls what tools the agent can invoke. Base rules ensure no agent can push code, publish packages, access secrets, or tamper with Ship's database — unless explicitly elevated.

**Layer 3: Gate review.** Before worktree work merges back, a reviewer agent (running with `ship-readonly`) or a human inspects the diff. The agent that did the work cannot approve its own merge.

Together: the agent works in an isolated branch, with tool restrictions matching its role, and a separate review step before anything reaches the main codebase. No single failure compromises the system.

## Quick reference

| I want to... | Use preset | Add to profile |
|---|---|---|
| Read-only analysis | `ship-readonly` | Nothing needed |
| Interactive pair programming | `ship-standard` | `tools_ask` for specific ops |
| Dispatch a specialist to a worktree | `ship-autonomous` | `tools_deny` for things it shouldn't touch |
| Run CI / deploy / release | `ship-elevated` | Scope carefully |
| Block a specific command | Any preset | `tools_deny = ["Bash(pattern*)"]` |
| Require confirmation for a command | `ship-standard` | `tools_ask = ["Bash(pattern*)"]` |
