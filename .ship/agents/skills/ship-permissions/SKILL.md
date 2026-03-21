---
name: ship-permissions
description: Use when the user asks about agent permissions, security tiers, allowlists, deny lists, or how to control what agents can do. Covers the 4-tier preset system, base security rules, per-profile customization, and how permissions compile into provider-specific configs.
tags: [reference, security, permissions, documentation]
authors: [ship]
---

# Ship Permissions

Ship controls what AI agents can do through a layered permission system. Permissions are declared in TOML, compiled by `ship use`, and emitted as provider-specific config (`.claude/settings.json`, `.cursor/cli.json`, etc.). The agent never sees the TOML — it sees the compiled output.

## The 4 Presets

Every agent profile references a preset via `[permissions] preset = "<name>"`. Presets are defined in `.ship/agents/permissions.toml` and sit on a strict-to-loose continuum.

### ship-readonly

```toml
[ship-readonly]
default_mode = "plan"
tools_deny = ["Write(*)", "Edit(*)", "Bash(rm*)"]
```

**Mode:** Plan only — the agent proposes actions but cannot execute them.
**Denies:** All file writes, edits, and destructive shell commands.
**Use for:** Code reviewers, gate agents, auditors. Anything that should read and assess but never modify.

### ship-standard

```toml
[ship-standard]
default_mode = "default"
```

**Mode:** Default — the agent asks for confirmation before tool calls that modify state.
**Denies:** Nothing beyond base rules (see below).
**Use for:** Interactive sessions, commander/orchestrator agents, brainstorming. The human stays in the loop.

### ship-autonomous

```toml
[ship-autonomous]
default_mode = "dontAsk"
```

**Mode:** Don't ask — the agent executes without confirmation prompts.
**Denies:** Nothing beyond base rules.
**Use for:** Specialist agents working in worktrees on scoped tasks. This is the default for dispatched agents. Over-restricting kills productivity — every approval prompt breaks flow.

### ship-elevated

```toml
[ship-elevated]
default_mode = "dontAsk"
tools_allow_override = ["Bash(git push*)", "Bash(*publish*)"]
```

**Mode:** Don't ask, plus unlocks commands that base rules normally deny.
**Allows:** `git push` and publish commands (npm, cargo, etc.) which every other preset blocks.
**Use for:** CI agents, release automation, deploy pipelines. The only preset that can push to a remote or publish a package.

## Base Rules (Injected Into All Presets)

The compiler injects these rules into every preset automatically. They cannot be removed by profile-level overrides.

```toml
# Always permitted — Ship's own tools must always work
always_allow = ["mcp__ship__*", "Bash(ship *)"]

# Always blocked — dangerous operations require explicit elevation
always_deny = [
  "Bash(sqlite3 ~/.ship/*)",      # direct DB access bypasses the runtime
  "Bash(git push*)",               # only ship-elevated unlocks this
  "Bash(*publish*)",               # only ship-elevated unlocks this
  "Read(.env*)", "Write(.env*)",   # secrets
  "Read(.dev.vars*)", "Write(.dev.vars*)",
  "Read(credentials*)", "Write(credentials*)",
  "Read(secrets/*)", "Write(secrets/*)",
]

# Always requires confirmation — even in dontAsk mode
always_ask = ["Write(.ship/*)", "Edit(.ship/*)"]

# Ship is the memory layer — provider-native memories are always off
autoMemoryEnabled = false
```

Key points:
- `always_deny` blocks `git push` and `publish` globally. Only `ship-elevated` reopens them via `tools_allow_override`.
- `always_ask` protects `.ship/` config files even in autonomous mode — an agent cannot silently rewrite its own configuration.
- Direct SQLite access is denied because all state must flow through Ship's runtime (MCP tools or CLI).

## Per-Profile Customization

Profiles add overrides on top of their preset. The `[permissions]` section in a profile TOML supports:

| Field | Type | Effect |
|-------|------|--------|
| `preset` | string | Which of the 4 presets to start from |
| `tools_deny` | string[] | Additional tool patterns to block |
| `tools_ask` | string[] | Additional tool patterns that require confirmation |
| `default_mode` | string | Override the preset's default mode |

Example — a Rust specialist that blocks force push and publish:

