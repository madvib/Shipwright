+++
id = "30effd39-4eb8-4cbe-8394-f85fbb1ca135"
title = "ship doctor command"
created = "2026-02-27T03:08:20.373037875Z"
updated = "2026-02-27T03:08:20.373037875Z"
tags = []
+++

# ship doctor

Diagnostic command that validates the Ship installation and project state.

## Checks to include

### Environment
- Ship binary version + build info
- Rust toolchain (if source install)
- Git version
- HOME, SHIP_DIR env vars

### Global state
- `~/.ship/` exists and is readable
- `~/.ship/projects.json` valid JSON, all registered paths still exist
- Global DB at `~/.ship/ship.db` accessible + migrations current
- Global skills dir exists

### Project (if in a project)
- `.ship/` found and readable
- `ship.toml` parses without error
- All namespace dirs exist (project/, workflow/, agents/, generated/)
- Project DB at `~/.ship/state/<slug>/ship.db` accessible + migrations current
- All declared MCP server commands are on PATH
- All declared skill IDs resolve (no dangling refs in ship.toml [agent])
- Git hooks installed (post-checkout present + executable)
- Generated files gitignored (CLAUDE.md, .mcp.json)

### Agent config
- `.mcp.json` parses if present
- All servers in .mcp.json are declared in ship.toml (detect drift)
- mcp_managed_state.toml is consistent with .mcp.json

## Output format
- ✓ / ✗ / ⚠ per check
- Summary line: "X checks passed, Y warnings, Z errors"
- `--fix` flag for auto-repairing safe issues (reinstall hooks, regenerate gitignore)
