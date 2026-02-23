+++
title = "Fix MCP Connection"
created = "2026-02-23T01:28:41.140379955Z"
updated = "2026-02-23T01:28:41.140380355Z"
tags = []
links = []
+++

# Fix MCP Connection

## Description
Resolve the invalid character error in MCP initialization and implement dynamic project resolution.

## Resolution
- Switched from `pnpm` to `node` in `mcp_config.json` to avoid log pollution.
- Implemented upward-scanning `.project` resolution in `getProjectDir()`.
- Verified with unit tests.

## Tasks
- [ ] Initial task

## Links
-
