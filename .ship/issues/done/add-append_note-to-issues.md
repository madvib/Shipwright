+++
title = "Add append_note to issues"
created = "2026-02-22T06:31:13.827566630Z"
updated = "2026-02-22T06:41:22.951631320Z"
tags = []
links = []
+++

Add an append_note(path, note) function to logic so agents and CLI can append implementation notes to issues without reading and rewriting the whole file. Expose via MCP as append_to_issue and as ship issue note <file> <text>.

## Implementation — 2026-02-22

**Changed files:**
- `crates/logic/src/issue.rs` — added `append_note(path, note)`
- `crates/logic/src/lib.rs` — exported `append_note`
- `crates/cli/src/lib.rs` — `ship issue note <file> [--status] <note>`
- `crates/mcp/src/lib.rs` — `append_to_issue` tool
