# Ship Feature Process Detail

> Per-feature breakdown: what each capability does, the exact process it runs,
> what gets written where, validations enforced, and side effects.
>
> Source-read: 2026-03-09. Covers ops layer, CRUD layer, DB writes, event writes.

## Capability audit snapshot (2026-03-08)

This section is the operational map we should keep aligned as we iterate.

- `✅` = surfaced and wired on that surface
- `🟡` = partial or surfaced indirectly
- `⚪` = not surfaced

### UI, CLI, MCP mapping by documented capability

| Capability | CLI | UI | MCP |
| --- | --- | --- | --- |
| `create_feature` | ✅ `feature create` | ✅ feature create sheet with release/spec/branch initialization | ✅ `create_feature` |
| `feature_start` | ✅ `feature start` | ✅ `feature_start_cmd` + status transition control | ⚪ |
| `feature_done` | ✅ `feature done` | ✅ `feature_done_cmd` + status transition control | ⚪ |
| `update_feature` + `update_feature_content` | ✅ `feature update` / `feature docs update` | ✅ feature content edit (`feature update`) | 🟡 `update_feature` (content path only; no docs-first flow) |
| `update_feature_documentation` | ✅ `feature docs update` | ✅ status + content edits in Feature docs tab (`update_feature_documentation_cmd`) | ⚪ |
| `sync_feature_docs_after_session` | 🟡 session end side effect (`end_session`) | ✅ session end passes updated feature ids and refreshes docs context | 🟡 `end_session` side effect |
| `create_spec` | ✅ `spec create` | ✅ spec create + editor | ✅ `create_spec` |
| `move_spec` | ⚪ | ✅ spec status transition control in Spec detail (`move_spec_cmd`) | ⚪ |
| `update_spec` | ✅ `spec update` | ✅ spec editor update | ✅ `update_spec` |
| `create_adr` | ✅ `adr create` | ✅ ADR create flow | ✅ `create_adr` |
| `move_adr` | ✅ `adr move` | ✅ move between statuses (`move`) | ⚪ |
| `create_release` | ✅ `release create` | ✅ release create with structured init metadata (`status`/`supported`/`target_date`/`tags`) + editor | ✅ `create_release` |
| `update_release` / `update_release_content` | ✅ `release update` | ✅ release editor update (content + mapped status/support/date metadata persistence) | ✅ `update_release` |
| `create_note` / `update_note_content` | ✅ `note create`, `note update` | ✅ note editor | ✅ `create_note`, `update_note_content` |
| `create_workspace` | ✅ `workspace create` (TTY-guided prompts by default; script/AI-safe via `--no-input`; supports immediate session bootstrap via `--start-session --goal --provider --session-mode`) | ✅ workspace create flow with environment profile seed (with suggested IDs) + mode + feature/spec/release linking + dashboard relinking controls | ✅ `create_workspace_tool` |
| `activate_workspace` | ✅ `workspace switch` | ✅ select/activate workspace | ✅ `activate_workspace` |
| `transition_workspace_status` | ✅ `workspace archive` | ✅ archive action in workspace status card | ⚪ |
| `start_session` | ✅ `session start` | ✅ start session (UI + terminal auto-start) | ✅ `start_session` |
| `end_session` | ✅ `session end` | ✅ end session (auto doc sync on feature updates) | ✅ `end_session` |
| `log_progress` | ✅ `workspace session note` (active session required) | ⚪ | ✅ `log_progress` |
| `sync_workspace` (git hook flow) | ✅ `workspace sync` | ✅ workspace sync action | ⚪ (use `activate_workspace`/`sync_workspace` flows + resources) |
| `repair_workspace` | ✅ `workspace repair` | ✅ repair action in workspace panel | ✅ `repair_workspace` |
| `install_hooks` | ✅ `git install-hooks` | ⚪ | ⚪ |
| `resolve_agent_config` | ⚪ | ✅ agents settings panel action | ⚪ |
| `Modes` (mode CRUD/list/set) | ✅ `mode` commands | ✅ modes panel + session context | ✅ `list_modes`, `set_mode`, `get_mode` |
| `Skills` (create/update/delete/list scope-aware) | ✅ `skill` commands | ✅ skills module in settings | 🟡 resources only (`ship://skills`, `ship://skills/{id}`) |
| `Ghost` (`scan` / `report` / `promote`) | ✅ `ghost scan/report/promote` | ⚪ | ⚪ (CLI/skill surface only) |
| `time_start` / `time_stop` | ✅ `time start/stop` | ⚪ | ⚪ (CLI/skill surface only) |
| `append_event` / `list_events` | ✅ session lifecycle/progress now emits canonical session events + list/export/ingest | ✅ activity timeline + event list | 🟡 resources `ship://events`, `ship://events/{since}` |
| `init_project` | ✅ `init` now prompts for project name in TTY (defaults retained for non-interactive) | ✅ project onboarding/create flow (`create_project_with_options`) | ⚪ |
| `create_rule` / `update_rule` / `delete_rule` | ⚪ | ✅ agents/rules editor | ⚪ |
| `get_permissions` / `save_permissions` | ⚪ | ✅ permissions panel | ⚪ |
| `list_providers` | ✅ `providers list` | ✅ providers panel | ✅ `list_providers` |
| `enable_provider` / `disable_provider` | ✅ `providers connect/disconnect` | ✅ providers panel | ⚪ |
| `autodetect_providers` | ✅ `providers detect` | ✅ providers panel | ⚪ |
| `add_mcp_server` / `remove_mcp_server` / `list_mcp_servers` | ✅ `mcp add/remove/list` | ✅ MCP server panel | ✅ `add_mcp_server`, `remove_mcp_server`, `list_mcp_servers` |
| `export` / `import` MCP config | ✅ `mcp export/import` | ✅ export only (`export_agent_config`) | ⚪ |
| `get_vision` / `update_vision` | ⚪ | ✅ project vision page | ⚪ |
| `migrate_project_state` | ✅ startup + `ship dev migrate --force` | ⚪ | ⚪ |
| Workspace Terminal command center (UI command panel + PTY) | ⚪ | ✅ dedicated workflow control | ⚪ |

