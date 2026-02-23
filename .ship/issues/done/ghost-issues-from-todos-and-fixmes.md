+++
title = "Ghost Issues from TODOs and FIXMEs"
created = "2026-02-22T05:30:30.895196016Z"
updated = "2026-02-22T05:30:30.895196986Z"
tags = []
links = []
+++

Automatically scan the codebase for TODO and FIXME comments and surface them as suggested ghost issues in the UI, making it easy to convert them into tracked work.

## Implementation — 2026-02-22

**New crate:** `crates/plugins/ghost-issues`
- Scans using `ignore` crate (respects `.gitignore`, skips `target/`, `node_modules/`, `.ship/`)
- Detects TODO, FIXME, HACK, BUG with optional `(author):` context
- `promote` converts a ghost comment to a real backlog issue
- Storage: `.ship/plugins/ghost-issues/last-scan.json`

**Changed files:**
- `crates/cli/src/lib.rs` — `ship ghost scan|report|promote`
- `crates/mcp/src/lib.rs` — `ghost_scan`, `ghost_report`, `ghost_promote` tools
- `Cargo.toml` — added `crates/plugins/ghost-issues` to workspace