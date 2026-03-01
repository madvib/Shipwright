+++
id = "KhW8FCCB"
title = "Ship UI — Vision & Production Roadmap"
created = "2026-02-28T21:15:00Z"
updated = "2026-02-28T21:15:00Z"
tags = []
+++

# Ship UI — Vision & Production Roadmap

**Status:** Active planning  
**Owner:** macOS agent session  
**Last Updated:** 2026-02-22

---

## The One-Line Vision

Ship's UI is where specs become issues become shipped software — with an AI collaborator built into every surface, not bolted on after the fact.

---

## Current State: What Exists

The UI is a functional prototype. It has a working kanban board, basic issue CRUD, a collapsible activity log, and a settings panel with three fields. It looks dark and clean. It does not yet feel like a product.

**What works:**
- Project open/switch (folder picker)
- Kanban: 4 hardcoded columns, create/edit/move/delete issues
- ADR creation (modal), list view (cards, read-only, 200-char truncation)
- Activity log (collapsible, relative timestamps, actor coloring)
- Settings: author name, default status, notification toggle (non-functional)

**What's broken or missing (the full list):**
- See sections below

---

## Critical Bugs (Fix First)

### 1. Project Detection Scans System Directories

`list_projects` in `src-tauri/src/lib.rs` walks the filesystem and finds `.ship` directories anywhere — including `.Trash`, `node_modules`, and temp paths. This shows garbage in the project switcher.

**Fix:** Projects must come from the registry (`~/.ship/registry.toml`) only. Discovery scanning must be removed or scoped to a user-defined search path. Tauri's `pick_and_open_project` is the correct path for adding new projects — it should register them in the registry via `register_project`, not just set `active_project` in-memory.

### 2. Type Misalignment — Frontend vs Backend

`types.ts` defines `Issue` with `created_at`/`updated_at` fields. The logic crate (after TOML migration) uses `created`/`updated`. This means any issue created or read via the backend will have blank timestamps in the UI.

**Fix:** Update `types.ts` to match current `IssueMetadata`:
```typescript
export interface Issue {
  title: string;
  description: string;
  assignee: string;
  tags: string[];
  spec: string;
  created: string;   // was created_at
  updated: string;   // was updated_at
  links: IssueLink[];
}

export interface IssueLink {
  type: string;
  target: string;
}
```

### 3. Status Columns Are Hardcoded

`IssueList.tsx` and `STATUS_CONFIG` in `types.ts` hardcode exactly 4 statuses: backlog, in-progress, blocked, done. The backend supports configurable statuses via `config.toml`. A project with a `review` column will never show it.

**Fix:** Load statuses from backend config at startup. Drive kanban columns, status chips, and color mappings from config data, not frontend constants.

### 4. ADR Type Misalignment

`types.ts` defines `ADR` with a `decision` field. The logic crate `AdrMetadata` / `ADR` struct uses `body` for content and `metadata.status`, `metadata.date`, `metadata.title`, `metadata.tags`. The Tauri backend's `list_adrs_cmd` command needs to be audited against the current struct shape.

### 5. `ship ui` → `ship`

Per the spec and user preference, the desktop app launches with `ship` (no subcommand). The Tauri `tauri.conf.json` should ensure the product name is just "Ship" and the launch binary is correct.

---

## View Specifications

### View 1: Kanban Board (Default Landing)

**Route/section:** `issues`

The kanban is the heartbeat of the app. It should feel like Linear's board view.

**Columns:**
- Driven from `config.statuses` — not hardcoded. Column order = config order.
- Each column: header (status name, colored dot, count), scrollable card list, "+" button to create in that column.
- Empty column shows a subtle empty state inline (no card, just a dim "drop issues here" text).

**Issue Cards:**
- Title (bold, 14px, full text — no truncation unless > 3 lines)
- Tags (colored pills, from `issue.tags`)
- Assignee avatar/initials if set
- Spec reference link if set (`issue.spec`)
- Creation date (right-aligned, muted)
- Hover: subtle lift effect (box-shadow transition)