### Skill library updates (2026-03-09)

Added template skills under `core/runtime/src/templates/skills/`:

- `create-document`
- `workspace-session-lifecycle`
- `release-orchestration`
- `workspace-profile-onboarding`
- `start-session`

These are now available as structured capability workflows for agent-guided operations.
MCP tool/resource reduction and CLI parity hardening remain separate follow-up passes.

### Core hardening updates (2026-03-09)

- Session lifecycle and progress logging now use one event-first stream (`session.start`, `session.stop`, `session.note`) from runtime, removing competing surface-level session event writes.
- `events.ndjson` remains export-only; runtime event DB remains source-of-truth.
- Workspace dashboard now surfaces richer workspace capability context:
  - profile/environment id
  - provider set
  - git activity (touched files, LOC delta, ahead/behind, upstream)
  - recent session notes

### MCP surface simplification (2026-03-09)

Read-heavy MCP capabilities were consolidated into resources to reduce tool duplication:

- Removed redundant read tools: `get_feature`, `get_release`, `get_log`, `list_modes`,
  `list_workspaces`, `list_providers_tool`, `list_models_tool`
- Added/expanded resources:
  - `ship://log`
  - `ship://events`
  - `ship://workspaces`
  - `ship://modes`
  - `ship://providers`
  - template `ship://workspaces/{branch}`
  - template `ship://providers/{id}/models`
  - template `ship://events/{since}`
- Normalized existing resource template params from `{file}` to canonical identifiers (`{id}`).
- Removed non-core MCP tools in favor of CLI + skills:
  - `list_events`
  - `create_skill`
  - `update_skill`
  - `delete_skill`
  - `ghost_scan`
  - `ghost_promote`
  - `generate_adr`
  - `git_config_set`
  - `git_hooks_install`
  - `connect_provider`
  - `disconnect_provider`
  - `detect_providers`
  - `time_start`
  - `time_stop`

This keeps MCP tools focused on write/lifecycle actions and uses resources for read flows.

### Where the gaps are (decision-ready)

- **UI gaps to fill first:** explicit hooks install affordance and validation visibility for lifecycle gates.
- **MCP cleanup candidates:** MCP config import/export can move toward skill-backed UX in a later pass while keeping core protocol read/write parity intact.
- **MCP scope choice:** context sync is handled through workspace control-plane flows (`activate_workspace`/`sync_workspace`) rather than a separate git-sync tool.

### Backlog for the next pass (ordered)

1. **P0 parity gap:** Decide whether feature-documentation status editing needs to be promoted into a separate skill flow, while keeping command parity through `feature docs update`.
2. **P1 parity gap:** Add explicit UI affordance for `install_hooks` and expose hook sync health in workspace status surfaces.
3. **P1 parity gap:** Evaluate whether `log_progress` should remain a direct MCP tool or be orchestrated by `start-session` + lifecycle skills.
4. **P1 parity gap:** Decide if MCP should keep config import/export tools or replace with pre-packaged skills.
5. **P2 hardening:** Document the status of `move_adr` and `move_release` as explicit workflow design decisions (documented in section, currently not surfaced as direct MCP/CLI UI operations for release move).
6. **P2 hardening:** Add a small "no-op safe" test matrix in this doc for each section: `CLI`, `UI`, `MCP`, `Resources`, `Skill`.

## Feature Planning
<!-- this should be a skill, launch interactive session with questions to gather as much info about the feature, brainstorm, interactively select permissionset, MCP, Skills etc. Results in a workspace allocation and prompt to start-->
### create_feature
**What it does:** Creates a feature document with optional links to a release and spec, plus
a git branch association. Automatically scaffolds a feature documentation record.

**Process:**
1. Validates title is non-empty
2. Uses `version` as the release id
3. Writes feature row to `feature` table (metadata only — no body in DB)
4. Writes body content to markdown file at `.ship/project/features/planned/{slug}.md`
   — file contains `ship:feature id={id}` marker for later lookup
5. Scaffolds a `feature_doc` row with status `not-started` and template content
   (includes `## Capability Summary` header)
6. Appends `"feature create"` to action log

**Writes:** `feature` table row + `feature_doc` row + markdown file.

**Side effects:** Action log. Feature documentation is automatically scaffolded — not optional.

---

<!--This could be a lot more powerful, no focus on markdown needs to provision workspace or roll into a workspace comamnd instead. Would prefer more graceful error behavior-->
### feature_start
**What it does:** Transitions a feature from `planned` → `in-progress`.

**Process:**
1. Fetches current feature
2. Validates current status is exactly `Planned` — rejects any other status with
   `InvalidTransition` error
3. Updates `status` in `feature` table
4. Moves markdown file from `.../planned/` to `.../in-progress/`
5. Appends `"feature start"` to action log

**Validation:** Hard gate — can only start a `planned` feature. Calling on `in-progress`,
`implemented`, or `deprecated` returns an error.

---

