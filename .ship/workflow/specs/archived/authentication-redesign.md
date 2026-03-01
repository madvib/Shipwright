# .ship — Directory Structure

**Last Updated:** 2026-02-26

---

## Design Principles

**Modules own namespaces.** The runtime owns all filesystem I/O. Modules declare a root namespace and the runtime enforces that each module only writes within it. The structure is always predictable without knowing which modules are loaded.

**Config is TOML.** Shipwright config files use TOML for readability and parity with frontmatter. Visual preferences (colors, theme) live in the GUI and SQLite — not in config files.

**Feature docs are the source of truth for branch agent config.** MCP servers, skills, model, and cost limit live as structured frontmatter in the feature doc. Shipwright reads this on checkout to generate tool configs. No separate branch config directory.

**Generated tool configs live in `.ship/generated/`.** All AI tool config files (CLAUDE.md, .mcp.json, per-tool configs) are generated outputs. They live under `.ship/generated/` rather than scattered at the project root. One gitignore entry covers all of them. Tools that support a config path flag are pointed directly at their file here. Tools that hardcode their config location get a symlink from the expected path into `.ship/generated/` — symlinks are used only where unavoidable and flagged clearly.

This approach keeps the project root clean regardless of how many AI tools a team supports. Adding a new tool means adding an adapter — nothing changes about the project structure. One update to the feature doc propagates to every allowed tool on next checkout.

**Team policy controls which tools are active.** The `agent_policy.allowed_providers` list in `project.toml` determines which tool configs Shipwright generates. Developers only get configs for tools they have installed. The feature doc is tool-agnostic — it never knows or cares how many tools consume its config.

**Templates are colocated.** Each document directory may contain a `TEMPLATE.md`. No top-level templates directory. Context stays with the documents it applies to.

**Status is the folder for issues.** Moving an issue means moving a file. No status field in frontmatter.

**Releases are optional structure.** The `release` field in feature frontmatter is a soft reference. If the referenced release doc doesn't exist, Shipwright surfaces a prompt to create it — not an error. The workflow never breaks on absent releases.

---

## Project Scope

```
/project-root/
│
│   # Project root is clean. No generated files visible here.
│   # Tools that cannot be redirected get a symlink — see .ship/generated/
│
└── .ship/
    ├── .gitignore               ← managed by Shipwright
    ├── ship.db                  ← runtime SQLite — GITIGNORED
    │
    ├── generated/               ← all generated tool configs — GITIGNORED
    │   │
    │   │   # One entry in .gitignore covers this entire directory.
    │   │   # Tools supporting a config path flag are pointed here directly.
    │   │   # Tools that hardcode config location get a symlink from their
    │   │   # expected path (e.g. project root) into the relevant file here.
    │   │   # Symlinks are created by Shipwright on first manage, removed on unmanage.
    │   │
    │   ├── claude/
    │   │   ├── CLAUDE.md        ← branch context for Claude Code
    │   │   └── mcp.json         ← MCP server config
    │   ├── gemini/
    │   │   └── settings.json
    │   ├── codex/
    │   │   └── config.toml
    │   └── <tool>/              ← one directory per allowed provider
    │
    ├── project/                 ← core namespace
    │   ├── project.toml         ← project identity, git config, team policy
    │   ├── overview.md          ← freeform project overview
    │   ├── vision.md            ← vision document
    │   ├── notes/
    │   │   ├── TEMPLATE.md
    │   │   ├── 2026-02-22-auth-rethink.md
    │   │   └── scratchpad.md
    │   └── adrs/
    │       ├── TEMPLATE.md
    │       ├── adr-001-use-sqlite.md
    │       └── adr-002-toml-frontmatter.md
    │
    ├── workflow/                ← workflow module namespace
    │   ├── releases/
    │   │   ├── TEMPLATE.md
    │   │   └── v1.0.md
    │   ├── features/
    │   │   ├── TEMPLATE.md
    │   │   ├── feature-auth.md      ← agent config in frontmatter
    │   │   └── feature-payments.md
    │   ├── specs/
    │   │   ├── TEMPLATE.md
    │   │   ├── spec-001-auth-redesign.md
    │   │   └── spec-002-payment-flow.md
    │   └── issues/
    │       ├── TEMPLATE.md
    │       ├── backlog/
    │       │   └── issue-004-mobile-offline.md
    │       ├── in-progress/
    │       │   ├── issue-001-auth-ui.md
    │       │   └── issue-002-jwt-refresh.md
    │       ├── review/
    │       │   └── issue-003-session-store.md
    │       ├── blocked/
    │       ├── done/
    │       └── archived/
    │
    └── agents/                  ← agents module namespace
        ├── modes/
        │   ├── planning.toml
        │   └── execution.toml
        ├── skills/
        │   ├── shipwright-workflow.md   ← always injected, locked
        │   └── nextjs-conventions.md
        └── prompts/
            └── owasp-checklist.md
```

---

## `.ship/.gitignore`

```gitignore
# Shipwright runtime
ship.db
ship.db-shm
ship.db-wal

# All generated tool configs
generated/
```

