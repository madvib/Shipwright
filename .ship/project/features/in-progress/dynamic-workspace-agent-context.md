+++
id = "eZKy7Tym"
title = "Dynamic Workspace Agent Context"
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

When a developer switches branches, the agent context should switch with them — automatically, without any manual steps. The post-checkout git hook fires on every `git checkout`, reads the branch, finds the linked feature, resolves the workspace agent config, and writes `CLAUDE.md` and `.mcp.json` to the correct locations. The agent wakes up already knowing what spec is active, which tools to use, and what the acceptance criteria are.

## Acceptance Criteria

- [ ] `post-checkout` hook installed by `ship init` (or `ship git install-hooks`)
- [ ] Hook calls `ship git post-checkout` with old/new branch refs
- [ ] Branch lookup finds feature where `branch == current_branch`
- [ ] Resolves full workspace agent config (mode + feature `[agent]` overrides)
- [ ] Writes `CLAUDE.md` to repo root with: vision excerpt, feature Why/AC/Todos, open issues, skill content
- [ ] Writes `.mcp.json` to repo root with resolved MCP server list
- [ ] Hook runs silently on success; prints warning on lookup miss (not error)
- [ ] `ship git sync` manually re-triggers hook logic for current branch
- [ ] Worktree support: writes to worktree root, not main repo root

## Delivery Todos

- [ ] Fix: hook currently writes CLAUDE.md to main repo root in worktrees (issue filed)
- [ ] Fix: `on_post_checkout` hardcodes "claude" — implement multi-provider dispatch
- [ ] Fix: `generate_claude_md` + `export_claude` both write CLAUDE.md — second clobbers first
- [ ] Branch lookup must search specs and releases, not only features (backlog issue filed)
- [ ] Implement teardown when switching to a non-feature branch (remove stale CLAUDE.md/.mcp.json)
- [ ] Wire active mode into CLAUDE.md generation header

## Notes

This is the delivery mechanism for Scoped Workspaces — the hook reads the Workspace data model and renders it as files. Sister feature to Scoped Workspaces (data model) and Unified Agent Configuration Standard (schema). The three together form the complete branch-aware agent context system.
