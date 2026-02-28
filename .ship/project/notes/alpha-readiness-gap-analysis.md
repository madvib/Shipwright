+++
id = "e26c4e57-daab-45f3-a753-6dcae64f9926"
title = "Alpha Readiness — Gap Analysis"
created = "2026-02-27T16:09:32.103959286Z"
updated = "2026-02-27T16:09:32.103959286Z"
tags = []
+++

# Alpha Readiness — Gap Analysis

_Against alpha-spec.md done criteria. Last updated: 2026-02-27_

---

## Done criteria status

| # | Criterion | Status | Notes |
|---|-----------|--------|-------|
| 1 | `ship init` < 10s, hooks installed, .gitignore updated | ✅ Done | Hooks + gitignore implemented, e2e tested |
| 2 | `ship note "..."` instant capture | ✅ Done | CLI works |
| 3 | Note promoted to spec | ❌ Not implemented | `note promote` not wired |
| 4 | Spec refined in split view with AI | ❌ UI only | Spec split editor not built |
| 5 | Issues extracted from spec via button or MCP sampling | ⚠️ Partial | MCP sampling not wired; extract_issues tool exists |
| 6 | Issues on Kanban | ❌ UI only | Kanban not built |
| 7 | Drag-and-drop moves issue file | ❌ UI only | |
| 8 | `git checkout` fires hook, CLAUDE.md appears | ✅ Done | Tested e2e |
| 9 | Connect to Ship MCP server | ✅ Done | `ship mcp start --stdio` works |
| 10 | Agent reads spec + issues via CLAUDE.md | ✅ Done | Context generation works |
| 11 | Agent updates issue → appears in Kanban in 1s | ❌ UI only | No live file watch yet |
| 12 | log.md coherent history | ✅ Done | events.ndjson tracked |
| 13 | Pre-commit blocks CLAUDE.md / .mcp.json | ✅ Done | Hook + gitignore tested |
| 14 | `ship worktree create` | ❌ Not implemented | Worktree awareness skeletal |
| 15 | `ship tools export --target claude` | ⚠️ Partial | `ship git sync` does it; CLI naming inconsistent |
| 16 | No account, no internet, one binary | ✅ Done | |

---

## Critical path items (block alpha)

### 1. Feature data model + namespace move
**Impact:** Features are the product backbone. Wrong namespace (.workflow/ vs .project/) and wrong status model (active vs planned/in-progress/implemented/deprecated) block the catalog, changelog, and agent context use cases.
- Move `features_dir` → `.ship/project/features/`
- Add `version`, `supersedes`, `description` fields
- Update status enum
- Migration for existing files

### 2. `ship feature start <id>` + `ship feature done <id>`
**Impact:** Without encapsulated branch creation, users manually create branches and the UUID→branch link is never established. String matching on branch names is brittle.
- `start`: creates branch, writes UUID→branch to SQLite, checks out
- `done`: marks implemented, sets `version` from active release filename

### 3. Branch → feature link via UUID (SQLite)
**Impact:** Every checkout that can't resolve a feature via UUID falls back to string matching (currently hardcoded branch name in frontmatter). Works only if you used `ship feature start`.
- `branch_context` table: `{branch_name, feature_id, feature_file, last_synced}`
- `ship feature start` populates it
- `on_post_checkout` queries it first, falls back to frontmatter `branch` field

### 4. Worktree: CLAUDE.md written to wrong root
**Impact:** Worktrees generate CLAUDE.md in main repo root, not worktree root. Context is wrong for the working directory.
- `on_post_checkout` needs `worktree_root: Option<&Path>` param
- Post-checkout hook must detect if running inside a worktree and pass correct root

### 5. `generate_claude_md` + `export_claude` double-write
**Impact:** Second write clobbers first. Agent gets incomplete context — either missing skills/MCP section or missing feature spec section.
- Merge into single write pipeline

### 6. `on_post_checkout` hardcodes "claude"
**Impact:** Gemini and Codex users get no agent config generated.
- Use PROVIDERS registry for multi-provider dispatch

---

## Secondary items (alpha quality, not blocking)

| Item | Impact |
|------|--------|
| `get_feature_catalog` MCP tool | Agent self-awareness of what product does |
| Feature-level MCP server filter not propagated | Minor — all servers written regardless of feature config |
| SHIPWRIGHT.md never read (dead artifact) | Cleanup / remove or fold into CLAUDE.md |
| `ship feature list --status` CLI | Nice-to-have for CLI users |
| Note promote → spec | Spec says it's alpha but untested end-to-end |
| Stale `.ship/specs/` directory | Confusion from old namespace |
| `mcp_managed_state.toml` → SQLite | Nice architectural cleanup, not urgent |

---

## What "alpha ready" actually means

The core loop that must work **without any workarounds**:

```
ship init → git checkout → CLAUDE.md → agent reads context →
agent creates/moves issues → ship feature start → branch created →
checkout fires → correct context → agent works → ship feature done →
feature marked implemented
```

Everything else (UI, catalog, changelog, worktrees) is additive. The MCP + git hook + feature lifecycle is the core.

**My recommendation:** focus the next 2-3 sessions on:
1. Feature model + namespace move (this session)
2. `ship feature start/done` + SQLite branch context table (next session)
3. CLAUDE.md double-write fix + multi-provider dispatch (after that)

Then alpha is unblocked.
