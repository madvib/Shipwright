+++
id = "MnHYXJRX"
title = "Cloud workspaces — agent execution"
status = "planned"
created = "2026-03-02T17:12:11.592910150Z"
updated = "2026-03-02T17:12:11.592910150Z"
release_id = "v0.3.0"
adr_ids = []
tags = []

[agent]
mcp_servers = []
skills = []
+++

## Why

The tier 3 unlock: an agent gets a full dev environment in the cloud — repo checked out, toolchain installed, running and streaming output live to the UI. Teams can spin up parallel workspaces without local machine constraints. This is the primary revenue driver.

## Acceptance Criteria

- [ ] Cloud workspace provisions in under 60 seconds
- [ ] Agent output streams to UI in real-time via Rivet Actor WebSocket
- [ ] Multiple team members see live workspace status simultaneously
- [ ] Containers are isolated per workspace, auto-terminate on idle
- [ ] Workspace type (feature/refactor/experiment/hotfix) determines container config
- [ ] Ship Cloud manages billing per active workspace-hour

## Delivery Todos

- [ ] Rivet Sandbox integration: provision isolated container per workspace
- [ ] Daytona or equivalent: repo checkout + toolchain install on container start
- [ ] Rivet Actor as coordinator: workspace lifecycle, output streaming, status
- [ ] WebSocket stream: agent stdout/stderr → Rivet Actor → UI
- [ ] Container config: devcontainer.json support for toolchain detection
- [ ] Idle detection: auto-suspend after N minutes of no activity
- [ ] Resume: restore container state from snapshot, reconnect WebSocket
- [ ] Billing hooks: track workspace-hours per project per account
- [ ] UI: cloud workspace panel, live terminal output, status indicators

## Notes

Execution stack: Rivet Sandbox (isolation + resource limits) + Daytona (dev environment setup). Rivet Actor bridges execution ↔ persistence ↔ UI. CF DO considered but ruled out — not self-hostable, can't offer enterprise on-prem tier.
