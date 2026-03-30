---
group: Studio
title: Skills IDE
order: 2
---

# Skills IDE

The Skills IDE is a three-panel editor for browsing and editing skill files. It is desktop-only -- on small screens, Studio shows a message directing the user to a wider viewport.

## Layout

Three panels arranged horizontally:

1. **File Explorer** (left) -- Skill directory tree with search, folder expand/collapse, and file creation.
2. **Editor** (center, flexible width) -- Code editor with syntax highlighting, tab management, and save controls.
3. **Preview Panel** (right, toggleable) -- Variables, metadata, and agent usage for the active skill.

## File explorer

The explorer shows all skills from the project `.ship/skills/` directory. Each skill is a collapsible folder showing its file tree organized by directory group:

- Root files (`SKILL.md`)
- `assets/` (vars.json, templates)
- `references/` (docs, API references)
- `evals/` (eval definitions)
- `scripts/` (helper scripts)

Colored dots on each skill folder indicate content types: amber for variables, emerald for reference docs, violet for evals.

### Search and navigation

The search bar filters skills by name or ID. The collapse-all button collapses every folder except the one containing the active file.

### Adding files

When a skill folder is expanded, an "Add file" button offers these options:

| Option | Path | Description |
|--------|------|-------------|
| Variables | `assets/vars.json` | Typed config schema for skill variables |
| Reference docs | `references/docs/index.md` | Documentation |
| API reference | `references/api/index.md` | API tables and external specs |
| Script | `scripts/run.sh` | Helper script |
| Template | `assets/templates/config.md` | Reusable config snippet |

Each option is hidden if the file already exists. Selecting one creates the file with a template and opens it in the editor.

### Creating skills

The "+" button opens a dialog to create a new skill. Enter a skill ID (lowercase, hyphens only). Studio generates a `SKILL.md` with valid frontmatter. The new skill appears immediately as a local draft.

## Editor

The center panel displays the active file with syntax highlighting and line numbers.

### Tabs

Open files appear as tabs along the top. Each tab shows the filename. A dot indicates unsaved changes. Closing a tab discards its draft content. Tab IDs use the format `{skillId}::{filePath}` to handle multiple files across skills.

### Toolbar

Above the tabs, a breadcrumb shows the current path: `skills / {skill-name} / {file-path}`. The toolbar contains:

- **Save button** -- Visible when the active file has unsaved changes. Calls MCP `write_skill_file`.
- **Panel toggle** -- Opens or closes the preview panel.

### Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+S / Ctrl+S | Save the active file |

## Preview panel

The preview panel shows metadata about the currently active skill across three tabs.

### Variables tab

Displays the skill's `vars.json` schema as variable cards. Each card shows the label, key, type badge (`string`, `bool`, `enum`, `array`, `object`), storage scope (global, project, local), default value, and description. Enum types show all allowed values. Bool types show a visual toggle.

If no variables exist, the tab prompts to add `vars.json`. When connected to the CLI, resolved variable values are fetched via `get_skill_vars` and can be set inline via `set_skill_var`.

### Info tab

Displays skill identity (stable-id, authors, license, source), tags, file list, description, and frontmatter validation warnings.

### Used By tab

Lists all agents that reference this skill, shown as cards with agent ID and name.

## Save flow

1. **Edit** -- User types in the editor. Changes are tracked as drafts in a React state map keyed by tab ID.
2. **Draft persistence** -- After 800ms of inactivity, drafts are written to IndexedDB (`ship-skills-ide-drafts`). This survives page reloads.
3. **Save** -- User presses Cmd+S or clicks Save. Studio calls `write_skill_file` with the skill ID, file path, and content.
4. **Confirmation** -- On success, the draft is marked as saved (original content updated to match). The query cache is invalidated, triggering a re-pull from the CLI.

Unsaved changes show as a dot on the tab. The save button is visible only when changes exist.

## Offline behavior

When the CLI is disconnected:

- A yellow banner appears: "CLI disconnected -- edits saved locally, connect to sync"
- Edits continue to work and persist to IndexedDB
- The explorer shows cached skill data from the last successful pull
- Save calls fail (MCP errors) but drafts are preserved locally
- When the CLI reconnects, the user can re-save

### State persistence

| Data | Storage | Purpose |
|------|---------|---------|
| Open tabs, active tab, expanded folders | localStorage | Layout continuity across reloads |
| File drafts (unsaved content) | IndexedDB | Offline resilience, no 5MB limit |
| Skill data (pull cache) | TanStack Query | Fast re-renders, reactive invalidation |
