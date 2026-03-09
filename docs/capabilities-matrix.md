# Ship Capabilities Matrix

> Pre-launch audit of runtime capabilities vs. surface coverage across CLI, MCP, and UI.
> Generated: 2026-03-08
>
> **Confidence note:** CLI and MCP were read from source and are high-confidence. UI section
> is based on Tauri command signatures — actual UI screens may cover more than listed here.
> Rows marked ⚠️ UI should be verified against the running app.

Legend: ✅ Full | ⚠️ Partial | ❌ Missing | 🔴 Friction

---

## 1. Project Initialization & Management

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Init project | ✅ `init` | ❌ | ✅ `create_project_with_options` | MCP has no init tool |
| List tracked projects | ✅ `projects list` | ❌ | ✅ `list_projects` | MCP gap |
| Track / Untrack project | ✅ `projects track/untrack` | ❌ | ✅ `pick_and_open_project` | MCP gap |
| Rename project | ✅ `projects rename` | ❌ | ✅ `rename_project_cmd` | MCP gap |
| Open / set active project | N/A | ✅ `open_project` | ✅ `set_active_project` | CLI doesn't need this |
| Full project context snapshot | N/A | ✅ `get_project_info` | ✅ (derived) | — |

---

## 2. Issue Tracking

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Create issue | ✅ `issue create` | ✅ `create_issue` | ✅ | — |
| List issues | ✅ `issue list` | ⚠️ via `get_project_info` | ✅ `list_items` | MCP no dedicated list tool |
| Update issue | ❌ | ✅ `update_issue` | ✅ | 🔴 CLI can't edit issue metadata |
| Move status | ✅ `issue move` | ✅ `move_issue` | ✅ | ✅ |
| Delete issue | ❌ | ✅ `delete_issue` | ✅ | 🔴 CLI missing delete |
| Search issues | ❌ | ✅ `search_issues` | ❌ ⚠️ UI? | No text search in CLI |
| Issue priority | ❌ | ❌ | ❌ ⚠️ UI? | Field in model, no confirmed surface |
| Link to spec/feature | ❌ | ❌ | ❌ ⚠️ UI? | `spec_id`/`feature_id` fields exist in model |
| AI-generate description | ❌ | ✅ `generate_issue_description` | ❌ ⚠️ UI? | MCP-only AI assist |
| Brainstorm issues | ❌ | ✅ `brainstorm_issues` | ❌ ⚠️ UI? | MCP-only |

**Core flow (create → move):** 2 commands. ✅

**Gaps:** No CLI update/delete. Priority and cross-entity links have no confirmed surface.

---

## 3. Feature Planning

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Create feature | ✅ `feature create` | ✅ `create_feature` | ✅ | — |
| List features | ✅ `feature list` | ⚠️ via `get_project_info` | ✅ | MCP no dedicated list tool |
| Get feature | ✅ `feature get` | ✅ `get_feature` | ✅ | — |
| Update feature content | ✅ `feature update` | ✅ `update_feature` | ✅ | — |
| Start (→ in-progress) | ✅ `feature start` | ❌ | ❌ ⚠️ UI? | 🔴 Source shows no Tauri `feature_start` cmd |
| Done (→ implemented) | ✅ `feature done` | ❌ | ❌ ⚠️ UI? | 🔴 Same — not found in Tauri commands |
| Delete feature | ✅ `feature delete` | ❌ | ❌ ⚠️ UI? | Not in Tauri commands found |
| Link to release (at create) | ✅ `--release-id` | ✅ | ✅ | — |
| Link to spec (at create) | ✅ `--spec-id` | ✅ | ✅ | — |
| Update release/spec link | ❌ | ❌ | ❌ ⚠️ UI? | 🔴 No update path post-creation |
| Feature documentation | ✅ `feature docs *` (4 subcmds) | ❌ | ❌ ⚠️ UI? | 🔴 Not found in MCP or Tauri |
| Filter by status | ✅ `feature list --status` | ❌ | ⚠️ UI? | — |