### feature_done
**What it does:** Transitions a feature from `in-progress` → `implemented`.
<!-- cross reference actual status. -->
**Process:**
1. Fetches current feature
2. Validates current status is exactly `InProgress` — rejects otherwise
3. **Validates feature has a branch set** — empty/missing branch blocks completion
4. **Validates feature_doc status is not `not-started`** — documentation must have been
   started (any status other than `not-started`)
5. **Validates feature_doc content is non-empty** — can't ship with empty docs
6. Updates `status` in `feature` table
7. Moves markdown file to `.../implemented/`  <!-- Markdown should depeend on export settings-->
8. Appends `"feature done"` to action log <!-- appends or sends event. Append Events are gone-->

**Validation:** Three hard gates — must be `in-progress`, must have a branch, and
documentation must be started and non-empty. This is a meaningful quality gate.

---

### update_feature / update_feature_content
<!-- must be using  DB not markdown -->
**update_feature:** Replaces all metadata fields. Validates title non-empty. Does NOT move
the markdown file if status changes — use `feature_start`/`feature_done` for status transitions.

**update_feature_content:** Replaces only the body in the markdown file. Does not touch the
`feature` DB row (except `updated_at`).

---

### update_feature_documentation
**What it does:** Updates the feature's documentation content and/or status.

**Process:**
1. Reads current `feature_doc` row to get current revision
2. If content or status changed: increments revision counter
3. Upserts `feature_doc` row with new content, status, revision, `updated_at`
4. If changed: writes an immutable revision snapshot to `feature_doc_revision` table
   with actor field set to caller (default: `"ship"`)
5. `verify_now=true` sets `last_verified_at` timestamp

**Writes:** `feature_doc` upsert + optional `feature_doc_revision` insert (append-only audit trail).

---

### sync_feature_docs_after_session
**What it does:** Called at end-of-session to update docs for features worked on during
the session.

<!-- This is terrible, docs are a consumer facing description of how to use current capabilities. Appending session logs is for logs, commit messages, specs. Literally anywhere else is better than here. This should be a skill that prompts an agent to evaluate -->
Appends session summary text to each feature's doc content.
**Process:**
1. For each feature_id in the session's `updated_feature_ids`:
   - Calls `record_feature_session_update` which appends summary to doc content
   - Sets doc status to `draft` if it was `not-started`
   - Writes revision snapshot with actor `"session"`
2. Appends `"feature docs sync"` to action log with count

**Writes:** `feature_doc` upserts + `feature_doc_revision` inserts.

---

## Specifications

<!-- Guess What, another skill! Specs can link to all sorts of other docs, workspace is the most sensible relationship but these document a unit of work and ideally should write a successful commit message for us. Audit trail and available in archive but not something surfaced for reference all that often -->
### create_spec
**What it does:** Creates a spec document, optionally linked to a workspace.

**Process:**
1. Validates title non-empty
2. Generates nanoid(8) id
3. Writes row to `spec` table (metadata + status = `draft`)
4. Writes body to markdown file at `.ship/project/specs/draft/{slug}.md`
5. Appends `"spec create"` to action log

**Writes:** `spec` table row + markdown file.

---

### move_spec
**What it does:** Transitions spec status: `draft → active → archived`.

**Process:**
1. Updates `status` in `spec` table
2. Moves markdown file to new status subdirectory
3. Appends `"spec move"` to action log

**No transition validation** — any status → any status is accepted. No guards like feature_done.

---

### update_spec
**What it does:** Full replace of spec metadata and body.

**Process:**
1. Validates title non-empty
2. Updates `spec` table row
3. Updates markdown file body
4. Appends `"spec update"` to action log

---

## Architecture Decision Records

<!-- Guess What, another skill! ADRs have a lot of thought put into them and are all about documenting the decision making process. Agent should lead user through brainstorming, researching multiple choices and following the users lead on creating a decision matrix, weighing pros/cons etc. Then this is documented as context and a decision proposal...draft status at the create stage. There will be team dynamics eventually. The skill could prompt user to document pre-emptively -->
### create_adr
**What it does:** Records an architectural decision with context and decision narrative.

**Process:**
1. Generates nanoid(8) id, sets date to today
2. Writes row to `adr` table with status `proposed`
3. Writes markdown export to `.ship/project/adrs/proposed/{slug}.md`
4. Appends to action log

**Writes:** `adr` table row + markdown file. Markdown is git-committed (default policy).

**Note:** `adr_option` table exists in the schema for structured alternatives but is NOT
populated by any current code path. The Rust struct has no `options` field.

---

### move_adr
**What it does:** Changes ADR status: `proposed → accepted/rejected/superseded/deprecated`.

**Process:**
1. Updates `status` in `adr` table
2. Moves markdown file to new status subdirectory
3. Appends to action log

**No transition validation** — any → any accepted.

---

## Releases
<!-- Guess What, another skill! Agent helps user shape Release content and attaches features, may create many features along with it. -->
### create_release
**What it does:** Creates a versioned release document.

**Process:**
1. Validates version string is non-empty
2. Generates nanoid(8) id
3. Writes row to `release` table, mapping UI status (`planned`/`active`/`shipped`/`archived`) to runtime status (`upcoming`/`active`/`deprecated`)
4. Persists structured metadata (`supported`, `target_date`) into the `release` row
5. Persists `tags` in markdown metadata/frontmatter only
6. Extracts breaking changes from markdown body into `release_breaking_change`
7. Writes body to markdown file at `.ship/project/releases/` (exact subdir TBD — verify)
8. Appends `"release create"` to action log

**Writes:** `release` table row + `release_breaking_change` child rows (if any) + markdown file.

<!-- This needs to be fixed. frontmatter is deprecated, not source of truth for anything-->
**Note:** `tags` field is in `ReleaseMetadata` struct but **NOT in the DB schema** — tags for
releases live in markdown frontmatter only, not queryable from SQLite.

