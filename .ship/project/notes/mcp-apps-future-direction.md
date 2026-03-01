+++
id = "a7200d55-270c-43c3-8e07-2eaa7821ee78"
title = "MCP Apps — Future Direction"
created = "2026-02-27T22:04:52.282336309Z"
updated = "2026-02-27T22:04:52.282336309Z"
tags = []
+++

# MCP Apps — Future Direction

MCP apps are a separate delivery mechanism — interactive web UIs built on the MCP protocol.
Ship's app layer will be a **Rust web project** (not Tauri).

Reference: https://modelcontextprotocol.io/extensions/apps/build

## Notes
- Separate from the desktop UI (Tauri) — browser-based, served by the Rust backend
- Natural home for Ship's project dashboard, feature catalog, release view
- Can compose with the existing MCP server surface without new tools

## Skills as an alternative surface reduction

Ship skills support scripts (`$ARGUMENTS` placeholder). Many read-heavy "tools" could instead
be skills that agents invoke via slash commands — e.g. a skill that calls `ship feature list`
and formats the output. This is worth pursuing before adding more MCP tools.

**Next work item:** Unified agent context — skills, resources, and CLAUDE.md generation unified
into a single coherent picture per branch/workspace.