**Core flow:** Create → link → start → done = 3–4 commands. Acceptable.

**Gaps:** `feature start/done` CLI-only per source. Feature docs missing from MCP. No post-creation link updates.

---

## 4. Specifications

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Create spec | ✅ `spec create` | ✅ `create_spec` | ✅ | — |
| List specs | ✅ `spec list` | ⚠️ via `get_project_info` | ✅ | MCP no dedicated list tool |
| Get spec | ✅ `spec get` | ❌ | ✅ | 🔴 MCP missing `get_spec` |
| Update spec | ❌ | ✅ `update_spec` | ✅ | 🔴 `ship spec update` doesn't exist (known issue) |
| Move spec status | ❌ | ❌ | ❌ ⚠️ UI? | 🔴 `move_spec` in runtime — zero confirmed surface |
| Delete spec | ❌ | ❌ | ✅ | — |
| Lifecycle (start/done) | ❌ | ❌ | ❌ | Known open issue |

**🔴 Critical:** `spec update` missing from CLI (known). `move_spec` runtime fn with no confirmed surface anywhere. Spec lifecycle is incomplete end-to-end.

---

## 5. Architecture Decision Records

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Create ADR | ✅ `adr create` | ✅ `log_decision` | ✅ | 🔴 MCP tool named `log_decision` — confusing |
| List ADRs | ✅ `adr list` | ⚠️ via `get_project_info` | ✅ | MCP no dedicated list tool |
| Get ADR | ✅ `adr get` | ✅ `get_adr` | ✅ | — |
| Update ADR | ❌ | ❌ | ✅ | 🔴 CLI/MCP missing |
| Move ADR status | ✅ `adr move` | ❌ | ✅ | 🔴 MCP missing |
| Delete ADR | ❌ | ❌ | ✅ | 🔴 CLI/MCP missing |
| AI-generate ADR | ❌ | ✅ `generate_adr` | ❌ ⚠️ UI? | MCP-only |

**🔴 Naming:** `log_decision` for ADR creation is non-obvious. Rename to `create_adr`.

---

## 6. Releases

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Create release | ✅ `release create` | ✅ `create_release` | ✅ | — |
| List releases | ✅ `release list` | ⚠️ via `get_project_info` | ✅ | MCP no dedicated list tool |
| Get release | ✅ `release get` | ✅ `get_release` | ✅ | — |
| Update release | ✅ `release update` | ✅ `update_release` | ✅ | Full replace only — no append/merge |
| Move release status | ❌ | ❌ | ❌ ⚠️ UI? | 🔴 `move_release` in runtime — zero confirmed surface |
| Link features to release | ❌ | ❌ | ❌ ⚠️ UI? | 🔴 `feature_ids` on release model, no surface |

**🔴 Release status lifecycle** (planned → active → shipped → archived) has no confirmed surface anywhere.

---

## 7. Notes

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Create note | ✅ `note create` | ✅ `create_note` | ✅ | — |
| List notes | ✅ `note list` | ❌ | ✅ | 🔴 MCP can create but not list/read |
| Get note | ✅ `note get` | ❌ | ✅ | 🔴 MCP gap |
| Update note | ✅ `note update` | ✅ `update_note` | ✅ | — |
| Delete note | ❌ | ❌ | ✅ | — |
| User-scoped notes | ✅ `--scope user` | ✅ | ✅ | — |

---

## 8. Workspace & Session Management

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Create workspace | ✅ `workspace create` | ✅ | ⚠️ UI? | Terminal-based UI per user |
| List workspaces | ✅ `workspace list` | ✅ | ⚠️ UI? | — |
| Activate workspace | ✅ `workspace switch` | ✅ | ⚠️ UI? | — |
| Sync workspace | ✅ `workspace sync` | ✅ | ⚠️ UI? | — |
| Repair workspace | ✅ `workspace repair` | ✅ | ⚠️ UI? | — |
| Provider matrix | ❌ | ✅ | ⚠️ UI? | — |
| Start session | ✅ `session start` | ✅ | ⚠️ UI terminal? | — |
| End session | ✅ `session end` | ✅ | ⚠️ UI terminal? | — |
| Session status | ✅ `session status` | ✅ | ⚠️ UI terminal? | — |
| Log progress | ✅ `log` | ✅ `log_progress` | ⚠️ UI? | — |
| List sessions | ✅ `session list` | ✅ | ⚠️ UI? | — |
| Open workspace in IDE | ✅ `workspace open` | ❌ | ⚠️ UI terminal? | — |
| Archive workspace | ✅ `workspace archive` | ❌ | ⚠️ UI? | — |
| Spawn shell / run provider | ❌ | ❌ | ✅ terminal cmds | UI-exclusive (command center) |

