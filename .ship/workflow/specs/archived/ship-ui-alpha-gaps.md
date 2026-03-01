+++
id = "1568239f-716e-4f93-86f9-b7868c52f10d"
title = "Ship UI — Alpha Gap Closure: Notes, Vision, Rules, Workspace, Skills & Permissions"
status = "draft"
created = "2026-03-01T02:02:15.914284350Z"
updated = "2026-03-01T02:02:15.914284350Z"
tags = []
+++

+++
id = ""
title = "Ship UI — Alpha Gap Closure: Notes, Vision, Rules, Workspace, Skills & Permissions"
created = "2026-02-28T22:00:00Z"
updated = "2026-02-28T22:00:00Z"
feature_id = "SjS4tQUW"
release_id = "v0.1.0-alpha"
status = "active"
tags = ["ui", "tauri", "alpha"]
+++

# Ship UI — Alpha Gap Closure

**Feature:** Desktop User Interface (`SjS4tQUW`)  
**Release:** v0.1.0-alpha  
**For:** An agent implementing UI work on the macOS/Windows build machine (not WSL)

---

## Context & Project Overview

Ship (Shipwright) is an AI-assisted project tracking CLI + desktop app. The Tauri desktop app is built at `crates/ui/`. The UI does **not** build in WSL — use a native macOS or Windows host.

**Tech stack:**
- Tauri v2 (Rust backend at `crates/ui/src-tauri/src/lib.rs`)
- React + TypeScript frontend at `crates/ui/src/`
- TanStack Router (file-based routing under `src/routes/`)
- React Query for async data fetching
- shadcn/ui component library
- Specta auto-generated TypeScript bindings at `src/bindings.ts`
- `ShipEvent` typed Tauri events for real-time updates

**Key pattern for all data fetching:**
```typescript
// Import from bindings.ts (auto-generated, do not edit)
import * as commands from '../bindings'

// In components, call via React Query:
const { data, isLoading } = useQuery({
  queryKey: ['notes'],
  queryFn: () => commands.listNotesCmd()
})
```

**Key pattern for Tauri event subscriptions:**
```typescript
import { listen } from '@tauri-apps/api/event'

useEffect(() => {
  const unlisten = listen('ship://notes-changed', () => {
    queryClient.invalidateQueries({ queryKey: ['notes'] })
  })
  return () => { unlisten.then(f => f()) }
}, [])
```

---

## What Already Exists

Existing routes under `src/routes/project/`:
- `overview.tsx` — project overview / dashboard
- `issues.tsx` — issue Kanban board (main feature)
- `specs.tsx` — specs list + editor
- `features.tsx` — features list
- `releases.tsx` — releases list (no delete yet)
- `adrs.tsx` — ADR list + create
- `agents.tsx` — agent config panel (skills currently read-only)
- `settings.tsx` — project settings
- `activity.tsx` — activity log

**Missing routes (alpha blockers):**
- `notes.tsx` — not implemented (backend commands exist)
- `vision.tsx` — not implemented (backend command exists)
- `rules.tsx` — not implemented (backend commands exist)

**Partially complete:**
- `agents.tsx` — skills appear read-only; modes/permissions not exposed
- `releases.tsx` — no delete release command on backend either; skip for alpha

---

## Backend Command Surface (Reference)

All commands are in `crates/ui/src-tauri/src/lib.rs` and exposed in `src/bindings.ts`.

### Notes
```typescript
commands.listNotesCmd()                                        // -> NoteInfo[]
commands.getNoteCmd(file_name: string)                        // -> NoteDocument
commands.createNoteCmd(title: string, content: string, scope?: string) // -> NoteDocument
commands.updateNoteCmd(file_name: string, content: string, scope?: string) // -> void
// No delete note command on backend — skip for alpha
```

`NoteInfo` shape (from Specta bindings):
```typescript
{ file_name: string, title: string, created: string, updated: string, scope: string }
```

`NoteDocument` shape:
```typescript
{ file_name: string, title: string, content: string, scope: string, created: string, updated: string }
```

### Vision
```typescript
commands.getVisionCmd()               // -> Vision
commands.updateVisionCmd(content: string) // -> void
```

`Vision` shape:
```typescript
{ content: string }  // singleton — no file_name, no frontmatter
```

