+++
title = "Tauri UI — Alpha Polish Pass"
created = "2026-02-23T02:29:08.234849952Z"
updated = "2026-02-23T02:29:08.234851552Z"
tags = []
links = []
+++

Everything the macOS Tauri session needs to complete for alpha done criteria.

## Views to build / complete

### Kanban (default landing view) — criteria #5, #6
- Columns driven by `config.toml` `[[statuses]]`
- Cards: title, assignee, tags
- Drag-and-drop moves the file to the correct status folder + updates `updated` timestamp
- Click card → Issue Detail
- File-watch refresh so agent moves appear live (criterion #9)

### Issue Detail — criterion #9
- Full markdown render
- Edit in place (title, body, assignee, tags, spec ref)
- Frontmatter fields as a form, not raw TOML
- Auto-saves on blur

### Spec Editor — criteria #3, #4
- Split view: left = editable markdown, right = AI conversation via MCP sampling
- "Extract Issue" button → creates issue pre-populated from spec context
- Scoped to the open spec

### ADR List
- Table: status, date, title
- Click to read full ADR
- "New ADR" button

### Settings
- GUI for `config.toml`: statuses (add/remove/reorder/recolor), git behaviour, templates
- Replaces hand-editing TOML for non-technical users

## Polish requirements
- Empty states for every view: one sentence + one CTA
- Spec Editor empty state especially welcoming (target: non-technical PMs)
- Typography and spacing consistent throughout
- Should feel like Linear, not a weekend project

## App icon
- `src-tauri/icons/` currently has placeholder Tauri icons
- Run `tauri icon logo.svg` from `crates/ui/` to regenerate all sizes from the SVG
- Or: `convert logo.svg -resize 512x512 icon.png && tauri icon icon.png`

## MCP auto-start
- MCP server should start automatically when `ship ui` / app launches
- Visible status indicator in UI (connected / not connected)
- Spec: recommend auto-start with indicator

## macOS-specific fixes
- Fix directory picker (open project dialog) — currently broken on macOS
- Test `ship init` path from the app onboarding flow

## File watching
- Kanban and Issue Detail must refresh when `.ship/issues/` changes on disk
- Enables criterion #9: agent updates issue → change appears in UI immediately
- Use Tauri's `watch` plugin or `notify` crate