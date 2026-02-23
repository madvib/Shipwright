+++
title = "Git commit customization"
created = "2026-02-22T06:31:13.845820517Z"
updated = "2026-02-22T06:41:41.782446895Z"
tags = []
links = []
+++

Allow users to control what Ship data gets committed to git. Some want issues/ADRs tracked in the repo (team sharing), others want .ship/ fully local. Implement via a managed .ship/.gitignore generated from config. Options: commit_issues, commit_adrs, commit_log, commit_plugins, commit_config. CLI: ship git include/exclude. MCP: git_config tools. ship init should write a sensible default.

## Implementation — 2026-02-22

**Changed files:**
- `crates/logic/src/config.rs` — `GitConfig` struct, `generate_gitignore`, `get_git_config`, `set_git_config`
- `crates/logic/src/project.rs` — `init_project` writes default `.ship/.gitignore`
- `crates/logic/src/lib.rs` — exported `GitConfig`, `generate_gitignore`, `get_git_config`, `set_git_config`
- `crates/cli/src/lib.rs` — `ship git status|include|exclude`
- `crates/mcp/src/lib.rs` — `git_config_get`, `git_config_set` tools

**Note:** gitignore only affects git operations, not MCP/filesystem reads. Agents always see all files.