**Drag and Drop:**
- Use `@dnd-kit/core` + `@dnd-kit/sortable`. Do NOT use react-beautiful-dnd (deprecated).
- Drag a card between columns → calls `move_issue_status` Tauri command.
- Drag within a column → reorders (visual only for alpha; persistence is a v1 problem).
- Drop target columns highlight (border + dim bg) while dragging.
- Card being dragged shows at 90% opacity with a drop shadow.
- Drag handle: either the entire card or a grip icon at top-left — decide and be consistent.

**New Issue:**
- "+" button in any column opens `NewIssueModal` pre-set to that column's status.
- Keyboard shortcut: `N` opens new issue modal (when no input focused).

**Filter Bar (above board):**
- Search input (filters card titles in real time, client-side)
- Assignee filter chip (dropdown)
- Tag filter chips (multi-select)
- "Clear filters" only appears when filters are active

### View 2: Issue Detail Panel

**Trigger:** Click any issue card. Opens as a right-side panel (520px) with backdrop.

**Layout (top to bottom):**
1. **Header row:** Status badge (pill) | Spec reference link | Close button
2. **Title:** Large editable text (single textarea, no border until focused, feels like inline editing)
3. **Meta row:** `created by [assignee]` · `[relative date]` · `[filename]`
4. **Tags:** Editable tag pills. Click to remove, type to add. Shows suggestions from project tag config.
5. **Assignee:** Editable inline field.
6. **Spec:** Editable inline field (freetext for alpha; autocomplete from spec list in v1).
7. **Links section:** List of typed links (`blocks`, `blocked-by`, `relates-to`). Add link button.
8. **Description:** Full markdown editor. Minimum viable: a textarea that renders markdown on blur (toggle edit/preview). Bonus: a split edit/preview like GitHub's issue editor.
9. **Status row:** Pill selector. Clicking a status moves the issue immediately (optimistic update).
10. **Actions:** Save (if dirty) | Delete (with confirm)

**Keyboard:** `Escape` closes the panel. `Cmd+S` saves.

### View 3: Specs

**Route/section:** `specs` — this section does not yet exist in the UI.

Specs are the PRIMARY document type. This is where work begins. The spec editor is Ship's signature feature.

**List View:**
- Clean table: title, status (draft/active/archived), last updated, author.
- "New Spec" button (top right).
- Click row → open Spec Editor.

**Spec Editor — Split View:**
Left panel (60%): Markdown editor. For alpha: textarea with toolbar (bold, italic, heading, code). Frontmatter fields shown as a collapsible form above the editor (title, status, author, tags).

Right panel (40%): AI Chat panel.
- Header: "Refine with AI" | model indicator (from MCP connection)
- Chat thread: messages between user and AI, rendered markdown, code blocks
- Input: multiline textarea at bottom, send button, Cmd+Enter shortcut
- AI has full context of this spec (passed in system prompt)
- **"Extract Issues" button** (prominent, at top of right panel or below spec editor): calls `generate_issue_description` / extract-from-spec MCP tool, shows a list of suggested issues with checkboxes, "Create Selected" creates them all.
- **"Draft ADR" button:** calls `generate_adr` MCP tool, pre-populates the New ADR modal.

When the right panel is hidden (toggle button), the editor takes full width.

**Empty State:** "Specs are living documents. Start with a rough idea — refine it with AI — extract issues. No perfect prose required."

### View 4: ADR List + Detail

**Route/section:** `adrs`

**List View:**
- Table columns: status badge | date | title | tags
- Status color coding: proposed=blue, accepted=emerald, rejected=red, superseded=amber, deprecated=zinc
- Click row → ADR Detail panel (right-side, same pattern as issue detail)
- "New Decision" button

**ADR Detail Panel:**
- Title (editable)
- Status (editable pill selector: proposed → accepted, rejected, superseded)
- Date (display only — set on create)
- Tags (editable)
- Spec reference (editable)
- Body sections: Context, Decision, Consequences — rendered as structured markdown. Editable.
- "Archive" button (moves to deprecated status)

