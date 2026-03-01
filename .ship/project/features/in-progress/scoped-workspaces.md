+++
id = "KLrvRWB8"
title = "Scoped Workspaces"
created = "2026-02-28T15:56:07Z"
updated = "2026-02-28T15:56:07Z"
branch = ""
release_id = "v0.1.0-alpha"
spec_id = ""
adr_ids = []
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

When a developer checks out a branch, Shipwright needs to know exactly what work is happening and configure the agent environment for it. A Workspace is the resolved, branch-scoped agent context — linking the branch to a feature/spec/release and computing the full agent configuration (skills, MCP servers, model, permissions) that applies to this session. Without this, agents start every session blind and users manage tool config manually.

## Acceptance Criteria

- [ ] `ship workspace` CLI command shows resolved context for current branch
- [ ] Checkout hook creates/updates workspace record in SQLite automatically
- [ ] Workspace links branch → feature/spec/release via ID
- [ ] Workspace resolves full agent config (mode defaults + feature [agent] overrides)
- [ ] UI workspace panel shows inherited vs overridden config fields clearly
- [ ] `ship workspace --feature <id>` links an existing feature to current branch atomically (branch + link + config generation)
- [ ] Worktree workspaces are tracked separately from main branch

## Delivery Todos

- [ ] Implement `ship workspace` CLI subcommand (show current, link document)
- [ ] Expose workspace resolution in MCP (`get_workspace` tool)
- [ ] UI panel: resolved agent config with inheritance labels
- [ ] Encapsulate branch creation in `ship workspace start` (atomic: branch + link + hook trigger)
- [ ] Define environment layer on Workspace struct (worktree_path already exists; add container, env_vars for future remote)

## Notes

**Resolution chain:** project defaults → active mode → feature `[agent]` block → resolved Workspace

Workspace is SQLite-only — not a markdown file. The file-based representation is CLAUDE.md + .mcp.json, written by the post-checkout hook. No `workspace.md` needed.

Modes and workspace configs share the same schema (`AgentConfig`). A Mode is a named, pre-configured workspace preset. The `[agent]` block in feature frontmatter is a partial `AgentConfig` override applied on top of the active mode. This unification means the schema is defined once and the `[agent]` block fields are validated against it.

For remote/container use: the Workspace struct has an environment layer slot (worktree_path today, container image + env vars + ports eventually). This is the dev container spec for the branch — same concept, different runtime target.