---

### update_release / update_release_content
**update_release:** Replaces full release struct including metadata and breaking_changes.
Deletes and re-inserts child rows in `release_breaking_change`.
When surfaced from UI, metadata updates now persist to DB (`version`, mapped `status`,
`supported`, `target_date`) while tags remain markdown metadata only.

**update_release_content:** Replaces only the markdown body. Does not touch breaking_changes
or DB metadata.

Both write action log entries.
**Note:** There is still no dedicated `move_release` op, but release status is now mutable via
`update_release` metadata updates surfaced in UI/CLI flows.

---

## Notes
<!-- works fine as a tool -->
### create_note / update_note_content
**What it does:** Creates or updates a freeform markdown note, project- or user-scoped.

**Process (create):**
1. Generates nanoid(8) id
2. Writes row to `note` table in **project DB** (scope=project) or **global DB** (scope=user)
3. No markdown file, no git artifact

**Writes:** `note` table only. Scope determines which database.

**Note:** User-scoped notes in global DB means they survive project deletion and are shared
across all projects on the machine.

---

## Workspace & Session Management

### create_workspace / activate_workspace
<--skill ;) -->
**What it does:** Creates a runtime workspace record that links a git branch to Ship entities
(feature, spec, or release) and tracks agent configuration.

**Process (create):**
1. Validates branch name is non-empty
2. Generates nanoid(8) id
3. Writes row to `workspace` table with status `active` by default (or specified status)
4. Persists optional linkage ids on workspace row (`feature_id`, `spec_id`, `release_id`, `environment_id`, `active_mode`)
5. If `feature_id` provided: writes to `branch_context` table (`branch → feature link_type + link_id`)
6. Sets `resolved_at` timestamp

**Process (activate):**
1. Sets workspace status to `Active`
2. Demotes all other workspaces for this project to `Idle`
   (only one workspace is active at a time)
3. Sets `last_activated_at`
4. Triggers workspace sync (regenerates agent context files)

**Writes:** `workspace` row + `branch_context` row + side effects from sync.

---

### start_session / end_session / log_progress
<!--very important to get the UI correct, and display lifecycle across all surfaces. not a common use case but maybe we can add human logs in UI-->
**What it does:** Tracks a discrete work session within a workspace — what was worked on,
by which provider, with what goal.

**Process (start):**
1. Generates session id
2. Writes row to `workspace_session` with status `active`, `started_at` = now
3. Records `mode_id`, `primary_provider`, and `goal`
4. Updates workspace `status` to `Active`

**Process (log_progress):**
1. Validates there is an active workspace session
2. Emits a canonical `Session.Note` event into the unified event stream

**Process (end):**
1. Sets session `status` to `ended`, `ended_at` = now
2. Records `summary`, `updated_feature_ids`, `updated_spec_ids`
3. Calls `sync_feature_docs_after_session` for any features listed in `updated_feature_ids`
   — this appends the session summary to each feature's documentation and bumps revision

**CLI UX note:** `ship workspace session ...` is the session command surface, including `workspace session note`.
4. Workspace remains in runtime `active` status unless explicitly archived

**Writes:** `workspace_session` row (create + update). `feature_doc` upserts via sync.

---

### sync_workspace (git hook flow)
<!-- git hooks are really just a fallback here. we need to tie this into provider hooks as well. I would love for this to be a very rarely used manual process and automated as often as possible-->
**What it does:** The core of Ship's agent context generation. Regenerates `CLAUDE.md`,
`.mcp.json`, and provider-specific config files for the current branch. Triggered
automatically on `git checkout` via the post-checkout hook.

**Process:**
1. `git checkout` fires → `.git/hooks/post-checkout` runs `ship git post-checkout`
2. Looks up current branch in `branch_context` table to find linked entity (feature/spec)
3. Resolves `AgentConfig` for the branch:
   - Loads project defaults from `ship.toml`
   - Applies active mode filters
   - Applies feature-level `[agent]` overrides (skills, mcp_servers)
4. For each enabled provider (claude, gemini, codex):
   - Writes `CLAUDE.md` / `GEMINI.md` / `AGENTS.md` at project root
   - Writes `.mcp.json` / provider-specific config
   - Writes `.claude/commands/*.md` (skill content inlined as slash commands)
   - Writes `SHIPWRIGHT.md` (agent layer summary: skills list, MCP servers, prompts)
5. Updates `mcp_managed_state` table with server IDs Ship wrote (for safe cleanup)
6. Updates `workspace.compiled_at` and `workspace.config_generation`

**Writes:** Multiple files at project root + `.claude/`, `.gemini/`, `.codex/` dirs.
All generated files are gitignored (pre-commit hook blocks staging them).

**Pre-commit hook:** Separately installed hook that blocks `CLAUDE.md`, `.mcp.json`,
`.claude/`, `.gemini/`, `.codex/`, `.agents/` from ever being committed.

**Manual trigger:** `ship git sync` reruns this for the current branch without a checkout.

---

## Git Configuration

### repair_workspace
<!-- should just run on it's own, maybe have a setting to opt out of automatic behavior-->
**What it does:** Detects and fixes drift between the workspace record, branch context, and
generated agent config files. Run manually or triggered when compile errors are detected.

**Process:**
1. Reads current workspace record from SQLite
2. Checks `branch_context` table for linked entity — verifies link is still valid
3. Checks whether `compiled_at` is stale relative to `config_generation` counter
4. If drift detected: re-runs `sync_workspace` to regenerate config files
5. Clears `compile_error` field on success
6. Returns a `WorkspaceRepairReport` describing what was fixed