### View 5: Settings

**Route/section:** `settings`

This is a proper settings UI, not a 3-field form. Two tabs: **Project** and **Global**.

#### Project Settings

**Statuses tab:**
- Drag-to-reorder list of current statuses (each row: color swatch | id | name | delete button)
- "Add Status" inline form: id, name, color picker (8 preset colors)
- Cannot delete a status that has issues in it (show count, offer migration)

**Tags tab:**
- Same CRUD list pattern: id, name, color
- "Add Tag"

**Git tab:**
- Two columns: "Committed to git" / "Ignored by git"
- Categories: issues, specs, adrs, config.toml, templates, log.md
- Toggle switches to move between committed/ignored
- Explains what each means inline

**Templates tab:**
- Three code editors: ISSUE.md, SPEC.md, ADR.md
- Syntax highlighting (codemirror or similar, minimal)
- Reset to defaults button

**Danger Zone:**
- "Unregister Project" (removes from registry, does not delete files)

#### Global Settings

**User tab:**
- Name (text)
- Email (text)

**Appearance tab:**
- Theme: Dark (only option for alpha, others greyed out with "coming soon")
- Accent color: 6 presets (blue, purple, emerald, amber, rose, zinc)

**MCP tab:**
- Port: number input (default 7700)
- Enabled: toggle
- Connection status indicator (green dot = server running, red = stopped)
- "Copy connection string" button (for Claude Desktop / Cursor config)

**Defaults tab:**
- Default issue status (dropdown)
- Default editor (text input: `code`, `nvim`, etc.)

### View 6: Activity Log (Full Page)

**Route/section:** `log`

The collapsible panel at the bottom of issues view is fine for a quick glance. This is the full-page version.

**Layout:**
- Timeline view. Each entry: timestamp (left, muted, monospace) | actor badge (`[ship]` / `[agent:claude]`) | action (colored) | details
- Actor badge color: ship=zinc, agent=blue, human=emerald
- Infinite scroll or pagination (last 200 entries, "load more")
- Filter bar: by actor, by action type, by date range
- Export button: copies log as plain text or downloads as `.md`

---

## AI Features — Integration Map

The AI panel is not a tab you navigate to. It lives _inside_ the relevant view.

| Surface | AI Feature | Mechanism |
|---------|-----------|-----------|
| Spec Editor | Chat with AI about this spec | `mcp__ship__generate_issue_description` context |
| Spec Editor | Extract issues from spec | New `extract_issues_from_spec` MCP tool |
| Spec Editor | Draft ADR from spec context | `mcp__ship__generate_adr` |
| Issue Detail | Suggest subtasks | `mcp__ship__brainstorm_issues` |
| ADR List | Generate ADR from decision prompt | `mcp__ship__generate_adr` |
| New Issue Modal | AI-expand title to description | `mcp__ship__generate_issue_description` |

**Implementation note:** All AI calls go through the Tauri backend → MCP server → `generate_*` functions. The frontend should not call any AI API directly. The MCP server handles model selection and API keys (or MCP sampling where available).

**Loading states:** AI calls can be slow. Every AI-triggered action must show a clear loading state — spinner in the button, streaming text if possible, never a frozen UI.

**Error handling:** Model unavailable, API error, sampling rejected — each must surface a clear, non-technical message. "Couldn't reach AI — check your MCP connection in Settings."

---

## Backend Hardening (Tauri / Logic Layer)

These are not UI features — they are infrastructure changes that the UI depends on.

### 1. Project Registry as Source of Truth

Remove filesystem scanning for projects. The Tauri backend should:
- On startup, call `load_registry()` to get registered projects
- `pick_and_open_project` → registers the project via `register_project`, persists to `~/.ship/registry.toml`
- `list_projects` → returns registry contents only, never scans

### 2. File Watching for Live Updates

When the MCP server or CLI updates an issue, the Tauri kanban board should reflect it without a manual refresh.

