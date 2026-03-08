+++
id = "KVVaSUTx"
title = "Pre-defined Agent Modes"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T22:33:22.806086+00:00"
release_id = "v0.1.0-alpha"
active_target_id = "v0.1.0-alpha"
spec_id = ""
branch = "feature/pre-defined-agent-modes"
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Modes are the operator-facing control surface for switching agent behavior by task type without hand-editing config files.

## Acceptance Criteria

- [x] Built-in baseline modes are seeded for planning/code/config workflows
- [x] Mode state persists in canonical runtime storage
- [x] Mode changes trigger active config recompute and provider sync behavior
- [x] Provider fallback behavior is deterministic when target agents are unspecified
- [ ] Mode-management MCP tools are complete and tested
- [ ] UI mode management surface is complete and launch-polished
- [ ] Tool-level enforcement of mode restrictions is hard-gated in execution paths

## Delivery Todos

- [x] Seed and persist default modes
- [x] Wire mode-set operations to runtime sync pipeline
- [x] Add tests for mode/provider target selection fallback
- [ ] Complete MCP mode-management APIs and e2e coverage
- [ ] Complete UI mode control surface with diagnostics
- [ ] Enforce active-tool policy at invocation boundaries

## Current Behavior

Mode control is functionally active but still in-progress for management UX and strict enforcement gates.

## Notes

Mode is the control-plane switch, not a PM status primitive.