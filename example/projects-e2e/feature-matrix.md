# Alpha E2E Feature Matrix

This is a working matrix to compare desired project-module behavior against current implementation.

| Requirement | Current Status | Validation Path | Gap / Note |
| --- | --- | --- | --- |
| Single opinionated alpha workflow | Implemented (CLI/MCP/UI) | `Vision -> Release -> Feature -> Spec -> Issues -> ADRs -> Close Feature -> Ship Release` + `ship release/*` + `ship feature/*` + `/project/releases` + `/project/features` | Continue hardening with e2e + UI tests |
| Custom issue/category names | Partial | Custom statuses are supported in CLI/UI | Git category model is still fixed (`issues`, `releases`, `features`, `specs`, `adrs`, `log.md`, `events.ndjson`, `config.toml`, `templates`, `plugins`) |
| Spec lifecycle | Partial | Specs exist as markdown docs and can be created/listed via CLI/MCP | Need explicit lifecycle states and issue linkage/rollups |
| ADR as separate module | Implemented | `ship adr create ...` and UI ADR route | Need clearer cross-linking UX with issues/specs |
| Choose what is git committed | Partial | `ship git include/exclude <category>` | No branch/worktree commit policy yet |
| Always-ignored temp scratchpad | Implemented (workspace-level) | `.ship/` is ignored in this example folder | Needs productized scratchpad primitive |
| MCP headless workflows | Implemented | Start with `ship mcp` and use tools over stdio | Workflow policy context injection still limited |
| Documents + IO as root primitives | Partial | Issue/spec/adr/feature/release markdown files are source of truth | Rich editor upgraded locally; swap to mdx-editor when dependency fetch is available |
| Link issues/specs/ADRs | Partial | Link model exists in logic | UI link editing/type alignment needs hardening |
| Tags + sortable metadata | Partial | Frontmatter exists and can be extended | No complete tag/filter/sort UX yet |
| Kanban + visual workflow | Partial | UI status lanes exist | DnD and richer board interactions need polish |
| Activity log | Implemented | `.ship/log.md` + UI activity route | Needs stronger event coverage/filters |
| Append-only event stream | Implemented (v1 + ingest) | `.ship/events.ndjson` + `ship event list --since ...` + `ship event ingest` + MCP `list_events`/`ingest_events` | Headless direct edits still need explicit `event ingest` unless UI watcher is running |

## Suggested Alpha Focus

1. Harden project primitives and link integrity with tests.
2. Complete mode + agent configuration UX flows.
3. Tighten MCP + CLI parity for mode and config operations.
4. Improve project board ergonomics (visuals + interaction).
