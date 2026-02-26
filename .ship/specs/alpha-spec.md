# Ship — Alpha Specification

**Version:** 0.1-alpha  
**Status:** Active  
**Last Updated:** 2026-02-25

---

## The Alpha in One Sentence

Markdown delivery artifacts in a git repo, with a clean UI, an MCP server that persists project memory across sessions, and one opinionated flow from feature to shipped work.

---

## The Core Loop

This is the only workflow that matters for alpha:

```
Vision → Release → Feature → Spec → Issues → ADRs → Close Feature → Ship Release
```

Everything in the alpha exists to serve this loop. If a feature doesn't directly support it, it doesn't ship in alpha.

---

## What Alpha Is Not

- No plugin system (architecture is plugin-aware, nothing is activated)
- No cloud sync
- No user accounts
- No cross-project linking
- No custom document types
- No agent session spawning
- No project scaffolding
- No time tracking, ghost issues, or marketplace
- No mobile

---

## Format: One Rule

| File | Format |
|------|--------|
| Issues, Specs, ADRs | Markdown with TOML frontmatter (`+++` delimiters) |
| All config | TOML (`.toml`) |

No YAML. No JSON. No exceptions.

---

## Directory Structure

```
/project-root/
└── .ship/
    ├── config.toml
    ├── templates/
    │   ├── RELEASE.md
    │   ├── FEATURE.md
    │   ├── ISSUE.md
    │   ├── SPEC.md
    │   ├── VISION.md
    │   └── ADR.md
    ├── releases/
    ├── features/
    ├── issues/
    │   ├── backlog/
    │   ├── in-progress/
    │   ├── review/
    │   ├── done/
    │   └── blocked/
    ├── specs/
    │   └── vision.md
    ├── adrs/
    └── log.md

~/.ship/
├── config.toml
└── registry.toml
```

**Status is the folder.** Moving an issue = moving a file. No status field in frontmatter needed. This makes it trivially parseable by any tool, agent, or grep.

**Templates are just markdown files.** `.ship/templates/RELEASE.md`, `.ship/templates/FEATURE.md`, `.ship/templates/ISSUE.md`, `.ship/templates/SPEC.md`, and `.ship/templates/ADR.md` are editable by users. No plugin required.

---

## Default Templates

### `.ship/templates/RELEASE.md`

```markdown
+++
id = ""
version = "v0.1.0-alpha"
status = "planned"
created = ""
updated = ""
target_date = ""
features = []
adrs = []
tags = []
+++

## Goal


## Scope

- [ ]

## Included Features

- [ ]

## Notes

```

### `.ship/templates/FEATURE.md`

```markdown
+++
id = ""
title = ""
status = "active"
created = ""
updated = ""
owner = ""
spec = ""
adrs = []
tags = []
+++

## Why


## Acceptance Criteria

- [ ]

## Delivery Todos

- [ ]

## Notes

```

### `.ship/templates/ISSUE.md`

```markdown
+++
id = ""
title = ""
created = ""
updated = ""
assignee = ""
tags = []
spec = ""

[[links]]
type = ""
target = ""
+++

## Description



## Tasks

- [ ] 

## Notes

```

### `.ship/templates/SPEC.md`

```markdown
+++
id = ""
title = ""
status = "draft"
created = ""
updated = ""
author = ""
tags = []
+++

## Overview



## Goals



## Non-Goals



## Approach



## Open Questions

```

### `.ship/templates/ADR.md`

```markdown
+++
id = ""
title = ""
status = "proposed"
date = ""
tags = []
spec = ""
+++

## Context



## Decision



## Consequences

### Positive


### Negative

```

---

## Project Config (`.ship/config.toml`)

Sensible defaults. Teams change what they need, leave the rest.