**Writes:** May update `workspace` row + regenerate provider config files.

---

### install_hooks
<!-- this is only triggered on init right? could be another setting for users to decide if they want to opt out (default opt-in) could even be part of init sequence -->
**What it does:** Installs two git hooks into `.git/hooks/`:

1. **post-checkout** — `ship git post-checkout "$@"` — triggers agent context regeneration
   on every branch switch
2. **pre-commit** — blocks staging of `CLAUDE.md`, `.mcp.json`, `.claude/`, `.gemini/`,
   `.codex/`, `.agents/` — prevents generated files from being committed

**Process:** Writes hook files, sets `chmod 755`. Idempotent — skips if content unchanged.

Also writes Ship's gitignore entries to `.gitignore` at project root (idempotent).

---

## Agent Configuration

### resolve_agent_config
<!--would love to get some performance monitoring done here. can an agent launch via CLI/MCP?-->
**What it does:** Computes the effective agent configuration for the current branch/workspace.
This is what gets written to `CLAUDE.md` and `.mcp.json`.

**Resolution order:**
1. Project-level defaults from `ship.toml` (providers, model, max_cost, max_turns)
2. Active mode filter from `agent_mode` table (filters which MCP servers and skills are active)
3. Feature-level `[agent]` overrides from `feature.agent_json` in DB
   (can add specific skills or MCP servers for this branch only)

For workspace session provider export specifically, precedence is:
`workspace.providers` → `feature.agent.providers` → `mode.target_agents` →
`config.providers` → `["claude"]`.
Use `ship workspace providers` to inspect effective resolution.

**Result:** `AgentConfig` struct containing resolved providers, model, max_cost, max_turns,
mcp_servers, skills, rules, permissions, active_mode. Written to disk as provider-specific files.

---

### Modes 
<!-- mode should also affect permissions -->
**What it does:** A mode is a named filter that restricts the agent's active tools. When a
mode is active, only the MCP servers and skills in `active_tools_json`, `mcp_refs_json`, and
`skill_refs_json` are included in the resolved agent config.

**Storage:** `agent_mode` table in project SQLite. Also readable from `.ship/agents/modes/`
TOML files.

**Effect on workspace:** Each workspace can override the project-level active mode. Mode
resolution: workspace mode override → project active mode → no filter (all tools).

---

### Skills
**What it does:** Skills are reusable AI instruction files injected into the agent's context.
They appear as slash commands in Claude (`.claude/commands/{id}.md`) and as inline context
in `CLAUDE.md`.

**Storage:** `.ship/agents/skills/{id}/SKILL.md` (project-scoped) or `~/.ship/skills/{id}/SKILL.md`
(user-scoped). No DB entry — filesystem only.

**Install from git:** `skill install` fetches a `.md` file from a git URL and writes it to
the skills directory. No registry or version locking — file is copied verbatim.

**Catalog:** 6 community skills + 10 official MCP servers embedded in the binary. Browsable
via `list_catalog` / `search_catalog`. Installing from catalog = writing the embedded content
to the skills directory.

---
<!-- should hide in next release-->
### Ghost Issues (TODO/FIXME scanner)
**What it does:** Scans the project codebase for `TODO`, `FIXME`, `HACK`, `BUG` comments
and surfaces them as potential work items.

**Process (scan):**
1. Walks project root recursively, reads source files
2. Finds comment lines matching the patterns
3. Returns list of `GhostIssue` records: file path, line number, kind, text

**Process (promote):**
1. Takes a file path + line number
2. Creates a real issue with the comment text as the title
3. Links the ghost issue location to the issue record

**Storage:** Scan results are in-memory only (not persisted). Promoted ghost issues become
normal `issue` rows.

**Note:** CLI commands are hidden from `--help` but fully functional. MCP tools exposed.

---
<!-- should hide in next release-->
## Time Tracking

### time_start / time_stop
**What it does:** Tracks wall-clock time spent on an issue.

**Process (start):**
1. Creates a timer entry linked to an issue ID
2. Records `started_at` in a time tracking table (plugin-managed)
3. Marks the timer as active

**Process (stop):**
1. Finds the active timer
2. Records `ended_at`, computes duration
3. Marks timer as complete

**Storage:** Plugin-managed — likely in project SQLite via the `time-tracker` plugin crate.

**Note:** All CLI commands hidden from `--help`. MCP has `time_start`/`time_stop` only —
no status, list, or report tools. UI has no time tracking surface.

---

## Event Log
<!-- Sqlite first, need to audit this is being called everywhere it needs to fire. Also important is identifying the actor who fired the event. Are we concerned about space? Maybe we export every so often?  How does this need to adapt for cloud migration? UNDO would be incredible to see -->

### append_event / list_events
**What it does:** The event log is an append-only stream of all meaningful state changes
in the project. Powers the activity feed and future sync/replication.

**Process:** Every CRUD operation calls `append_event` after success. Events have:
- `seq` — monotonically increasing sequence number
- `actor` — who triggered it (user, agent, session)
- `entity` — entity type (feature, issue, spec, etc.)
- `action` — what happened (created, updated, status-changed, etc.)
- `entity_id` — nanoid of affected entity
- `details_json` — freeform context

**Storage:** `events` table in project SQLite. NDJSON export is on-demand via `event export`.

**Note:** Action log (`log_action`) is separate from the event log — the action log is a
human-readable summary exposed via resources/UI, while events are machine-readable with
`seq` for cursor-based reads.

---

## Project Initialization