> **Note:** UI has workspace terminal commands (`read/write/resize/stop_workspace_terminal_cmd`,
> `list_workspace_editors_cmd`). The workspace command center is UI-native. The ⚠️ rows above
> need verification against the actual UI screens.

**Duplicate surface:** `session *` = `workspace session *`. 🔴 Cognitive overhead for users.

---

## 9. Git Integration

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Show git policy status | ✅ `git status` | ❌ | ❌ | — |
| Include/exclude category | ✅ `git include/exclude` | ✅ `git_config_set` | ✅ (via project config) | — |
| Install git hooks | ✅ `git install-hooks` | ✅ `git_hooks_install` | ❌ ⚠️ UI? | — |
| Manual sync (CLAUDE.md etc.) | ✅ `git sync` | ✅ `git_feature_sync` | ❌ ⚠️ UI? | — |

---

## 10. Agent Configuration

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| List providers | ✅ `providers list` | ✅ | ✅ | — |
| Connect provider | ✅ `providers connect` | ✅ | ❌ ⚠️ UI? | — |
| Disconnect provider | ✅ `providers disconnect` | ✅ | ❌ ⚠️ UI? | — |
| Detect providers | ✅ `providers detect` | ✅ | ❌ ⚠️ UI? | 🔴 detect + connect = 2 steps, should be 1 |
| List models | ✅ `providers models` | ✅ | ✅ | — |
| Get resolved agent config | ❌ | ❌ | ✅ `get_agent_config_cmd` | 🔴 Agents (Claude, Codex) can't inspect resolved config |
| MCP: list servers | ✅ `mcp list` | ❌ | ✅ | 🔴 MCP server can't list its own registry |
| MCP: add server | ✅ `mcp add` / `add-stdio` | ❌ | ✅ | 🔴 `add` vs `add-stdio` — transport should be inferred |
| MCP: remove server | ✅ `mcp remove` | ❌ | ✅ | — |
| MCP: export / import config | ✅ | ❌ | ❌ | CLI-only |
| Skill: list | ✅ `skill list` | ❌ | ✅ | 🔴 MCP can't list skills |
| Skill: create/update/delete | ✅ | ✅ | ✅ | — |
| Skill: install from git | ✅ `skill install` | ❌ | ❌ | CLI-only |
| Catalog: browse/search | ❌ | ✅ | ✅ | 🔴 CLI missing catalog browse |
| Mode: list/set/clear | ✅ `mode *` | ✅ | ✅ | — |
| Rules: CRUD | ❌ | ❌ | ✅ | 🔴 No CLI or MCP access to rules |
| Permissions: get/save | ❌ | ❌ | ✅ | 🔴 No CLI or MCP access to permissions |
| Vision: get/update | ❌ | ❌ | ✅ | 🔴 Project narrative only editable in UI |

---

## 11. Ghost Issues

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Scan for TODO/FIXME | ✅ `ghost scan` *(hidden)* | ✅ `ghost_scan` | ❌ | 🔴 Hidden CLI command |
| Promote to issue | ✅ `ghost promote` *(hidden)* | ✅ `ghost_promote` | ❌ | 🔴 Hidden CLI command |
| View last scan report | ✅ `ghost report` *(hidden)* | ❌ | ❌ | — |

---

