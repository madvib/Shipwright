# Alpha E2E Feature Matrix

This is a working matrix to compare desired project-module behavior against current implementation.

| Requirement | Current Status | Validation Path | Gap / Note |
| --- | --- | --- | --- |
| Workspace-first flow | Implemented (CLI/MCP/UI) | `ship init`, `ship release/*`, `ship feature/*`, `ship workspace/*`, `ship workspace session/*` | Keep extending cross-surface parity tests |
| Feature lifecycle + status signals | Partial | Feature CRUD + docs commands + e2e status assertions | Typed `Declaration/Status/Delta` model still in progress |
| ADR as separate module | Implemented | `ship adr create/list/get/move` + UI ADR route | Better feature/capability linking UX needed |
| Choose what is git committed | Partial | `ship git include/exclude <category>` | No branch/worktree commit policy yet |
| Always-ignored temp scratchpad | Implemented (workspace-level) | `.ship/` is ignored in this example folder | Needs productized scratchpad primitive |
| MCP headless workflows | Implemented | Start with `ship mcp` and use tools over stdio | Workflow policy context injection still limited |
| DB-canonical entities + markdown projection | Partial | e2e entity tests (`feature`, `release`, `adr`, `note`) | Remove remaining assumptions that markdown is source of truth |
| Link features/releases/ADRs | Partial | Link model exists in logic | UI link editing/type alignment needs hardening |
| Tags + sortable metadata | Partial | Frontmatter exists and can be extended | No complete tag/filter/sort UX yet |
| Kanban + visual workflow | Partial | UI status lanes exist | DnD and richer board interactions need polish |
| Activity log | Implemented | Event stream (`events.ndjson`) + UI activity route | Needs stronger event coverage/filters |
| Append-only event stream | Implemented | `.ship/events.ndjson` + `ship event list --since ...` + workspace/session events in e2e | More assertions for error and rollback paths |

## Suggested Alpha Focus

1. Harden feature Declaration/Status/Delta model with tests.
2. Complete mode + agent configuration UX flows.
3. Tighten MCP + CLI parity for workspace/session operations.
4. Improve workspace page performance and reactivity.