### init_project
**What it does:** Creates the `.ship/` directory structure, writes `ship.toml`, creates
the project SQLite database, applies all schema migrations, and registers the project
in the global registry.

**Directory structure created:**
<!--   workflow is deprecated. Spec archive TBD ...prompts are deprecated

-->
```
.ship/
  ship.toml              — project metadata, git policy, providers, active mode
  db.sqlite              — project database (all schema migrations applied)
  project/
    adrs/proposed/       — ADR markdown files by status
    releases/            — release markdown files
    features/planned/    — feature markdown files by status
    specs/draft/         — spec markdown files by status
    notes/               — (unused — notes are DB-only)
    vision.md            — project vision (freeform, no schema)
  agents/
    skills/              — project-scoped skill files
    modes/               — mode config files
    prompts/             — prompt files
    rules/               — always-active rule files
    mcp.toml             — MCP server registry
    permissions.toml     — Ship permissions
```

<!-- this is just false...databases are at ~/.ship/state/$project/db and ~/.ship/ship.db scrub references to wrong files -->
**Writes:** Directory tree + `ship.toml` + `db.sqlite` (with all migrations). Registers
project path in global DB at `~/.config/ship/db.sqlite`.

<!-- should be automatic unless user opts out-->
**Side effects:** Writes gitignore entries for generated agent files. Does NOT install git
hooks — that requires a separate `ship git install-hooks` call.


---

## Rules

### create_rule / update_rule / delete_rule
**What it does:** Manages always-active instruction files that are injected into every agent
context for this project, regardless of mode.

**Process (create/update):**
1. Validates file name: must be a single component (no path separators), must end in `.md`
   or have no extension (auto-appended), must not be `README.md`
2. Writes to `.ship/agents/rules/{file_name}` using atomic write (write to tmp, rename)
3. No DB entry — filesystem only

**Process (delete):**
1. Same filename validation
2. `fs::remove_file` — no soft delete, gone immediately

**Effect on agent context:** Rules are collected by `list_rules` and included in
`AgentConfig.rules`. All rules in the directory are active — there is no enable/disable
mechanism. Presence in the directory = active.

**Storage:** `.ship/agents/rules/*.md` — plain markdown files, no frontmatter.
Git-committed by default (part of `agents/` category).


<!-- is there consistent error handling across various domains? ie we aren't just throwing some random strings at users? Nice to include things like resolutions when possible-->

**Validation rejections:** `""`, `"README.md"`, `"../escape.md"`, `"nested/rule.md"`,
`"bad.txt"` (non-`.md` extension) all return errors.

---

## Permissions

### get_permissions / save_permissions
<!-- this needs to be well documented and tested. Would LOVE to have inference/autocomplete for tools and commands to make this easier. Max cost and turns is really silly bc users are almost all on a plan with another provider. Worth a discussion but still.

I want to expand this to be compatible with sandboxes if provider allows and we are also going to start hooking into providers so we can do things like add our own pre-post tool call logic and improve the experience with too many approval requests (without going to yolo mode like everyone else)
-->
**What it does:** Reads and writes the Ship permission model for the project. Controls
what the agent is allowed to do — tool access, filesystem paths, shell commands, network.

**Structure:**
```
Permissions {
  tools: { allow: ["*"], deny: [] }         -- tool name globs
  filesystem: { allow: [], deny: [] }        -- path globs
  commands: { allow: [], deny: [] }          -- shell command patterns
  network: {
    policy: none|localhost|allow-list|unrestricted
    allow_hosts: []
  }
  agent: {
    max_cost_per_session: Option<f64>
    max_turns: Option<u32>
    require_confirmation: []                 -- action patterns requiring user confirmation
  }
}
```

**Process:** Read: parses `.ship/agents/permissions.toml` as TOML. Returns default
(all tools allowed, no network) if file doesn't exist.
Write: serializes struct to TOML, atomic write to `.ship/agents/permissions.toml`.

**Effect on agent context:** Permissions are included in `AgentConfig.permissions` and
written into provider config files during `sync_workspace`. Modes can overlay
`tools.allow/deny` via `ModeConfig.permissions` — mode permissions merge on top of base.

**Storage:** `.ship/agents/permissions.toml` — git-committed (part of `agents/` category).

---

## Provider Management
<!-- we need to manage this very tightly in terms of cleaning up spawned processes that are not longer in use. MAybe something to add to the base ship skill -->
### list_providers / enable_provider / disable_provider / autodetect_providers
**What it does:** Manages which AI providers (Claude Code, Gemini, Codex) are connected to
the project and have their config files generated.

<!-- adding new providers should be a clear process. rather than adding as many as we can we care about supporting many models. WE will support providers/agents that are popular, secure, extensible. Really want to see hooks in their API. Codex is an exception because of quality and popularity...hope they add hooks soon. Others to add next...cursor, opencode. -->
**Providers:** Three supported: `claude`, `gemini`, `codex`.
Ship does not call AI APIs directly — it spawns the provider's CLI binary.

<!-- a test would be nice here too to make sure it works. maybe hit the models endpoint or something? also make sure this is disposed if we spawn an ephemeral process -->
**Process (autodetect):**
1. Checks PATH for known binaries: `claude`, `gemini`, `codex`
2. For each found: calls `detect_version(binary)` to get version string
3. Sets `installed: true` on the provider descriptor
4. Calls `enable_provider` for each detected binary

**Process (enable/disable):**
1. Reads `providers` list from `ship.toml`
2. Adds or removes the provider ID from the list
3. Writes updated `ship.toml`
4. Triggers `sync_workspace` to regenerate config files for newly enabled providers

