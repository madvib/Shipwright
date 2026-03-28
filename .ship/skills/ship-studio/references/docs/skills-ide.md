---
group: Studio
title: Skills IDE
section: guide
order: 2
---

# Skills IDE

The Skills IDE is a three-panel editor for browsing and editing skill files. It is desktop-only -- on small screens, Studio shows a message directing the user to a wider viewport.

## Layout

The IDE has three panels arranged horizontally:

1. **File Explorer** (left, 240px) -- Skill directory tree with search and folder expand/collapse.
2. **Editor** (center, flexible) -- Code editor with syntax highlighting, line numbers, and tab management.
3. **Detail Panel** (right, 320px) -- Metadata, variables, and usage info for the active skill. Togglable via the panel button in the editor toolbar.

## File Explorer

The explorer splits skills into two sections:

- **Project Skills** -- Skills from the project `.ship/skills/` directory. Includes a "+" button to create new skills.
- **Library** -- Skills from the global `~/.ship/skills/` directory. Displayed with a violet "library" badge.

Each skill is a collapsible folder. When expanded, it shows the full file tree organized by directory groups:

- Root files (SKILL.md)
- `assets/` (vars.json, templates)
- `references/` (docs, API references)
- `evals/` (eval definitions)
- `scripts/` (helper scripts)

Colored dots on each skill folder indicate content: amber for variables, emerald for reference docs, violet for evals.

### Search

The search bar at the top of the explorer filters skills by name or ID. The collapse-all button (double chevron icon) collapses every folder except the one containing the active file.

### Adding Files

When a skill folder is expanded, an "Add file" button appears below the root files. Clicking it opens a popover with spec-compliant file options:

| Option | Path | Description |
|--------|------|-------------|
| Variables | `assets/vars.json` | Typed config schema for smart skill variables |
| Reference docs | `references/docs/index.md` | Human and agent readable documentation |
| API reference | `references/api/index.md` | API tables and external specs |
| Script | `scripts/run.sh` | Helper script referenced in SKILL.md |
| Template | `assets/templates/config.md` | Reusable config snippet |

Each option is hidden if the file already exists. Selecting an option creates the file with a sensible template and opens it in the editor.

### Creating Skills

The "+" button in the Project Skills header opens a dialog to create a new skill. Enter a skill ID (lowercase, hyphens) and Studio generates a SKILL.md with valid frontmatter. The new skill appears in the explorer immediately as a local draft.

## Editor

The editor occupies the center panel. It displays the content of the active file with syntax highlighting and line numbers.

### Tabs

Open files appear as tabs along the top of the editor. Each tab shows the filename. A dot indicator marks unsaved tabs. Click the X to close a tab. Closing a tab discards its draft content.

### Breadcrumb Toolbar

Above the tabs, a breadcrumb shows the current path: `skills / {skill-name} / {file-path}`. The toolbar also contains:

- **Save button** -- Visible when the active file has unsaved changes. Triggers an MCP `write_skill_file` call.
- **Panel toggle** -- Opens or closes the detail panel on the right.

### Syntax Highlighting

The editor uses a custom highlighter (`editor-highlight.ts`) that colorizes YAML frontmatter, Markdown headings, code blocks, bold/italic text, links, and inline code. The implementation overlays a highlighted `div` on top of a transparent `textarea` so the user types in a native input while seeing colorized output.

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+S / Ctrl+S | Save the active file |

## Detail Panel

The detail panel shows metadata about the currently active skill. It has three tabs:

### Variables Tab

Displays the skill's `vars.json` schema as a list of variable cards. Each card shows:

- Variable label and key
- Type badge (`string`, `bool`, `enum`, `array`, `object`)
- Storage scope icon (globe for global, folder for project, user for local)
- Default value
- Description text
- For enum types: all allowed values with the default highlighted
- For bool types: a visual toggle showing the default state

If no variables are defined, the tab shows a prompt to add `vars.json` with a button that creates the file and opens it in the editor.

The tab also displays the CLI command to set variables: `ship vars set {skill-id} <key> <value>`.

### Info Tab

Displays skill identity and metadata:

- **Identity** -- stable-id, authors, license, source origin
- **Tags** -- Rendered as chips
- **Files** -- Full file list in monospace
- **Description** -- From frontmatter
- **Frontmatter warnings** -- Validation errors and warnings (missing required fields, invalid values) shown as colored alerts

### Used By Tab

Lists all agents that reference this skill. Each agent is shown as a card with the agent ID and name. If no agents reference the skill, a message says so.

## Save Flow

1. **Edit** -- User types in the editor. Changes are tracked as drafts in a React state map keyed by tab ID (`{skillId}::{filePath}`).
2. **Draft persistence** -- After 800ms of inactivity, drafts are written to IndexedDB (`ship-skills-ide-drafts`). This survives page reloads and browser restarts.
3. **Save** -- User presses Cmd+S or clicks the Save button. Studio calls MCP `write_skill_file` with the skill ID, file path, and content.
4. **Confirmation** -- On success, the draft is marked as saved (original content updated to match). The pull query is invalidated, triggering a refresh from the CLI to confirm the file was written.

Unsaved changes are indicated by a dot on the tab and a dot on the Save button. The editor tracks which tabs have content that differs from the last-known original.

## Offline Support

When the CLI is disconnected:

- A yellow banner appears: "CLI disconnected -- edits saved locally, connect to sync"
- Edits continue to work and are persisted to IndexedDB
- The file explorer shows cached skill data from the last successful pull
- Saves fail silently (the MCP call errors) but drafts are preserved locally
- When the CLI reconnects, the user can re-save their changes

### State Persistence

| Data | Storage | Purpose |
|------|---------|---------|
| Open tabs, active tab, expanded folders | localStorage | Session continuity across page reloads |
| File drafts (unsaved content) | IndexedDB | Offline resilience, no 5MB limit |
| Skill data (pull cache) | TanStack Query cache | Fast re-renders, auto-refetch when connected |