```toml
version = "1"
name = "my-project"
description = ""

[[statuses]]
id = "backlog"
name = "Backlog"
color = "gray"

[[statuses]]
id = "in-progress"
name = "In Progress"
color = "blue"

[[statuses]]
id = "review"
name = "Review"
color = "yellow"

[[statuses]]
id = "done"
name = "Done"
color = "green"

[[statuses]]
id = "blocked"
name = "Blocked"
color = "red"

[[tags]]
id = "priority:high"
color = "red"

[[tags]]
id = "priority:low"
color = "green"

[git]
# Ship manages .ship/.gitignore with these defaults
ignore = []                  # generated from commit list by Ship
commit = ["releases", "features", "specs", "adrs", "config.toml", "templates"]

[ai]
context_files = ["AGENTS.md"]
```

---

## Global Config (`~/.ship/config.toml`)

```toml
version = "1"

[user]
name = ""
email = ""

[defaults]
editor = "code"

[mcp]
enabled = true
port = 7700

[ui]
theme = "dark"
```

---

## CLI

Minimal. Every command maps directly to the core loop.

```
# Project setup
ship init [path]              # Initialize .ship/ with defaults
ship project link <path>      # Register existing project
ship project list
ship project info

# Specs
ship spec create <title>      # Create from template, open in editor
ship spec list
ship spec show <id>
ship spec edit <id>

# Issues
ship issue create <title>     # Create from template, open in editor
ship issue list [--status]
ship issue show <id>
ship issue edit <id>
ship issue move <id> <status>
ship issue delete <id>
ship issue link <id> <id>

# ADRs
ship adr create <title>
ship adr list
ship adr show <id>
ship adr edit <id>

# Config
ship config global
ship config local

# MCP + UI
ship mcp start
ship mcp status
ship ui
```

---

## Desktop App (Tauri)

React + TypeScript + Tailwind CSS v4. Tauri 2.x.

### Views

**Kanban** — The default landing view. Columns from config statuses. Cards show title, assignee, tags. Drag-and-drop moves the file to the correct status folder and updates `updated` timestamp. Click a card to open issue detail.

**Issue Detail** — Full markdown render. Edit in place. Frontmatter fields editable as a form (title, assignee, tags, spec reference, links). Feels like a native editor, not a web form.

**Spec Editor** — Split view. Left: editable markdown document. Right: AI conversation scoped to this spec via MCP sampling. "Extract Issue" button creates a new issue pre-populated from spec context. This is the primary surface for PMs and founders.

**ADR List** — Simple table. Status, date, title. Click to read, button to create.

**Settings** — GUI for `.ship/config.toml`. Edit statuses, tags, git behavior, templates. This replaces hand-editing TOML for most users.

### Empty States

Every view needs a polished empty state. This is a first impression. One sentence explaining what the thing is. One CTA. For Specs especially — welcoming to non-technical users who've never heard of an ADR.

### Quality Bar

This is a commercial product. Drag-and-drop must be smooth. Typography and spacing must be consistent. It should feel like Linear, not like a developer's weekend project.

---

## MCP Server

The alpha's killer feature. Persistent project memory that survives across AI sessions.

### What This Enables

An agent starts a session and can immediately read all open issues, the current spec, prior ADRs, and a log of what's happened. It does its work, updates issue status, logs a decision as an ADR. The next session — human or agent — picks up exactly where things left off. No re-explaining. No lost context.

### Alpha MCP Tools

```
# Issues
ship_list_issues          # Optional status filter
ship_get_issue            # Full content by id
ship_create_issue         # Create from frontmatter + body
ship_update_issue         # Update fields or body
ship_move_issue           # Change status (moves file)
ship_link_issues          # Add relationship between issues

# Specs
ship_list_specs
ship_get_spec
ship_create_spec
ship_update_spec

# ADRs
ship_list_adrs
ship_get_adr
ship_create_adr

# Project context
ship_get_project_info     # Name, statuses, tags, config
ship_get_log              # Recent action log entries

# Sampling (user-initiated only)
ship_refine_spec          # Chat against a spec
ship_extract_issues       # Suggest issues from spec content
ship_draft_adr            # Draft an ADR from context
```

