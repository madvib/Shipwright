# Ship Workspace: Project Manager Layer

## Status
Draft

## Overview

Ship should have a first-class **project workspace** — a persistent, non-branch-locked workspace that acts as the PM layer of the project. Rather than requiring a mode to unlock planning and management tooling, the `project` workspace type automatically surfaces the full management surface: issues, releases, specs, ADRs, roadmap. It is the home base from which feature workspaces are spawned, and the landing zone for end-of-session feedback.

This is distinct from feature/refactor/experiment/hotfix workspaces, which are code-first and session-bound. The project workspace is always running in the background as a high-altitude view of the whole project.

## Intent

Agents and humans working on planning, triage, release prep, and architecture should not need to first activate a mode or remember which tools are gated. The project workspace is the right mental model for "I want to think about the project, not write code right now." It is the place where you:

- Brainstorm and refine features before assigning them to a branch
- Triage issues and decide priority
- Prepare a release and write release notes
- Write a spec or ADR before implementation begins
- Review what sessions have accomplished and update documentation
- Coordinate across multiple active feature workspaces

## Design

### New workspace type: `Project`

```
WorkspaceType::Project
```

- Created automatically on `init_project` with branch name `ship` (or configurable)
- Not tied to a git branch in the code-branch sense — it tracks the project root
- Only one project workspace per project (enforced)
- Persists across sessions; never archived or merged

### Automatic tool surface expansion

When the active workspace is of type `Project`, the tool gate expands without requiring a mode. All planning and management tools are in the core surface:

**Core (always available):**
- `get_project_info`, `open_project`
- `create_note`, `create_feature`, `get_feature`, `update_feature`, `log_decision`
- `list_workspaces`, `get_workspace`, `activate_workspace`, `create_workspace_tool`
- `list_modes`, `set_mode`
- `start_session`, `end_session`, `get_session_status`, `log_progress`

**Auto-expanded in project workspace (no mode needed):**
- `list_issues`, `create_issue`, `update_issue`, `move_issue`
- `list_releases`, `create_release`, `update_release`
- `list_specs`, `create_spec`, `get_spec`, `update_spec`
- `list_adrs`, `get_adr` (create via `log_decision`)
- `list_sessions` (cross-workspace session history)

### Provider context

When a session starts in a project workspace, the compiled CLAUDE.md takes a bird's-eye view:

- All open features with their status and Documentation flag
- All open issues grouped by priority/status
- Active workspaces and their current sessions
- Upcoming release and what's in it
- Recent session log (what got done across all workspaces)
- Linked specs and ADRs for in-flight features

This is fundamentally different from a feature workspace CLAUDE.md which focuses on a single branch/feature.

### Session semantics

Sessions in the project workspace are **planning sessions**, not coding sessions:

- Goal: "Plan v0.2.0 release", "Triage backlog", "Write spec for PTY integration"
- `log_progress` notes capture decisions made, features refined, issues moved
- `end_session` feedback can link to features that were created/refined (not code-changed)
- No git diff context in the CLAUDE.md — instead, changelog of what features moved status

### init_project seeding

On `init_project`, after the usual workspace setup:

```
ship workspace create --type project --branch ship
ship workspace activate ship
```

Or internally, `seed_project_workspace()` creates the SQLite record with:
- `workspace_type: WorkspaceType::Project`
- `branch: "ship"` (configurable via ship.toml)
- `status: WorkspaceStatus::Active`

### CLI ergonomics

The project workspace makes `ship session start` feel like a single entry point:

```
ship session start "plan v0.2.0 release"
# → detects current workspace type
# → if project workspace: broad PM context compiled
# → if feature workspace: feature-focused context compiled
# → with PTY: spawns agent in terminal with right context
```

`ship status` (new top-level command) shows:
- Active workspace + type
- Active session goal (if any)
- Open issues count, in-progress features, upcoming release

## Acceptance Criteria

- [ ] `WorkspaceType::Project` variant in runtime
- [ ] `seed_project_workspace()` called on `init_project` — creates `ship` workspace
- [ ] `is_core_tool()` / `enforce_mode_tool_gate()` expands surface when active workspace is type `Project`
- [ ] `list_tools()` in MCP server reflects expanded surface for project workspace
- [ ] `get_project_info` detects project workspace and outputs bird's-eye CLAUDE.md context (cross-workspace view)
- [ ] `start_session` in project workspace compiles project-overview CLAUDE.md (not feature-branch context)
- [ ] Only one project workspace enforced per project (create errors if one exists)
- [ ] `ship session start` / `ship session end` / `ship session log` top-level CLI shortcuts
- [ ] `ship status` top-level command

## Out of Scope (Alpha)

- Multi-project coordination (project workspace spanning multiple repos)
- UI-specific project workspace panel (tracked separately)
- Automatic cross-workspace session summarization
- GitHub Projects / Linear sync from project workspace sessions

## Related

- Feature: MCP Surface Rationalization
- ADR: CLI Surface — workflow-first, plumbing hidden
- Spec: PTY Integration (when written)
- Feature: Workspace Management UI