### Rules
```typescript
commands.listRulesCmd()                                        // -> Rule[]
commands.getRuleCmd(file_name: string)                        // -> Rule
commands.createRuleCmd(title: string, content: string)        // -> void
commands.updateRuleCmd(file_name: string, content: string)    // -> void
commands.deleteRuleCmd(file_name: string)                     // -> void
```

`Rule` shape:
```typescript
{ file_name: string, title: string, content: string, created: string }
```

### Skills (existing commands — expose CRUD in UI)
```typescript
commands.listSkillsCmd(scope?: string)                                    // -> Skill[]
commands.getSkillCmd(id: string, scope?: string)                          // -> Skill
commands.createSkillCmd(id: string, name: string, content: string)        // -> void
commands.updateSkillCmd(id: string, name?: string, content?: string, scope?: string) // -> void
commands.deleteSkillCmd(id: string, scope?: string)                       // -> void
```

`Skill` shape:
```typescript
{ id: string, name: string, content: string, source: string, version?: string, author?: string }
```

### Modes
```typescript
commands.listModesCmd()                       // -> ModeConfig[]
commands.addModeCmd(mode: ModeConfig)         // -> void
commands.removeModeCmd(id: string)            // -> void
commands.setActiveModeCmd(id?: string)        // -> void  (null = no mode)
commands.getActiveModeCmd()                   // -> ModeConfig | null
```

`ModeConfig` shape (check bindings.ts for exact fields — it mirrors AgentConfig):
```typescript
{ id: string, name: string, description?: string, skills?: SkillRef[], mcp_servers?: McpServerRef[] }
```

### Permissions
```typescript
commands.getPermissionsCmd()                           // -> Permissions
commands.savePermissionsCmd(perms: Permissions)        // -> void
```

`Permissions` shape (abbreviated — check bindings.ts):
```typescript
{
  tools: { allow: string[], deny: string[] },
  filesystem: { allow: string[], deny: string[] },
  commands: { allow: string[], deny: string[] },
  network: { policy: 'none' | 'localhost' | 'allow-list' | 'unrestricted', allow_hosts: string[] },
  agent: { max_cost?: number, max_turns?: number, require_confirmation: boolean }
}
```

### Workspace
```typescript
commands.getWorkspaceCmd(branch: string)   // -> Workspace | null
```

`Workspace` shape:
```typescript
{ branch: string, feature_id?: string, spec_id?: string, active_mode?: string, resolved_at: string }
```

To get current branch from within Tauri: use a shell command sidecar or pass it from the frontend via `git rev-parse --abbrev-ref HEAD` (simplest: expose a `get_current_branch_cmd()` Tauri command if not already present, or hardcode to read from `SHIP_DIR/.git/HEAD`).

### MCP Servers
```typescript
commands.listMcpServersCmd()                          // -> McpServerConfig[]
commands.addMcpServerCmd(server: McpServerConfig)     // -> void
commands.removeMcpServerCmd(id: string)               // -> void
```

### Providers & Agent Config
```typescript
commands.listProvidersCmd()              // -> ProviderInfo[]
commands.listModelsCmd(provider_id)     // -> ModelInfo[]
commands.getAgentConfigCmd()            // -> AgentConfig (fully resolved)
commands.exportAgentConfigCmd(target)   // -> void  (writes CLAUDE.md etc)
```

---

## Views to Implement

### 1. Notes View (`src/routes/project/notes.tsx`)

**Purpose:** Quick capture of freeform project notes. Not tied to any document type.

**Layout:**
- Two-panel: left list (30%) + right editor (70%)
- Left panel: "Notes" header + "New Note" button (top right), list of note cards sorted by `updated` desc
- Right panel: when a note is selected, show title (editable h1) + markdown editor (textarea for alpha, Monaco for V1)
- Empty left state: "No notes yet. Capture a thought."
- Empty right state: "Select a note or create a new one."

**Note list item:**
```
[title]
[relative date] · [scope badge: project|user]
```

**New Note flow:**
1. Click "New Note" → focus a new blank note in the right panel with empty title
2. Typing in title and hitting Tab focuses the content editor
3. On first blur/save, call `createNoteCmd(title, content, 'project')` (scope always 'project' for alpha)
4. Optimistic update: add to list immediately, replace with server data on success