Use `notify` crate (already likely in workspace or add it). Watch `.ship/issues/**` and `.ship/log.md`. On change event, emit a Tauri event (`ship://issues-changed`, `ship://log-changed`). Frontend subscribes with `listen()` and refreshes the relevant data.

This is alpha criterion #9 ("Agent updates an issue — change appears in the Kanban board").

### 3. Config-Driven Status Loading

Tauri command `get_config()` must return full `ProjectConfig` including `statuses: Vec<StatusConfig>`. The frontend drives all status rendering from this — no hardcoded status names or colors anywhere in the frontend.

New Tauri command needed: `get_project_config() -> ProjectConfig`

### 4. Tauri Command API Cleanup

Audit all existing Tauri commands against current logic crate API. Specific known issues:
- `list_items` → rename to `list_issues` for clarity
- `create_new_issue` → must pass `assignee`, `tags` fields now that `IssueMetadata` has them
- `create_new_adr` → must use new `AdrMetadata` struct fields
- `get_app_settings` → currently loads `GlobalConfig`; add `get_project_config` for `ProjectConfig`

### 5. Tauri Error Handling

All Tauri commands currently return `Result<T, String>` — adequate for alpha but the error strings need to be user-readable. No Rust panic messages should reach the frontend. Wrap with a proper error enum or at minimum sanitize messages before returning.

### 6. MCP Server Lifecycle

Per the alpha spec open question: the MCP server should auto-start when the Tauri app launches, and its status should be visible in Settings → Global → MCP.

Tauri sidecar config in `tauri.conf.json` for `ship-mcp` binary. Use `tauri-plugin-shell` sidecar API to start it. Expose `mcp_status()` and `mcp_start()` / `mcp_stop()` commands.

---

## Quality Bar

This is a commercial product. The quality bar is:

- **No layout shift** on data load — use skeleton loaders
- **No silent failures** — every error surfaces somewhere the user can see it
- **Keyboard navigable** — primary actions have keyboard shortcuts documented
- **Consistent spacing** — 8px grid, no one-off margins
- **Typography** — one font, two weights. Title 15-16px bold, body 13-14px regular, meta 12px muted. No italic except in markdown rendering.
- **Animations** — 150ms ease-out for panels, 100ms for hover states. Nothing longer.
- **Empty states** — every view has one. Meaningful copy, one CTA.
- **Loading states** — skeleton screens, not spinners-in-the-middle (except AI calls)

---

## Implementation Priority Order

For the macOS agent session, this is the recommended order:

### Phase 1 — Fix What's Broken
1. Fix project detection (registry only, no filesystem scan)
2. Fix `types.ts` field names (`created`, `updated`, `assignee`, `tags`)
3. Fix status loading from config (not hardcoded)
4. Fix ADR type alignment

### Phase 2 — Core Features Missing from Alpha Criteria
5. File watching → live kanban updates
6. Specs section (list + editor — even without AI chat)
7. ADR detail panel (view + edit existing ADRs)
8. Drag-and-drop on kanban (`@dnd-kit`)
9. MCP sidecar auto-start + status in settings

### Phase 3 — AI Integration
10. AI panel in Spec Editor (chat + extract issues + draft ADR)
11. AI description expansion in New Issue modal
12. AI subtask suggestions in Issue Detail

### Phase 4 — Settings
13. Full project settings (statuses CRUD, tags CRUD, git toggles, templates)
14. Full global settings (user, MCP config, accent color)

### Phase 5 — Polish
15. Filter bar on kanban
16. Full-page log view with actor filter
17. Issue linking UI
18. Keyboard shortcuts
19. Skeleton loaders throughout
20. Empty state polish

---

## What to NOT Build in Alpha

Per the alpha spec — these belong in v1:
- Light mode
- Time tracking UI
- Ghost issues scanner UI
- Plugin marketplace
- Cross-project linking
- Export/import
- Mobile
- Multi-user / roles
- Undo/redo