### MCP Sampling

Ship uses MCP sampling — requests completions from the user's connected AI client, not from any API it controls. No API keys in Ship. No model costs charged through Ship. Works with Claude, Cursor, Windsurf, or anything MCP-compatible.

Sampling features in alpha are always user-initiated. No auto-triggers.

### Action Log (`log.md`)

Every write operation appends to `.ship/log.md`:

```
2026-02-22T14:30:00Z [human] issue-001 moved in-progress → review
2026-02-22T14:35:00Z [agent:claude] issue-001 updated: added task breakdown
2026-02-22T14:36:00Z [agent:claude] adr-003 created: "Use Redis for session storage"
```

Append-only. Human-readable. Ignored by git by default (configurable). Gives agents project history without diffing files.

---

## Crate Structure

```
crates/
├── logic/          # Core domain — issues, specs, ADRs, config, log, templates
├── cli/            # CLI (calls logic)
├── mcp/            # MCP server (calls logic)
├── ui/
│   ├── src/        # React + TypeScript
│   └── src-tauri/  # Tauri 2.x (calls logic via commands)
└── plugins/        # First-party premium plugins (all implement ShipPlugin)
                    # Empty in alpha — home for v1 premium plugin crates
```

### Plugin Readiness (Architecture Only)

No plugins activate in alpha. But the logic crate's mutation paths call through a plugin hook point (no-op) so the surface exists without breaking changes later.

```rust
// Stubbed in alpha — real in v1
pub trait ShipPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn on_issue_created(&self, ctx: &PluginContext, issue: &Issue) -> Result<()>;
    fn on_issue_moved(&self, ctx: &PluginContext, issue: &Issue, from: &str, to: &str) -> Result<()>;
    fn on_spec_updated(&self, ctx: &PluginContext, spec: &Spec) -> Result<()>;
    fn on_adr_created(&self, ctx: &PluginContext, adr: &Adr) -> Result<()>;
    fn mcp_tools(&self) -> Vec<McpTool> { vec![] }
    fn ui_panels(&self) -> Vec<UiPanel> { vec![] }
    fn templates(&self) -> Vec<Template> { vec![] }
    fn document_types(&self) -> Vec<DocumentType> { vec![] }
}
```

Note `document_types()` — this is the hook that will eventually let a plugin define an entirely new document type (with its own schema, MCP tools, UI view, and template). Alpha stubs it. V1 makes it real.

---

## Alpha Done Criteria

Ship alpha is done when the core loop works end-to-end without friction:

1. `ship init` in a new project — works in under 10 seconds
2. Open desktop app — polished, no rough edges on the happy path
3. Create a spec, refine it in the split view with an AI conversation
4. Extract 2–3 issues from that conversation
5. See those issues on the Kanban board
6. Move an issue via drag-and-drop
7. Open Claude Desktop or Cursor, connect to Ship's MCP server
8. Agent reads the spec and open issues without being told they exist
9. Agent updates an issue — change appears in the Kanban board
10. `log.md` shows a coherent history of human and agent actions
11. No account. No internet. One binary.

---

## Open Questions (Decide Before Shipping Alpha)

1. **Issue IDs**: `issue-001` (auto-increment) vs slugs vs UUIDs? Recommend locking `issue-NNN` for human readability and CLI ergonomics.
2. **MCP server lifecycle**: Auto-start when `ship ui` launches, or always explicit? Recommend auto-start with a visible status indicator in the UI.
3. **Spec conversation persistence**: Ephemeral (log.md only) or saved alongside the spec? Recommend ephemeral for alpha — simpler, revisit in v1.
4. **`ship init` behavior**: Should it detect an existing git repo and offer to commit `.ship/` immediately?

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 0.1-alpha | 2026-02-22 | Rewrite — ruthless alpha scope, plugin-aware architecture, core loop focus |
