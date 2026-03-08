+++
id = "TyEDHXCL"
title = "Auto-Detection of System Agents"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T22:33:22.043399+00:00"
release_id = "v0.1.0-alpha"
active_target_id = "v0.1.0-alpha"
spec_id = ""
branch = ""
tags = []

[agent]
model = "claude"
max_cost_per_session = 10.0
mcp_servers = []
skills = []
+++

## Why

Automatic provider detection keeps the control plane aligned with what is actually installed and executable on the machine.

## Acceptance Criteria

- [x] Provider detection reports installed status and available CLI/provider metadata
- [x] Detection results are consumable by settings/workspace/session surfaces
- [x] Provider selection gracefully handles missing providers with explicit feedback
- [x] Detection does not require network access

## Delivery Todos

- [x] Keep provider discovery in runtime and expose through UI/CLI APIs
- [x] Normalize provider IDs to shared runtime enum/model
- [x] Surface provider availability in workspace/session launch paths

## Current Behavior

Provider detection is active and integrated into workspace/session control-plane behavior.

## Follow-ups

- Add explicit remediation UX when expected providers are missing or misconfigured.