**Auto-save:** Debounced 1500ms after last keystroke. Show "Saved" indicator top-right for 2s.

**Events:** Listen for `ship://notes-changed` to refresh list.

**Route registration:** Add to the project layout nav alongside other routes.

---

### 2. Vision View (`src/routes/project/vision.tsx`)

**Purpose:** Singleton document — the project's north star. No frontmatter. Pure markdown prose.

**Layout:**
- Full-width editor — no list panel needed (singleton)
- Header: "Vision" (h1 in page, not editable) + "Edit" toggle button top-right
- View mode: rendered markdown (use a markdown renderer — `react-markdown` is likely already in deps, check package.json; if not, use a simple textarea with `white-space: pre-wrap`)
- Edit mode: textarea fills the space, identical to view mode but editable
- "Save" button appears in edit mode; "Cancel" reverts

**Empty state:** "No vision document yet. Write your project's north star — the single paragraph that explains why this project matters and where it's going."

**On load:** Call `getVisionCmd()`. If content is empty/null, show empty state + "Write Vision" button that enters edit mode.

**On save:** Call `updateVisionCmd(content)`. Emit optimistic update.

**Events:** Listen for `ship://config-changed` to refresh (vision changes emit config-changed).

---

### 3. Rules View (`src/routes/project/rules.tsx`)

**Purpose:** Agent rules — always-active markdown files that get injected into every agent context. Think of them as persistent system prompt snippets.

**Layout:**
- Two-panel: left list (30%) + right editor (70%), same pattern as Notes
- Left: "Rules" header + "New Rule" button
- Rules are always active if they exist (no on/off toggle needed for alpha)

**Rule list item:**
```
[title]
[relative date]
[trash icon button — with confirm dialog before delete]
```

**New Rule flow:** Same as Notes. On create, call `createRuleCmd(title, content)`.

**Edit:** Select rule → edit in right panel → auto-save debounced 1500ms → `updateRuleCmd(file_name, content)`.

**Delete:** Trash icon → "Delete rule?" confirm dialog → `deleteRuleCmd(file_name)`.

**Events:** Listen for `ship://config-changed`.

---

### 4. Agents Panel — Full CRUD (`src/routes/project/agents.tsx`)

The current `agents.tsx` shows skills read-only. This task completes it.

**Target: 4-tab layout within agents.tsx:**
- Tab 1: **Skills** — full CRUD (currently read-only)
- Tab 2: **MCP Servers** — already partially implemented, verify edit/remove work
- Tab 3: **Modes** — new
- Tab 4: **Permissions** — new

#### Tab 1: Skills

**List:** Table with columns: Name, ID, Source badge (custom/builtin/community), Actions (Edit | Delete).

**Skill detail (right panel or modal):**
- Name (editable text)
- ID (display only — set on create, immutable after)
- Content (markdown editor — textarea for alpha)
- Source badge (display only)

**New Skill flow:**
1. "New Skill" button → modal with fields: Name, ID (auto-slugified from name), Content
2. On confirm: `createSkillCmd(id, name, content)`

**Edit:** Click skill row → open detail panel → edit name/content → "Save" → `updateSkillCmd(id, name, content, scope)`

**Delete:** Trash icon → confirm → `deleteSkillCmd(id, scope)`

**Builtin skills** (source="builtin"): Show lock icon. Allow viewing but not editing/deleting.

#### Tab 3: Modes

**List:** Table with columns: Name, ID, Active indicator, Actions (Set Active | Delete).

**Active mode:** Show a badge "ACTIVE" on the active row. "Set Active" button on inactive rows calls `setActiveModeCmd(id)`. "Clear Mode" button in header (or deactivate the active row) calls `setActiveModeCmd(null)`.

**New Mode:** "New Mode" button → modal with Name and ID fields. Content/config editing is out of scope for alpha — just create a named mode with defaults.

**Delete:** `removeModeCmd(id)`.

**On load:** Call `listModesCmd()` and `getActiveModeCmd()` to know which is active.

#### Tab 4: Permissions

**Purpose:** Visual editor for `agents/permissions.toml`. Lets users configure what agents can do.

**Layout — 5 sections in a single scrollable panel:**

**1. Tools**
```
Allow: [chip list with × button] [+ Add input]
Deny:  [chip list with × button] [+ Add input]
```
Placeholder: "e.g. Bash, Edit, Write"