```toml
[permissions]
preset = "ship-autonomous"
tools_deny = ["Bash(git push --force*)", "Bash(*cargo publish*)", "Bash(*npm publish*)"]
```

Example — a commander that wants confirmation on destructive git ops:

```toml
[permissions]
preset = "ship-standard"
tools_deny = []
tools_ask = ["Bash(git reset*)", "Bash(git push --force*)"]
```

Profiles can only add restrictions (deny, ask) on top of presets. They cannot remove base rules.

## How Compilation Works

When you run `ship use <profile>`, the compiler:

1. **Loads the base** `Permissions` struct (safe defaults from the runtime).
2. **Resolves the preset** from `.ship/agents/permissions.toml` by section name (e.g. `[ship-autonomous]`). This sets `default_mode`, `tools_deny`, `tools_ask`, and optionally `tools_allow`.
3. **Applies profile overrides** — `tools_deny` and `tools_ask` from the profile TOML are merged on top. Profile-level `default_mode` wins over the preset value.
4. **Emits provider-specific config.** For Claude Code, this becomes `.claude/settings.json`:

```json
{
  "permissions": {
    "allow": ["Read", "Glob", "LS", "mcp__ship__*", "Bash(ship *)"],
    "deny": ["Write(*)", "Edit(*)", "Bash(rm*)"],
    "defaultMode": "plan"
  },
  "autoMemoryEnabled": false
}
```

For Cursor, permissions go to `.cursor/cli.json`. For Gemini, they go to `.gemini/policies/`. The TOML is the single source — provider configs are ephemeral build artifacts.

## The Compound Command Problem

Claude Code matches permission patterns against the **full command string**. This means compound shell commands defeat simple pattern matching.

```toml
# This will NOT match: cd .target/bin && ship exec blah
tools_ask = ["Bash(ship exec*)"]
```

The pattern `Bash(ship exec*)` only matches when `ship exec` is at the start of the command. If the agent chains it with `cd` or `&&`, the pattern silently fails to match.

Rules for working around this:
- **Do not try to allowlist compound commands with patterns.** It will not work reliably.
- **Use `default_mode` via presets to set the baseline.** If the agent needs to run commands freely, use `ship-autonomous` — don't try to enumerate every allowed pattern.
- **Use `tools_ask` only for specific destructive operations** where the pattern is unambiguous (e.g. `Bash(rm -rf*)`).
- If a workflow always chains commands, either document the full pattern or grant the preset that doesn't need to ask.

```toml
# Wrong: trying to pattern-match compound commands
tools_ask = ["Bash(ship exec*)"]   # misses: cd dir && ship exec

# Right: set mode via preset, guard only what's genuinely dangerous
[permissions]
preset = "ship-autonomous"
tools_deny = ["Bash(rm -rf*)"]
```

## Security Model: Defense in Depth

Ship does not rely on any single mechanism for safety. Three layers work together:

### Layer 1: Worktree Isolation

Specialist agents work in git worktrees — separate working directories branched from the main repo. An agent in a worktree cannot corrupt the main branch. If something goes wrong, `git worktree remove` deletes the mess.

### Layer 2: Permission Presets

The 4-tier system controls what tools the agent can invoke. Base rules ensure no agent can push code, publish packages, access secrets, or tamper with Ship's own database — unless explicitly elevated.

### Layer 3: Gate Review

Before worktree work merges back, a reviewer agent (running with `ship-readonly`) or a human inspects the diff. The agent that did the work cannot approve its own merge.

Together: the agent works in an isolated branch, with tool restrictions matching its role, and a separate review step before anything reaches the main codebase. No single failure compromises the system.

## Quick Reference

| I want to... | Use preset | Add to profile |
|---|---|---|
| Read-only analysis | `ship-readonly` | Nothing needed |
| Interactive pair programming | `ship-standard` | `tools_ask` for specific ops |
| Dispatch a specialist to a worktree | `ship-autonomous` | `tools_deny` for things it shouldn't touch |
| Run CI / deploy / release | `ship-elevated` | Scope carefully |
| Block a specific command | Any preset | `tools_deny = ["Bash(pattern*)"]` |
| Require confirmation for a command | `ship-standard` | `tools_ask = ["Bash(pattern*)"]` |
