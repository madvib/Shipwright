# Handoff — v0.1.0 Session 2026-03-21

## Branch: `v0.1.0`

## What happened this session

### Completed (uncommitted on v0.1.0)
- **TUI improvements**: Events tab + detail view, reverse tab cycling (Shift+Tab), job log scrolling, capability progress bars, auto-refresh
- **DB schema split**: `schema.rs` → `schema/{mod,state,work,events,agents}.rs` with doc comments
- **DB migrations**: Conditional old event_log detection + drop/recreate. ALTER TABLE for session metrics columns.
- **Event system**: `as_db()` → public `as_str()`. Context columns with indexes.
- **Session metrics**: 6 new columns on workspace_session_record
- **Gate tracking**: record_gate_outcome, list_gate_outcomes with tests
- **File ownership**: claim_files, release_claims, check_conflicts, list_claims (9 tests)
- **CLI audit**: Removed dead Export, extracted init.rs, added 6 help topics, hidden ship surface
- **MCP**: update_target tool with tests
- **Permission presets fixed**: All 4 presets have explicit `tools_allow`. `Bash(ship *)` allowed everywhere.
- **Help agent**: `ship use help` → read-only umbrella agent. `/ship-tutorial` for onboarding, `/ship-help` for troubleshooting. Skills: cli-reference, schema-reference, permissions. Never runs `ship use`/`ship compile`.
- **Job autostart rule**: Checks `.ship-session/job-spec.md` (preferred) then CWD.

### Capability map (9 marked actual)
hbaR6ZUE, jkpsVD9S, 57TDpYFJ, wQ69bpJa, fQn266AF, bbY2ij4F, AHMsc57t, kFfZnyjT, JxszZzc6

### Tests: 312 runtime, 6 CLI, 3 MCP — all passing

### Worktrees
- `A39uK8JX` — merged, prune
- `agent-rename` — stale, prune
- `mLaHiccr` — registry rework (4 files), needs gate review
- `tutorial` — outdated (was tutor, now help agent), prune or update

## Next steps

### Immediate
1. Commit all uncommitted work on v0.1.0
2. Prune stale worktrees: A39uK8JX, agent-rename
3. Gate review mLaHiccr
4. Rebuild MCP server + reinstall CLI

### v0.1.0 gaps

**Docs (README is #1 launch deliverable):**
- Ybubd6Su — README that sells and explains
- CPYSX7xM — Getting started guide
- kWBygg8E — CLI reference with examples
- R9v5gLtZ — Schema reference
- ti6uZqBu — Architecture overview

Create a dedicated `docs-writer` agent. Must write for external devs who have 30 seconds to decide. Run real commands, show real output.

**Skills library:**
- 28VcuDyk — Skills as documentation (verify doc-skills in exports)
- TqwRaGHu — Curated starter set

**Registry:**
- u8eTPpks — Published agent spec
- YcNWSHPB — Package signing
- YVtzcbHw — JSON Schema for editor autocomplete

**Compiler:**
- gNCTZBBs — JSONC config format (NOT done, all config still TOML, defer to v0.2?)

**Workflow:**
- t9TFpfM5 — Commander showcase

### Known issues
- `ship use` from wrong CWD writes output to wrong location
- Plugin uninstall warnings when switching agents (cosmetic)
