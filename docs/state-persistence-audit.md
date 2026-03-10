# State & Persistence Audit (UI / CLI / MCP)

Date: 2026-03-10
Scope: source of truth, persistence layout, git boundaries, interface capability parity, and deprecation fallout.

## 1) Current State Snapshot

- Issue tracking APIs are now removed from active UI/CLI/MCP surfaces.
- Project and global state remain split across SQLite, TOML, and JSON files.
- Event/log truth is SQLite (`event_log`); NDJSON is export-only.
- UI reactivity still relies on filesystem watching (ship config file + sqlite file writes).
- In this Codex environment, Ship MCP was audited from repo/runtime code and local binaries; there is no direct MCP client bridge tool available to this agent session.

## 2) Capability Surface By Interface

### Planning/workflow entities

| Capability | CLI | UI | MCP |
|---|---|---|---|
| Project open/switch/init | Yes (`init`, `projects`, `ui`) | Yes | Yes (`open_project`) |
| Feature create/list/get/update | Yes | Yes | Yes (create/update) |
| Feature status transitions | Yes (`start`, `done`) | Yes | No explicit tool |
| Feature delete | Yes | Yes | No |
| Spec create/list/get/update | Yes | Yes | Yes (create/update) |
| Spec move/delete | No | Yes | No |
| Release create/list/get/update | Yes | Yes | Yes (create/update) |
| Release delete | No | UI not exposed in CLI path | No |
| ADR create/list/get/move | Yes | Yes | Create only |
| ADR update/delete | No | Yes | No |
| Notes create/list/get/update | Yes | Yes | Yes (create/update) |
| Notes delete | No | Yes | No |
| Workspace/session lifecycle | Yes | Yes | Yes |
| Event list/ingest/export | Yes | Yes | Read via `ship://events` |

### Read surfaces in MCP

MCP is read-heavy via resources, not many read tools:
`ship://project_info`, `ship://features`, `ship://releases`, `ship://specs`, `ship://adrs`, `ship://notes`, `ship://skills`, `ship://workspaces`, `ship://sessions`, `ship://modes`, `ship://providers`, `ship://log`, `ship://events`.

## 3) Source-Of-Truth Matrix

| Domain | Canonical store today | Secondary/derived store | Notes |
|---|---|---|---|
| Project identity (`id`) | `.ship/ship.toml` | Used to derive DB path `~/.ship/state/<id>/ship.db` | Hard dependency before DB open |
| Project config core (`name`, `description`, `statuses`, `git`, `namespaces`) | `.ship/ship.toml` | Loaded into runtime structs | Still file-first |
| Project runtime agent settings (`providers`, `active_mode`, `hooks`) | Project SQLite (`agent_runtime_settings`) | `ship.toml` intentionally stripped on save | DB-first |
| MCP server registry | `.ship/agents/mcp.toml` | Indexed in SQLite artifact registry | File-first with DB index |
| Modes | Project SQLite (`agent_mode`) | Resolved refs from file catalog | DB-first |
| Skills (project) | `.ship/skills/<id>/SKILL.md` | Indexed for mode ref resolution | File-first |
| Skills (global) | `~/.ship/skills/<id>/SKILL.md` | Project init seeds built-ins here | File-first |
| Legacy skills location | `~/.ship/projects/<slug>/skills/` | Migrated on access | Legacy fallback |
| Features | SQLite `feature*` tables | Markdown files under `.ship/project/features/...` | Dual-write / drift risk |
| Releases | SQLite `release*` tables | Markdown files under `.ship/project/releases/...` | Dual-write / drift risk |
| ADRs | SQLite `adr*` tables | Markdown files under `.ship/project/adrs/...` | Dual-write / drift risk |
| Specs | SQLite `spec` table | `file_name/path` are virtual; markdown files typically absent | Model mismatch |
| Notes (project/user) | SQLite `note` table (project DB or global DB) | Legacy markdown import path | DB-first |
| Vision | `.ship/project/vision.md` | None | File-only |
| Events/log | SQLite `event_log` | NDJSON export (`ship event export`), generated snapshot index | DB-first |
| Global tracked projects | `~/.ship/projects.json` | None | JSON file, not in global DB |
| Global active/recent project | `~/.ship/app_state.json` | None | JSON file, not in global DB |

