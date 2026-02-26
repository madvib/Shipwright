# .ship — Directory Structure

**Last Updated:** 2026-02-22

---

## Design Principles

**Modules own namespaces.** The runtime owns all filesystem I/O. Modules declare a root namespace and the runtime enforces that each module only writes within it. The structure is always predictable without knowing which modules are loaded.

**Config is JSONC with published schema.** Every `.jsonc` file carries a `$schema` field pointing to the published Shipwright schema. This gives autocomplete and inline validation in any editor without a plugin. Visual preferences (colors, theme) live in the GUI and SQLite — not in config files.

**Feature docs are the source of truth for branch agent config.** MCP servers, skills, model, and cost limit live as structured frontmatter in the feature doc. Shipwright reads this on checkout to generate tool configs. No separate branch config directory.

**Generated tool configs live in `.ship/generated/`.** All AI tool config files (CLAUDE.md, .mcp.json, per-tool configs) are generated outputs. They live under `.ship/generated/` rather than scattered at the project root. One gitignore entry covers all of them. Tools that support a config path flag are pointed directly at their file here. Tools that hardcode their config location get a symlink from the expected path into `.ship/generated/` — symlinks are used only where unavoidable and flagged clearly.

This approach keeps the project root clean regardless of how many AI tools a team supports. Adding a new tool means adding an adapter — nothing changes about the project structure. One update to the feature doc propagates to every allowed tool on next checkout.

**Team policy controls which tools are active.** The `agentPolicy.allowedProviders` list in `project.jsonc` determines which tool configs Shipwright generates. Developers only get configs for tools they have installed. The feature doc is tool-agnostic — it never knows or cares how many tools consume its config.

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
    │   ├── project.jsonc        ← project identity, git config, team policy
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
        │   ├── planning.jsonc
        │   └── execution.jsonc
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

## Project Config — `.ship/project/project.jsonc`

Visual preferences — status colors, tag colors, theme — are set in the GUI and
stored in SQLite. This file contains the things that need to be committed and
shared with the team.

```jsonc
{
  "$schema": "https://schema.shipwright.dev/project/v1.json",
  "version": "1",
  "name": "my-project",
  "description": "",

  "workflow": {
    "preset": "solo"
    // "solo" | "team"
    // Transition configuration is managed through the GUI.
  },

  // IDs are what get committed. Display names and colors are set in the GUI.
  "statuses": [
    { "id": "backlog"                          },
    { "id": "in-progress"                      },
    { "id": "review"                           },
    { "id": "blocked"                          },
    { "id": "done"                             },
    { "id": "archived",    "hidden": true      }
  ],

  "tags": [
    { "id": "priority:high" },
    { "id": "priority:low"  },
    { "id": "type:bug"      },
    { "id": "type:feature"  }
  ],

  "git": {
    "branchPrefix": "feature/",
    "hooks": {
      "postCheckout":     true,
      "preCommit":        true,
      "prepareCommitMsg": true,
      "postMerge":        true
    }
  },

  // Controls which AI tool configs Shipwright generates on checkout.
  // Configs are only generated for tools the developer has installed.
  // Adding a new provider here takes effect on next branch checkout.
  "agentPolicy": {
    "allowedProviders": ["claude", "gemini", "codex"]
  }
}
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
status = "active"
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

## Mode Config — `.ship/agents/modes/planning.jsonc`

Modes shape the Shipwright UI and control which Shipwright MCP tools are
surfaced. They do not control external MCP servers — that is the feature doc's
responsibility.

```jsonc
{
  "$schema": "https://schema.shipwright.dev/mode/v1.json",
  "id": "planning",
  "name": "Planning",
  "color": "#6366f1",
  "shipwrightTools": [
    "ship_list_notes",
    "ship_create_note",
    "ship_list_specs",
    "ship_get_spec",
    "ship_create_spec",
    "ship_update_spec",
    "ship_list_issues",
    "ship_create_issue",
    "ship_draft_adr",
    "ship_get_project_info"
  ]
}
```

---

## Global Scope — `~/.ship/`

```
~/.ship/
├── config.jsonc         ← user preferences — hand-editable
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

## Global Config — `~/.ship/config.jsonc`

User preferences only. Project registry, server library, entitlements, and
model cache all live in SQLite — they are managed through the GUI, not this file.

```jsonc
{
  "$schema": "https://schema.shipwright.dev/global-config/v1.json",
  "version": "1",

  "user": {
    "name": "",
    "email": ""
  },

  "defaults": {
    "editor": "code",
    "workflowPreset": "solo",
    "theme": "dark",
    "accentColor": "blue"
  },

  "mcp": {
    "port": 7700,
    "autoStart": true
  },

  "ai": {
    "defaultModel": ""
  },

  "git": {
    "hooks": {
      "postCheckout":     true,
      "preCommit":        true,
      "prepareCommitMsg": true,
      "postMerge":        true
    }
  }
}
```

---

## Autocomplete Surface

Every field referencing another entity must offer completions. This is a
product requirement, not a nice-to-have. Fields marked GUI require a popover
with a "create new" affordance when the referenced entity does not yet exist.

| Field | Source | Mechanism |
|-------|--------|-----------|
| `tags` in any frontmatter | `project.jsonc` | Schema enum + GUI controlled input |
| `status` in issue frontmatter | `project.jsonc` | Schema enum |
| `model` in feature frontmatter | SQLite models cache | GUI popover |
| `agent.mcp_servers[].id` | SQLite server library | GUI popover |
| `agent.skills[].id` | `agents/skills/` | GUI popover |
| `agent.prompts[].id` | `agents/prompts/` | GUI popover |
| `release` in feature frontmatter | `workflow/releases/` | GUI popover + create new |
| `spec` in feature frontmatter | `workflow/specs/` | GUI popover + create new |
| `allowedProviders` entries | Published schema | Schema enum |
| Mode `shipwrightTools` | Published schema | Schema enum |

Soft references (`release`, `spec`) show a prompt to create the referenced
document when it does not exist — not an error.
