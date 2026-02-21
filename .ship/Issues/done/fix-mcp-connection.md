---
title: Fix MCP Connection
status: done
created: 2026-02-21T17:58:24.698Z
links: []
---

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
