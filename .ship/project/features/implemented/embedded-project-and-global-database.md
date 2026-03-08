+++
id = "sfCQ4U3x"
title = "Embedded Project and Global Database"
created = "2026-02-28T15:56:07Z"
updated = "2026-03-07T21:49:48.399090+00:00"
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

Ship needs durable, queryable runtime state for project and user scopes. Embedded SQLite provides transactional local persistence without external infrastructure.

## Acceptance Criteria

- [x] Project-scoped SQLite state is used for runtime/planning/workspace/session data
- [x] Global/user-scoped SQLite state is used for cross-project records
- [x] Runtime state DB APIs are centralized in runtime state modules
- [x] Module operations use those APIs instead of ad hoc storage paths
- [x] Project/global DB separation is enforced by scope-aware operations

## Delivery Todos

- [x] Maintain centralized DB open/read/write primitives
- [x] Keep project and global scope handling explicit in module ops
- [x] Validate migration and open paths across CLI/UI/MCP surfaces

## Current Behavior

Embedded SQLite underpins Ship runtime state and planning entities. Scope-aware note/config/project operations already rely on project/global DB separation.

## Follow-ups

- Continue decomposing oversized schema/runtime files into stitched modules.