## 4) Git Boundary (What Is / Isn’t In Git)

### Never in git (global/runtime)

- `~/.ship/**` global state (`ship.db`, per-project DBs under `state/`, user notes DB rows, etc.)
- Runtime process/session internals not under repo root.

### Project-local (`.ship/`) default behavior

Generated `.ship/.gitignore` now ignores by default:

- `workflow/specs`
- `project/features`
- `project/releases`
- `project/adrs`
- `project/notes`
- `generated/`
- `.tmp-global/`

And leaves committed by default:

- `agents`
- `ship.toml`
- `templates`
- `vision` (`project/vision.md`)

### Repo-level ignores

Root `.gitignore` excludes generated client artifacts (`CLAUDE.md`, `.mcp.json`, provider dirs, etc.) and legacy `.ship/ship.db*` patterns.

## 5) Event Stream Reality

- Canonical event stream is SQLite `event_log`.
- `log_action*` writes event rows; human log output is derived from those rows.
- `events.ndjson` is export-only (not a live sink).
- External filesystem changes are ingested into the same `event_log` only when `ingest_external_events` runs.
- Snapshot state for filesystem diffing is persisted at `.ship/generated/event_index.json`.

## 6) Main Conflicts To Resolve

1. **Issue deprecation is not fully removed from persistence model**
- `issue` table still exists in project schema.
- `EventEntity::Issue` remains for compatibility parsing.
- Migration still moves legacy `issues/` trees and `ISSUE.md` template.
- Hidden legacy plugins (`ghost-issues`, `time-tracker`) still use issue terminology.

2. **Spec storage model is inconsistent**
- Specs are DB-native now, but namespace/layout still implies markdown-backed spec files.
- Spec file names/paths are virtual in DB responses; files are usually absent.

3. **`ship.toml:id` is a hard runtime dependency**
- DB path resolution requires TOML read before DB connection.
- If this ID is considered non-authoritative, a new bootstrap keying mechanism is needed.

4. **Global state is split across DB + JSON files**
- `projects.json` and `app_state.json` live outside SQLite.
- Global DB has `global_state` table but is not currently the single source for these records.

5. **Reactivity is file-watch based at UI boundary**
- Tauri watcher reacts to file changes (`ship.toml`, sqlite file writes), not to a typed in-process event bus.
- Performance and correctness depend on FS notifications and debounce timing.
- Runtime perf counters expose `watcher_ingest_*`, but ingestion is not wired into that watcher loop.

6. **Tracked repo `.ship/` contains legacy/placement drift**
- Examples include misplaced files at status-root levels and duplicate vision naming conventions.
- These create ambiguity for migration/import and contributor expectations.
  - `.ship/project/adrs/sqlite-as-canonical-data-store---markdown-as-agent-export.md` (not under a status folder)
  - `.ship/workflow/specs/ship-workspace-project-manager.md` (not under `draft/active/archived`)
  - `.ship/project/VISION.md` and `.ship/project/vision.md` coexist

## 7) Recommended Target Model (Concrete)

1. **Single canonical store for mutable runtime state: SQLite**
- Keep markdown as export/import interoperability layer, not required canonical state.

2. **Unify global state into SQLite**
- Move `projects.json` and `app_state.json` into global DB tables.
- Keep file export/import only for backup/debug.

3. **Replace `ship.toml:id` bootstrap dependency**
- Add global registry table mapping canonical project path -> stable project_id.
- Keep `ship.toml:id` as optional mirror during migration, then de-emphasize.

4. **Replace FS-driven UI reactivity with event subscription from runtime writes**
- Emit typed change events directly from mutation paths (`append_event`, config saves, workspace/session writes).
- Keep file-watch fallback only for truly external edits.

5. **Finish issue deprecation at schema/plugin layer**
- Mark `issue` table as legacy-only and stop new writes.
- Remove/rename hidden issue-centric plugins or migrate them to neutral work-item semantics.

6. **Clarify spec persistence policy explicitly**
- Either write spec markdown exports consistently, or remove virtual-file assumptions from API/UI and docs.

## 8) Immediate Cleanup Candidates (Low Risk)

- Remove stale issue references from docs/examples/help text that still imply issue CRUD exists.
- Normalize tracked `.ship/` files to status-directory conventions.
- Keep only one vision filename convention (`project/vision.md`).