<!-- I am not sure this is a great TOML configuration. Git commit is good, but how much are we allowing users to configure?  -->
**Storage:** `providers` array in `ship.toml`. Per-provider config written at sync time.

**Provider descriptor:** Each provider has:
- `id`: `"claude"` | `"gemini"` | `"codex"`
- `installed: bool` — is binary in PATH?
- `version: Option<String>` — binary version if installed
- `models: Vec<ModelInfo>` — static list of known models (embedded in binary)

<!-- this is a terrible idea, and already 8 months out of date for gemini and codex -->
**Model lists (static, embedded):**
- Claude: `claude-sonnet-4-6` (recommended), `claude-opus-4-6`, `claude-haiku-4-5`
- Gemini: `gemini-2.5-pro` (recommended)
- Codex: `gpt-4o` (recommended)

---

## MCP Server Management
<!-- mode toggles this correct? Is ENV safe to git commit?-->
### add_mcp_server / remove_mcp_server / list_mcp_servers
**What it does:** Manages the registry of MCP servers available to agents. The registry is
stored in `.ship/agents/mcp.toml` and is the source of truth for what gets written to
`.mcp.json` at sync time.

**`McpServerConfig` struct:**
```
id: String          -- nanoid(8), generated on add
name: String        -- display name
command: String     -- binary to run (stdio) or URL (http/sse)
args: Vec<String>
env: HashMap<String, String>
scope: String       -- "global" | "project" | "mode"
transport: McpServerType  -- stdio (default) | sse | http
```

**Process (add):**
1. Generates nanoid(8) id
2. Appends entry to `.ship/agents/mcp.toml`
3. Does NOT immediately sync — caller must run `sync_workspace` or `git sync` to apply

**Process (remove):**
1. Reads `mcp_managed_state` table to find Ship-managed server IDs (written by sync)
2. Removes from `mcp.toml`
3. On next sync: `managed_state` entries are cleaned from `.mcp.json`

**Transport variants:**
- `stdio` — runs a local binary: `{ command, args, env }`
- `sse` — connects to HTTP server via Server-Sent Events: `{ command: url }`
- `http` — connects to HTTP endpoint: `{ command: url }`

**Storage:** `.ship/agents/mcp.toml` — git-committed (part of `agents/` category).

### export / import MCP config
**export:** Reads `mcp.toml` and writes `.mcp.json` (or provider-specific equivalent) in
the format expected by the target AI client. Merges with existing client config, preserving
non-Ship entries. Records Ship's server IDs in `mcp_managed_state` table.

**import:** Reads the AI client's current config (e.g. Claude's `claude_desktop_config.json`),
extracts MCP server entries, and writes them to `.ship/agents/mcp.toml`. Useful for
migrating existing client configs into Ship management.

---

## Agent Export (sync_workspace detail)

### What gets written per provider

| Provider | Context file | MCP config | Slash commands |
|---|---|---|---|
| `claude` | `CLAUDE.md` at root | `.mcp.json` at root | `.claude/commands/*.md` |
| `gemini` | `GEMINI.md` at root | `.gemini/mcp.json` | (none yet) |
| `codex` | `AGENTS.md` at root | `.codex/mcp.json` | (none yet) |

Also written for all enabled providers:
<!-- not sure how valuable this is compared to CLAUDE/AGENTS, consider deprecating, but if we keep it it must not be git committed -->
- `SHIPWRIGHT.md` — agent layer summary (active skills, mode, MCP servers, session context)

### CLAUDE.md content structure
1. Project name + feature title (if on feature branch)
2. Feature description and body
3. Open issues linked to this feature
4. Active skills — content inlined
5. Active rules — content inlined
6. Session goal (if active session)
7. Links to spec

### Hooks
<!-- yes! we are going to expand this big time, every provider with hooks we are going to own. we can audit tool calls, restructure complex commands, apply our security filter here, and even disable tools if they are getting in the way ie memory etc -->
**What it is:** Shell commands that fire on agent lifecycle events.
Defined per-mode in `ModeConfig.hooks` or globally in `ship.toml`.

**Trigger types:**
- `PreToolUse` — before a tool call
- `PostToolUse` — after a tool call
- `Notification` — on agent notification
- `Stop` — when agent stops
- `SubagentStop` — when a subagent stops
- `PreCompact` — before context compaction

**`matcher`** — optional glob/regex for tool name (e.g. `"Bash"`, `"mcp__*"`). Empty = all tools.

Hooks are written into the provider's config format at sync time. For Claude: into
`.claude/settings.json` `hooks` array.

---

## Project Config (`ship.toml`)
<!-- needs good documentation. not the best place to configure colors...that can just live in the ui. I mean we really need to justify how much of this is not just UI, this exists for CLI/MCP first users. The last thing I want is a provider not connecting because someone did not open up a config file nested 3 levels deep in their project -->
**What it is:** The project's central config file. Written at init, updated by various
commands. Source of truth for git policy, providers, active mode, statuses, and
agent layer settings.

**Key sections:**
```toml
[project]
name = "my-project"
id = "nanoid"

[git]
commit = ["agents", "ship.toml", "templates", "releases", "features", "specs", "adrs"]
ignore = ["issues", "notes"]

[ai]
provider = "claude"
model = "claude-sonnet-4-6"

[[statuses]]
id = "backlog"
name = "Backlog"
color = "gray"

[agents]
skills = []
prompts = []
context = []

providers = ["claude"]
active_mode = "focus"   # optional
```

**Config file lookup:** Ship checks for `ship.toml`, then legacy `config.toml`
(alias), then `config.toml` (legacy). First found wins.

**`get_effective_config(project_dir, mode_id)`** — returns config with mode-level tool
permission overrides applied on top of base config. Used by agent export.