## 12. Time Tracking

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Start timer | ✅ `time start` *(hidden)* | ✅ `time_start` | ❌ | 🔴 Hidden CLI, UI missing |
| Stop timer | ✅ `time stop` *(hidden)* | ✅ `time_stop` | ❌ | 🔴 Hidden CLI, UI missing |
| Timer status | ✅ `time status` *(hidden)* | ❌ | ❌ | — |
| Log manual time | ✅ `time log` *(hidden)* | ❌ | ❌ | — |
| List time entries | ✅ `time list` *(hidden)* | ❌ | ❌ | — |
| Time report | ✅ `time report` *(hidden)* | ❌ | ❌ | — |

**🔴 Time tracking is effectively invisible** — hidden CLI + partial MCP + zero UI.

---

## 13. Events & Logging

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| List events | ✅ `event list` | ✅ `list_events` | ✅ | — |
| Ingest events | ✅ `event ingest` | ❌ | ✅ | MCP gap |
| Export events | ✅ `event export` | ❌ | ❌ | CLI-only |
| Get action log | ❌ | ✅ `get_log` | ✅ | 🔴 No `ship log list` CLI command |

---

## 14. Configuration

| Capability | CLI | MCP | UI | Notes |
|---|---|---|---|---|
| Status: list/add/remove | ✅ `config status *` | ✅ `manage_status` | ✅ | — |
| Show active AI config | ✅ `config ai` | ❌ | ✅ | — |
| App settings | ❌ | ❌ | ✅ | CLI/MCP gap |
| Global config path mgmt | ❌ | ❌ | ❌ | Known backlog item |

---

## Confirmed Gaps Summary

### 🔴 Critical (blocks core workflows or is misleading)

| # | Gap | Where |
|---|---|---|
| 1 | `spec update` missing from CLI | CLI |
| 2 | Spec lifecycle (`move_spec`) has zero surface | CLI + MCP + UI? |
| 3 | Release status lifecycle has zero surface | CLI + MCP + UI? |
| 4 | `feature start/done` missing from MCP and UI | MCP + UI |
| 5 | Rules management missing from CLI and MCP | CLI + MCP |
| 6 | Permissions management missing from CLI and MCP | CLI + MCP |
| 7 | MCP `log_decision` should be `create_adr` | MCP naming |
| 8 | Vision management missing from CLI and MCP | CLI + MCP |

### 🔴 UX Friction

| # | Issue |
|---|---|
| 9 | `issue list` not in MCP — agents must call `get_project_info` just to see issues |
| 10 | Feature `release_id`/`spec_id` only settable at create time, no update path |
| 11 | `providers detect` + `connect` is 2 steps, should be 1 |
| 12 | `mcp add` vs `mcp add-stdio` — transport requires upfront knowledge |
| 13 | `session *` duplicates `workspace session *` — confusing for users |
| 14 | Ghost + time tracking hidden from `--help` — undiscoverable |
| 15 | MCP can't list skills, notes, or individual specs |
| 16 | `get_agent_config` (resolved view) is UI-only — agents can't inspect their own config |

---

## Coverage Estimates by Domain

| Domain | CLI | MCP | UI |
|---|---|---|---|
| Issues | ~70% | ~70% | ~90% ⚠️ |
| Features | ~85% | ~55% | ~50% ⚠️ |
| Specs | ~50% | ~40% | ~80% ⚠️ |
| ADRs | ~65% | ~60% | ~90% ⚠️ |
| Releases | ~75% | ~70% | ~70% ⚠️ |
| Notes | ~75% | ~40% | ~90% ⚠️ |
| Workspace / Session | ~90% | ~80% | unknown |
| Git Integration | ~85% | ~60% | ~40% ⚠️ |
| Agent Config | ~75% | ~50% | ~70% ⚠️ |
| Ghost Issues | ~50% | ~40% | 0% |
| Time Tracking | ~50% | ~25% | 0% |
| Events / Logging | ~75% | ~60% | ~70% ⚠️ |

> ⚠️ UI percentages are based on Tauri command signatures only and may undercount
> actual UI screens. Treat UI column as a lower bound until verified on a running build.