**2. Filesystem**
```
Allow paths: [chip list] [+ Add]
Deny paths:  [chip list] [+ Add]
```
Placeholder: "e.g. /home/user/project/**"

**3. Commands**
```
Allow commands: [chip list] [+ Add]
Deny commands:  [chip list] [+ Add]
```

**4. Network**
```
Policy: [radio group: None | Localhost | Allow List | Unrestricted]
Allow hosts: [chip list, only shown when policy=allow-list] [+ Add]
```

**5. Agent Limits**
```
Max cost per session: [number input, $ prefix]
Max turns:            [number input]
Require confirmation: [toggle switch]
```

**Save:** Single "Save Permissions" button at bottom → `savePermissionsCmd(perms)`.

**Load:** `getPermissionsCmd()` on mount.

---

### 5. Workspace Panel (Project Layout — Sidebar Widget)

**Purpose:** Show the current branch → linked feature/spec → resolved agent config. Helps the user know "what mode am I in right now?"

**Location:** Add as a collapsible widget at the bottom of the left sidebar (under nav links), or as a dedicated section in `overview.tsx`. Recommend sidebar widget.

**Display:**
```
┌─ Workspace ──────────────────┐
│ Branch: feature/my-feature   │
│ Feature: Desktop UI          │
│ Mode: (none)                 │
│ [Sync Config]                │
└──────────────────────────────┘
```

**Implementation:**
1. Read current branch: expose `get_current_branch_cmd()` Tauri command if not present, OR use `invoke('plugin:shell|execute', ...)` to run `git rev-parse --abbrev-ref HEAD`. Simplest: add a Tauri command that reads `.git/HEAD`.
2. Call `getWorkspaceCmd(branch)` → get `Workspace | null`
3. If workspace exists, fetch linked feature title via `getFeatureCmd(workspace.feature_id)`
4. Display branch, feature title (linked → opens features.tsx), active mode name
5. "Sync Config" button → calls `exportAgentConfigCmd('claude')` → shows success toast

**Empty state (no workspace for this branch):** "No workspace for current branch."

---

## Implementation Priority

Do these in order. Each is independently shippable.

### P0 — Missing Views (Alpha Blockers)
1. **Vision view** — simplest (singleton, no list)
2. **Notes view** — list + editor, no delete needed for alpha
3. **Rules view** — list + editor + delete

### P1 — Agents Panel Completion
4. **Skills CRUD** — complete what's there; unblock skill management
5. **Modes tab** — list + set active (create with defaults)
6. **Permissions tab** — full form editor

### P2 — Workspace Panel
7. **Workspace sidebar widget** — read-only display for alpha; Sync Config button

---

## Adding Routes

New routes follow TanStack Router file convention. Add route files at:
- `src/routes/project/notes.tsx`
- `src/routes/project/vision.tsx`
- `src/routes/project/rules.tsx`

Then register them in the root layout/nav. Check `src/routes/__root.tsx` and the project layout component for the nav link pattern — add links using the same style as existing nav items.

---

## Nav Link Labels

Use these exact labels for consistency with CLI/MCP surface:
- Notes
- Vision
- Rules

In `agents.tsx`, the tab labels should be:
- Skills
- MCP Servers
- Modes
- Permissions

---

## Do Not

- Do not build a release delete view — no backend command exists; skip for alpha
- Do not build AI chat panels — out of scope for this spec; see `ship-ui-vision-production-roadmap.md`
- Do not edit `src/bindings.ts` — it is auto-generated by Specta; re-run `cargo tauri build` or `specta` to regenerate after backend changes
- Do not add new Tauri commands unless a view absolutely requires one (the workspace branch read is the only likely addition)
- Do not implement light mode, time tracking UI, or ghost issues scanner — V1 only

---

## Reference

- Full UI vision and V1 roadmap: `.ship/workflow/specs/draft/ship-ui-vision-production-roadmap.md`
- Feature document: `.ship/project/features/in-progress/desktop-user-interface.md` (id: `SjS4tQUW`)
- Tauri commands: `crates/ui/src-tauri/src/lib.rs`
- TypeScript bindings: `crates/ui/src/bindings.ts`
- Existing route examples: `crates/ui/src/routes/project/specs.tsx`, `notes.tsx` (once created)