---

## Vision

### get_vision / update_vision
**What it does:** Reads and writes the project's freeform vision document. No structured
fields — pure markdown narrative.

**Storage:** `.ship/project/vision.md` — a single file, no frontmatter, no schema.
Git-committed (part of `project/` category).

**Process:** Read: `fs::read_to_string`. Write: atomic write via `write_atomic`.
No validation, no log entry. Completely freeform.

**Current surface:** UI only (`get_vision_cmd`, `update_vision_cmd`). No CLI or MCP access.

---

## Migration

### migrate_project_state
**What it does:** Scans the project for legacy data formats and imports them into the
current SQLite schema. Runs automatically on startup via `ensure_imported()` and can be
forced via `ship dev migrate --force`.

**What it migrates:**
<!-- this needs an audit!  YAML issues migration occured already on the one machine in the world with this software, now it's just an orphaned snapshot of something does not exist anymore. Same with frontmatter, old json etc. REMOVE THE CRUFT -->
- YAML issues → TOML issues → SQLite `issue` rows
- Old JSON `config.json` → `ship.toml`
- Markdown ADRs with TOML frontmatter → `adr` table rows
- Feature `.md` files with `id = "..."` frontmatter → `feature` table rows
- Spec `.md` files → `spec` table rows
- Release `.md` files → `release` table rows
- Old `events.ndjson` → SQLite `event_log` table

**Process:** Each entity type has its own `import_*_from_files` function. Migration is
tracked in `migration_audit` table — prevents double-importing.

<!-- good god no, this is a dev tool at the moment. eventually very useful as a runtime migration service. keep that scaffolded. WE should also be able to run this like once after an update and not on every fucking CLI call holy shit -->
**Known issue:** `ensure_imported()` currently runs on every CLI command invocation,
not just on first run. Performance impact on large projects.

---

## Workspace Terminal (UI Command Center)
<!-- yeah this is awesome, we need really good tests and session management. clean up of processes. Performance is a real concern here. In the future we could intercept PTY and use a cutom designed terminal with nicer UI for generated docs and MCP calls. -->
**What it does:** The UI's workspace command center spawns a PTY (pseudo-terminal) session
running a configured provider binary (e.g. `claude`, `gemini`) inside the project root.
This is the primary way the UI launches and interacts with AI agents.

**PTY session struct:**
```
PtySession {
  id: String          -- session identifier
  branch: String      -- workspace branch
  provider: String    -- binary being run (e.g. "claude")
  cwd: String         -- working directory
  cols/rows: u16      -- terminal dimensions
  master/writer/child -- PTY handles
  output_rx           -- channel for reading output bytes
  closed: bool
  exit_code: Option<u32>
}
```

**Tauri commands:**
- `list_workspace_editors_cmd` — lists available editors/IDEs
- `read_workspace_terminal_cmd(terminal_id, lines)` — reads buffered output bytes
- `write_workspace_terminal_cmd(terminal_id, input)` — sends keystrokes/input
- `resize_workspace_terminal_cmd(terminal_id, cols, rows)` — resizes PTY
- `stop_workspace_terminal_cmd(terminal_id)` — kills the process

**Process (start terminal):**
1. Resolves provider binary path from `AiConfig.effective_cli()`
2. Creates PTY via `portable_pty`
3. Spawns provider binary in PTY with `cwd` = project root
4. Spawns reader thread that buffers output into `mpsc` channel
5. Returns session ID to UI for subsequent read/write calls

**Performance tracking:** `RuntimePerfCounters` tracks every terminal operation
(start, read, write, resize, stop) with call counts, error counts, and last operation
duration in microseconds. Exposed via `get_runtime_perf_cmd`.

**Note:** Terminal sessions are in-memory only — not persisted to SQLite, not linked to
`WorkspaceSession` records. The two session concepts are separate:
`WorkspaceSession` = Ship's tracking record; PTY session = the actual live process.

---

## Capability Audit Checkpoint (2026-03-09)

### MCP surface alignment
- Removed redundant MCP read tools:
  - `get_project_info`
  - `get_workspace`
  - `get_workspace_provider_matrix`
  - `get_session_status`
  - `list_sessions`
- Added/expanded read resources:
  - `ship://project_info`
  - `ship://sessions`
  - `ship://sessions/{workspace}`
  - `ship://workspaces/{branch}/provider-matrix`
  - `ship://workspaces/{branch}/session`
- Net effect: MCP is now resource-first for read/query flows; tools focus on creation, mutation, and lifecycle control.

### Runtime state/migration hardening
- `workspace.environment_id` is now explicitly ensured in compatibility backfills (`ensure_project_schema_compat`), preventing partial-legacy DBs from missing environment linkage.
- Workspace deletion now cleans up dependent runtime projection rows:
  - `runtime_process`
  - `git_workspace`
  - (existing) `workspace_session`
  - (existing) clears `spec.workspace_id`
- Event-log schema now includes a composite lookup index:
  - `event_log(timestamp, actor, entity, action, subject)`
  - Applied both in migration DDL and compat/index backfill path.

### CLI startup/performance alignment
- Auto-import file checks are now skipped for non-project workflow commands:
  - `ship ui`, `ship projects ...`, `ship providers ...`, `ship mcp ...`, `ship version`, `ship doctor`, and no-command/default invocation.
- Net effect: fewer unnecessary SQLite/import checks on command paths that do not mutate/read project entities.

### Event-log ingestion behavior
- `ensure_event_log` no longer imports/migrates NDJSON files at runtime.
- Event stream source of truth is SQLite `event_log` only.