Symlinks created at the project root (for tools that hardcode config location)
are also gitignored. Shipwright appends them to the project root `.gitignore`
when it creates them.

---

## Project Config — `.ship/project/project.toml`

Visual preferences — status colors, tag colors, theme — are set in the GUI and
stored in SQLite. This file contains the things that need to be committed and
shared with the team.

```toml
version = "1"
name = "my-project"
description = ""

[workflow]
preset = "solo" # "solo" | "team"

# IDs are what get committed. Display names and colors are set in the GUI.
[[statuses]]
id = "backlog"

[[statuses]]
id = "in-progress"

[[statuses]]
id = "review"

[[statuses]]
id = "blocked"

[[statuses]]
id = "done"

[[statuses]]
id = "archived"
hidden = true

[[tags]]
id = "priority:high"

[[tags]]
id = "priority:low"

[[tags]]
id = "type:bug"

[[tags]]
id = "type:feature"

[git]
branch_prefix = "feature/"

[git.hooks]
post_checkout = true
pre_commit = true
prepare_commit_msg = true
post_merge = true

# Controls which AI tool configs Shipwright generates on checkout.
# Configs are only generated for tools the developer has installed.
# Adding a new provider here takes effect on next branch checkout.
[agent_policy]
allowed_providers = ["claude", "gemini", "codex"]
```

---

## Feature Doc — `.ship/workflow/features/feature-auth.md`

The feature doc is both the human description of the work and the source of
truth for the branch agent environment. The `[agent]` frontmatter block is
read by Shipwright on checkout to generate configs for every allowed provider.
The body is loaded into each tool's context document (CLAUDE.md, etc.).

Server and skill IDs reference definitions in the global library managed through
the Agents UI. The feature doc contains IDs only — no inline definitions.

```markdown
+++
id = "feature-auth"
title = "Authentication Redesign"
branch = "feature/auth"
created = "2026-02-22"
updated = "2026-02-22"
spec = "spec-001"
release = "v1.0"
tags = ["priority:high"]

[agent]
model = "claude-opus-4-5"        # logical model — Shipwright maps per provider
max_cost_per_session = 5.00      # optional

[[agent.mcp_servers]]
id = "github"

[[agent.mcp_servers]]
id = "postgres"

[[agent.skills]]
id = "nextjs-conventions"
+++

## What We're Building

Authentication redesign supporting SSO, magic links, and Redis-backed
session management. See ADR-003 for the architectural decision.

## Linked Spec

`workflow/specs/spec-001-auth-redesign.md`
```

---

## Mode Config — `.ship/agents/modes/planning.toml`

Modes shape the Shipwright UI and control which Shipwright MCP tools are
surfaced. They do not control external MCP servers — that is the feature doc's
responsibility.

```toml
id = "planning"
name = "Planning"
color = "#6366f1"
shipwright_tools = [
  "ship_list_notes",
  "ship_create_note",
  "ship_list_specs",
  "ship_get_spec",
  "ship_create_spec",
  "ship_update_spec",
  "ship_list_issues",
  "ship_create_issue",
  "ship_draft_adr",
  "ship_get_project_info",
]
```

---

## Global Scope — `~/.ship/`

```
~/.ship/
├── config.toml          ← user preferences — hand-editable
├── shipwright.db        ← global SQLite:
│                            project registry
│                            global mode state
│                            entitlements
│                            global MCP server library
│                            available models cache (24h TTL)
│                            global MCP connection state
└── skills/              ← skills available across all projects
    └── my-skill.md
```

---

## Global Config — `~/.ship/config.toml`

User preferences only. Project registry, server library, entitlements, and
model cache all live in SQLite — they are managed through the GUI, not this file.

```toml
version = "1"

[user]
name = ""
email = ""

[defaults]
editor = "code"
workflow_preset = "solo"
theme = "dark"
accent_color = "blue"

[mcp]
port = 7700
auto_start = true

[ai]
default_model = ""

[git.hooks]
post_checkout = true
pre_commit = true
prepare_commit_msg = true
post_merge = true
```

---

## Autocomplete Surface

Every field referencing another entity must offer completions. This is a
product requirement, not a nice-to-have. Fields marked GUI require a popover
with a "create new" affordance when the referenced entity does not yet exist.

| Field | Source | Mechanism |
|-------|--------|-----------|
| `tags` in any frontmatter | `project.toml` | Config enum + GUI controlled input |
| `status` in issue frontmatter | `project.toml` | Config enum |
| `model` in feature frontmatter | SQLite models cache | GUI popover |
| `agent.mcp_servers[].id` | SQLite server library | GUI popover |
| `agent.skills[].id` | `agents/skills/` | GUI popover |
| `agent.prompts[].id` | `agents/prompts/` | GUI popover |
| `release` in feature frontmatter | `workflow/releases/` | GUI popover + create new |
| `spec` in feature frontmatter | `workflow/specs/` | GUI popover + create new |
| `allowed_providers` entries | `project.toml` | Config enum |
| Mode `shipwright_tools` | `agents/modes/*.toml` | Config enum |

Soft references (`release`, `spec`) show a prompt to create the referenced
document when it does not exist — not an error.
