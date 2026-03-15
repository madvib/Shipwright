# State & Persistence Audit (UI / CLI / MCP)

Date: 2026-03-10
Scope: source of truth, persistence layout, git boundaries, interface capability parity, and cleanup actions.

## 1) Current State Snapshot

- Canonical project DB path is now human-identifiable and path-independent: `~/.ship/state/ship-hrvmuz4p/ship.db` (`<project-name>-<project-id>`).
- Project and global state are still split across SQLite, TOML, and JSON files.
- Event/log truth is SQLite (`event_log`); NDJSON is export-only.
- UI reactivity still relies on filesystem watching (`ship.toml` + sqlite file writes).
- The old secondary TOML fallback has been removed from runtime config lookup; config lookup is now `ship.toml`, then legacy `config.toml`.
- In this Codex environment, there is no direct Ship MCP bridge in the session (`list_mcp_resources` returns no configured servers).

## 2) Capability Surface by Interface

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
| Release delete | No | UI path only | No |
| ADR create/list/get/move | Yes | Yes | Create only |
| ADR update/delete | No | Yes | No |
| Notes create/list/get/update | Yes | Yes | Yes (create/update) |
| Notes delete | No | Yes | No |
| Workspace/session lifecycle | Yes | Yes | Yes |
| Event list/ingest/export | Yes | Yes | Read via `ship://events` |

### MCP read surfaces

MCP is read-heavy via resources:
`ship://project_info`, `ship://features`, `ship://releases`, `ship://specs`, `ship://adrs`, `ship://notes`, `ship://skills`, `ship://workspaces`, `ship://sessions`, `ship://modes`, `ship://providers`, `ship://log`, `ship://events`.

## 3) Source-of-Truth Matrix

| Domain | Canonical store today | Secondary/derived store | Notes |
|---|---|---|---|
| Project identity (`id`) | `.ship/ship.toml` | Used to derive slug key `<name>-<id>` | Still required before DB open |
| Project core metadata (`version`, `id`, `name`, `description`) | `.ship/ship.toml` | Runtime structs | File-first, identity-focused |
| Project runtime settings (`providers`, `active_mode`, `hooks`, `statuses`, `ai`, `git`, `namespaces`) | Project SQLite (`agent_runtime_settings`) | Migrated out of `ship.toml` | DB-first |
| MCP server registry | `.ship/agents/mcp.toml` | Indexed in SQLite artifact registry | File-first + DB index |
| Modes | Project SQLite (`agent_mode`) | Resolved refs from file catalog | DB-first |
| Skills (project) | `.ship/agents/skills/<id>/SKILL.md` | Indexed for mode resolution | Canonical project skill store |
| Skills (global) | `~/.ship/skills/<id>/SKILL.md` | Seeded built-ins | Canonical user/global skill store |
| Skill cache / legacy compatibility | `~/.ship/projects/<slug>/skills` and `.ship/skills` | Read+promote into `.ship/agents/skills` | Legacy migration path only (not canonical) |
| Features | SQLite `feature*` tables | Markdown under `.ship/project/features/...` | Dual-write drift risk |
| Releases | SQLite `release*` tables | Markdown under `.ship/project/releases/...` | Dual-write drift risk |
| ADRs | SQLite `adr*` tables | Markdown under `.ship/project/adrs/...` | Dual-write drift risk |
| Specs | SQLite `spec` table | Virtualized `file_name/path` in some APIs | Model mismatch remains |
| Notes | SQLite `note` table | Legacy markdown import path | DB-first |
| Vision | `.ship/project/vision.md` | None | File-only |
| Events/log | SQLite `event_log` | NDJSON export (`ship event export`) + snapshot index | DB-first |
| Global tracked projects | `~/.ship/projects.json` | None | JSON file, not in global DB |
| Global active/recent project | `~/.ship/app_state.json` | None | JSON file, not in global DB; paths are canonicalized+deduped on load |

### Skill storage policy (canonical)

- Project-scoped skills: `.ship/agents/skills/<id>/SKILL.md`
- User/global skills: `~/.ship/skills/<id>/SKILL.md`
- Legacy cache path `~/.ship/projects/<slug>/skills` is migration-only and should not be treated as source of truth.
- Online/catalog installs do not maintain a separate persistent cache-of-record; installed output is copied into one of the canonical directories above.

## 4) Git Boundary (What Is / Isn’t in Git)

### Never in git (global/runtime)

- `~/.ship/**` global state (`ship.db`, per-project DBs under `state/`, user notes DB rows, etc.)
- Runtime process/session internals outside repo root

### Repository policy applied in this repo

This repo now tracks controlled `.ship/` state:

- Tracked: `.ship/ship.toml`, `.ship/.gitignore`, `.ship/agents/mcp.toml`, `.ship/agents/permissions.toml`, `.ship/agents/rules/*.md`
- Removed from git: skills, vision/docs, feature/release/adr/spec markdown, legacy templates

## 5) Event Stream Reality

- Canonical event stream is SQLite `event_log`.
- `log_action*` writes event rows; rendered logs are derived.
- `events.ndjson` is export-only (not a live append sink).
- External filesystem edits enter `event_log` only through ingest (`event ingest`).
- Snapshot diff index persists at `.ship/generated/event_index.json`.

## 6) Main Conflicts Still Open

1. **`ship.toml:id` remains a bootstrap dependency**
- DB path resolution still needs TOML read before DB connection.

2. **Global state is still split (DB + JSON)**
- `projects.json` and `app_state.json` remain outside global SQLite.

3. **UI reactivity is still FS-watch-based**
- Current model depends on file notifications and debounce behavior.

4. **Issue deprecation is partial in surface area**
- Issue-centric docs/examples/plugins still exist in parts of the repo, even after persistence cleanup.

5. **Spec storage model still has dual semantics**
- DB-native specs coexist with file-oriented assumptions in some APIs/docs.

## 7) Cleanup Executed (2026-03-10)

### DB key and compatibility

- Project DB key now resolves as slug `<project-name>-<project-id>`.
- Runtime includes one-time promotion from legacy ID-only dir (`state/<id>/ship.db`) to slug dir (`state/<slug>/ship.db`) when needed.

### On-disk archival + deletion

Backups created at:

- `~/.ship/state/backups/20260310-151838-legacy-cleanup/`

Archived then removed from active state:

- `~/.ship/state/C-Users-Micah-Desktop-Dev-Ship`
- `~/.ship/state/Users-micahcotton-dev-landing-page`
- `~/.ship/state/Users-micahcotton-dev-<legacy-repo-slug>`
- `~/.ship/projects/C-Users-Micah-Desktop-Dev-Ship`
- `~/.ship/projects/Users-micahcotton-dev-<legacy-repo-slug>`

Current active canonical DB:

- `~/.ship/state/ship-hrvmuz4p/ship.db`

## 8) Cross-Platform Sync Policy (Immediate)

1. Sync one canonical project DB per project slug:
   - `~/.ship/state/<project-name>-<project-id>/ship.db` (+ `-wal`/`-shm` if present)
2. Do not sync as shared truth:
   - `~/.ship/app_state.json`
   - `~/.ship/projects.json`
   - `~/.ship/*.sync-conflict-*`
   - Rationale: these are machine-local UX pointers; runtime now self-heals path canonicalization for local renames.
3. Keep `.ship/ship.toml` project identity stable across machines.
4. Treat path-keyed state dirs under `~/.ship/state/` as legacy and archive/remove after verification.
