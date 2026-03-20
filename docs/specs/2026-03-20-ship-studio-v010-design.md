# Ship Studio v0.1.0 — UI Design Spec

Date: 2026-03-20
Status: Approved

## Vision

Ship Studio is a visual editor for agent configuration. It defines a universal agent spec and compiles to every major coding agent (Claude, Gemini, Codex, Cursor). The UI must instantly communicate: "you're configuring an agent, here's everything you can control, and here's exactly what gets generated."

Ship is a package manager for agents and skills. The registry is a first-class experience.

## Information Architecture

Three top-level pages, navigated via a bottom dock:

| Dock | Page | Purpose |
|---|---|---|
| Agents | Agent list + detail editor | Your agents. Click to configure. |
| Skills | IDE-lite skill editor | Create, edit, manage skills. |
| Registry | Marketplace | Browse/install agents, skills, MCP servers. |

Plus a **Compile** button in the dock that triggers provider output generation with inline provider toggles.

### What is NOT a page

- **Export** — compile is a button, not a page. File download is a nested action per agent/skill.
- **Providers** — provider selection happens at compile time (dock button). Per-provider settings (model, hooks, mode) live in the agent's Settings section.
- **Rules** — live in agent Settings section, not a standalone page.
- **Hooks** — live in agent Settings section (provider-specific), not standalone.
- **Subagents** — a picker section within the agent detail page, not standalone.
- **Permissions** — a section within the agent detail page.

### Header

- Ship logo + nav (Studio, Workflow, Jobs)
- Publish button (publish to registry)
- GitHub button (connect GitHub App)
- Account avatar

## Page 1: Agent Detail

**Mockup:** `.superpowers/brainstorm/approved/agent-detail.png`
**HTML:** `.superpowers/brainstorm/approved/agent-detail-v1.html`

### Layout

Single scrollable page. Breadcrumb back to agent list. Agent header with avatar, name, description, provider tags, stat summary.

### Sections (in order)

#### Skills
- Chip grid layout. Each chip: 2-letter icon, name, version, source (project/registry).
- "Add" button opens autocomplete palette searching library + registry.
- Chip click navigates to skill in Skills IDE.
- Remove via × on chip.

#### MCP Servers
- Same chip grid pattern as skills.
- Each chip shows a badge: "all" (green), "8/18" (orange/partial), or tool count.
- Chip click expands inline to show the **MCP Tool Toggle** panel (see below).
- "Add" opens autocomplete palette.

#### MCP Tool Toggle (expanded view)
**Mockup:** `.superpowers/brainstorm/approved/mcp-tool-toggle.png`
**HTML:** `.superpowers/brainstorm/approved/mcp-tool-toggle-v1.html`

- Search bar to filter tools by name.
- Tools grouped by operation type (Read, Write, Admin) with "Enable all / Disable all" per group.
- Three-state per-tool: **allow** (green toggle), **ask** (orange toggle, requires confirmation), **deny** (off, name strikethrough).
- Status bar: "8 allowed · 1 ask · 9 denied" + preset buttons (Reset to defaults, Read-only preset).
- This is the per-agent MCP tool scoping — same MCP server, different tool sets per agent.

#### Subagents
- Same chip grid pattern. Picks from your other agents.
- "Add" opens autocomplete filtering your agent library.

#### Permissions
- Preset bar: read-only | ship-guarded | ship-standard | full-access | custom
- 4-card grid showing all permission dimensions:
  - **Tools**: allow/deny patterns (e.g. `Read, Grep, Bash(git *)`)
  - **Filesystem**: allow/deny paths (e.g. `apps/web/**`, deny `.env`)
  - **Commands**: allow/deny shell patterns
  - **Network**: allow hosts / policy
- "Edit" button opens full permissions editor (modal or expanded).
- Agent limits (max_cost, max_turns, require_confirmation) shown below.

#### Settings
- 2-column compact grid. Shows "(inherits from global defaults)" label.
- Model selector, default mode, extended thinking toggle, auto memory toggle.
- Override toggle per setting (inherits unless overridden).

#### Hooks
- Compact rows: trigger badge (PreToolUse, Stop, etc.) + command + provider dots.
- Provider dots show which providers the hook maps to (Claude = orange, Gemini = blue).
- "Add" opens hook creation with trigger dropdown + command input + matcher.

#### Rules
- File list: filename in monospace + first-line preview.
- "Edit" opens modal editor (markdown + always_apply toggle + glob patterns).
- "Add" creates new rule file.

### Acceptance Criteria

- [ ] Agent detail renders as single scrollable page with all sections
- [ ] Skills/MCP/Subagents use chip grid with add/remove
- [ ] Every "Add" action uses autocomplete palette (search library + registry)
- [ ] MCP chip click expands tool toggle panel inline
- [ ] Tool toggle supports three states: allow, ask, deny
- [ ] Tool toggle groups tools by operation type
- [ ] Permissions section shows preset bar + 4-dimension card grid
- [ ] Settings section shows inherited defaults with override toggles
- [ ] Hooks shown as compact rows with provider mapping dots
- [ ] Rules shown as file list with modal editor on click

## Page 2: Skills IDE

**Mockup:** `.superpowers/brainstorm/approved/skills-ide.png`
**HTML:** `.superpowers/brainstorm/approved/skills-ide-v1.html`

### Layout

Full-viewport three-panel IDE layout:
- **File explorer** (left, 240px) — skill directories as folders
- **Editor** (center, flex) — markdown editor with syntax highlighting
- **Preview panel** (right, 340px, collapsible) — metadata + output + usage

### File Explorer

Three sections:
1. **Project Skills** — folders for each skill in `.ship/agents/skills/`. Expandable to show `SKILL.md` and any assets. Active file highlighted with orange left border.
2. **Installed** — skills installed from registry. Package icon (blue). Shows version badge.
3. **Templates** — "New from template..." quick-start option.

Search bar at top filters all sections.
"+" button in Project Skills header creates new skill (folder + SKILL.md scaffold).

### Editor

- Tab bar with open files, unsaved indicator dot (orange), close buttons.
- Breadcrumb toolbar: `skills / ship-coordination / SKILL.md`
- Action buttons: Format, Preview, History.
- Line-numbered editor with syntax highlighting:
  - Frontmatter (`---` fences): keys in blue, values in green
  - Markdown: headings white/bold, text gray, code orange, links blue, lists muted

### Preview Panel

Three tabs:
1. **Metadata** — parsed frontmatter as key-value pairs. Allowed tools as tag chips. "Attached to Agents" showing which agents use this skill.
2. **Output** — compiled provider output (e.g. `.claude/skills/ship-coordination/SKILL.md`). Per-provider tabs.
3. **Used by** — list of agents that reference this skill.

### Acceptance Criteria

- [ ] Three-panel IDE layout fills viewport
- [ ] File explorer shows project skills as folders, installed skills as packages
- [ ] Editor has tab bar, line numbers, syntax highlighting for YAML frontmatter + markdown
- [ ] Preview panel shows metadata, compiled output, and usage
- [ ] New skill creation scaffolds folder + SKILL.md with frontmatter template
- [ ] Search filters all file explorer sections
- [ ] Preview panel is collapsible

## Page 3: Registry

**Mockup:** `.superpowers/brainstorm/approved/registry.png`
**HTML:** `.superpowers/brainstorm/approved/registry-v1.html`

### Layout

Centered content, max-width 960px.

### Sections

1. **Hero** — "Ship Registry" title. Search bar with inline filter chips: All | Skills | Agents | MCP.
2. **Stats** — package counts (skills, agents, MCP servers, total installs).
3. **Category tabs** — Trending | New | Most installed | Curated.
4. **Featured banner** — highlighted collection or package (e.g. Superpowers Skill Pack). "Install pack" CTA.
5. **Package grid** — 3-column cards organized by type:
   - Each card: icon (color-coded by type), name, author (@scope), description, install count, rating.
   - Agent cards additionally show composition: "5 skills · 2 MCP".
   - MCP cards show tool count: "18 tools".
   - Install button: "Install" or "Installed" (green state).

### Acceptance Criteria

- [ ] Search bar with filter chips (All/Skills/Agents/MCP) searches the registry API
- [ ] Category tabs filter results (trending, new, most installed, curated)
- [ ] Package cards show name, author, description, stats, install button
- [ ] Agent cards show skill/MCP composition count
- [ ] MCP cards show tool count
- [ ] Install action adds to user's library
- [ ] Featured banner promotes collections/packs
- [ ] Stats bar shows live counts from registry API

## Global UX Patterns

### Bottom Dock
- 3 navigation items: Agents, Skills, Registry
- Separator
- Compile button (orange) with inline provider toggles (small squares: C, G, O, Cu)
- Hover tooltips on all dock items
- Active state: orange background tint

### Autocomplete Palette
Every "Add" action (skills, MCP, subagents) opens a command-palette-style typeahead:
- Search your local library + registry results simultaneously
- Results grouped by source (Project | Installed | Registry)
- Keyboard navigable (arrow keys + enter)
- Shows metadata inline (version, author, description)

### No Emojis
All icons are SVG (Lucide icon set or similar). No emoji anywhere in the UI.

### Dark Theme
Primary background: `#0a0a0a`. Cards/panels: `#0d0d0d`–`#111`. Borders: `#1e1e1e`. Accent: `#f97316` (orange). Text hierarchy: `#fafafa` > `#e4e4e7` > `#a1a1aa` > `#71717a` > `#52525b` > `#3f3f46`.

### Global Defaults vs Agent Overrides
Settings (model, mode, hooks, env vars) inherit from global defaults. Per-agent overrides shown with "inherits from defaults" label. Override toggle per setting.

### Rules
Inline preview (filename + first line). Click opens modal editor with full markdown content, `always_apply` toggle, and glob patterns. Not a separate page.

## Compiler Coverage

The UI must cover 100% of what the compiler supports:

| Feature | Compiler | UI (this spec) |
|---|---|---|
| Skills (full metadata) | id, name, version, description, license, compatibility, allowed_tools, content | Skills IDE covers all fields via frontmatter editor |
| MCP Servers | command, args, env, timeout, server_type, per-provider fields | Agent detail MCP section + tool toggle |
| MCP per-provider fields | codex_enabled_tools, gemini_trust, gemini_include/exclude | MCP tool toggle (per-agent tool scoping) |
| Rules | filename, content, always_apply, globs, description | Agent detail rules section with modal editor |
| Permissions (tools) | allow, ask, deny | Agent detail permissions card |
| Permissions (filesystem) | allow, deny paths | Agent detail permissions card |
| Permissions (commands) | allow, deny patterns | Agent detail permissions card |
| Permissions (network) | policy, allow_hosts | Agent detail permissions card |
| Permissions (agent limits) | max_cost, max_turns, require_confirmation | Agent detail permissions section |
| Hooks | 6 triggers, per-provider mapping, matcher, command | Agent detail hooks section |
| Subagents / AgentProfiles | full nested profile | Agent detail subagents chip picker |
| Provider settings (Claude) | model, theme, memory, env, mode, attribution | Agent detail settings section |
| Provider settings (Gemini) | approval_mode, max_turns, sandbox, disable_yolo | Agent detail settings section |
| Provider settings (Codex) | model | Agent detail settings section |
| Provider settings (Cursor) | environment.json, hooks, cli permissions | Agent detail settings section |
| Plugins | install list, scope | Agent detail settings section |

## Page 4: Landing Page

**Mockup:** `.superpowers/brainstorm/approved/landing-hero.png` (+ features, cta)
**HTML:** `.superpowers/brainstorm/approved/landing-v1.html`

### Sections

1. **Nav** — Ship logo, Studio / Registry / Docs / Pricing links, Sign in, Get started
2. **Hero** — "One config. Every agent." + "The package manager for AI coding agents" + Open Studio / Install CLI CTAs
3. **Provider strip** — Claude Code, Gemini CLI, Codex CLI, Cursor with branded color dots
4. **Product showcase** — browser chrome frame containing mini agent detail view (skills, MCP with tool counts, permissions, output preview)
5. **Feature grid** (2x2 + 1 wide):
   - Package manager for agents
   - Write once, compile everywhere
   - Visual skill editor
   - Community registry
   - Tool scoping (wide, with live allow/ask/deny demo)
6. **How it works** — 3 steps: Define → Compile → Use
7. **Bottom CTA** — "Start shipping agents" + registry stats
8. **Footer** — minimal

### Acceptance Criteria

- [ ] Hero communicates the value prop in under 5 seconds
- [ ] Product showcase shows a realistic agent configuration
- [ ] Tool scoping demo shows allow/ask/deny three-state
- [ ] Provider strip shows all supported providers
- [ ] CTAs link to Studio and CLI install

## Page 5: Empty States

**Mockup:** `.superpowers/brainstorm/approved/empty-agents.png` (+ empty-skills)
**HTML:** `.superpowers/brainstorm/approved/empty-states-v1.html`

### Agents Empty State

- "No agents yet" with two CTAs: Create first agent + Browse registry
- **GitHub import banner** — persistent, prominent:
  - "Import from GitHub" title
  - Connect GitHub App button + paste repo URL input
  - "Already using CLAUDE.md or .cursor/rules? We'll convert it."
- CLI install banner with copy-able curl command
- Quick start cards: Starter agent / From registry / Import existing

### Skills IDE Empty State

- "No skills yet" with Create + Browse registry CTAs
- Compact GitHub import banner with repo URL input

### Acceptance Criteria

- [ ] GitHub import banner is always visible in empty states
- [ ] Repo URL paste triggers import flow
- [ ] CLI install command is copy-able
- [ ] Quick start cards lead to working flows

## Page 6: Settings / Account

**Mockup:** `.superpowers/brainstorm/approved/settings-account.png`
**HTML:** `.superpowers/brainstorm/approved/empty-states-v1.html` (screen 3)

### Sections

1. **Account** — avatar, name, email, plan badge, sign out
2. **GitHub** — connection status, auto-import toggle, create-PR toggle
3. **Global Defaults** — default provider, model, mode, thinking, memory, permission preset. These are what agents inherit unless overridden.
4. **Global Hooks** — hooks applied to all agents, overridable per-agent
5. **Environment Variables** — global KEY=VALUE pairs
6. **CLI** — installed version, install command
7. **Danger Zone** — delete all agents, delete account

### Acceptance Criteria

- [ ] Global defaults are editable and persist
- [ ] Agent detail settings show "inherits from defaults" when not overridden
- [ ] GitHub connection status reflects real OAuth state
- [ ] CLI version displays correctly

## Page 7: GitHub Import Flow

**Mockup:** `.superpowers/brainstorm/approved/github-import.png`
**HTML:** `.superpowers/brainstorm/approved/empty-states-v1.html` (screen 4)

### Layout

- Connected status with repo search/filter
- Repo list with three states:
  - **Detected** (orange highlight) — "CLAUDE.md + .mcp.json detected" → Import & PR button
  - **No config** — "No agent config detected" → Add Ship button (creates starter .ship/ PR)
  - **Already imported** (dimmed, green check) — "PR #42 merged"
- "Publish to registry" CTA at bottom

### Acceptance Criteria

- [ ] Repos with detected agent configs are highlighted
- [ ] Import & PR creates a real PR via GitHub API
- [ ] Add Ship creates a starter .ship/ config PR
- [ ] Already-imported repos show PR status

## Capability Matrix

What's real vs what needs backend work:

| UI Feature | Backend Status | Notes |
|---|---|---|
| Compile button | REAL | WASM client-side, works today |
| GitHub OAuth + import | REAL | Full flow working |
| Registry search | REAL | Public API, paginated |
| Registry publish | REAL | Fetches ship.toml, indexes skills |
| Library CRUD + sync | REAL | Server-backed, debounced |
| Agent profiles | MOCK | localStorage only, needs server sync |
| Registry package details | MOCK | Falls back to mock data |
| MCP tool toggle per-agent | MISSING | Needs UI state + compiler integration |
| Skill install from registry | MISSING | Only via library JSON |
| Global defaults persistence | MISSING | Needs settings API |
| Autocomplete palette | MISSING | Needs Command component + data sources |

## Design Tokens

Use existing project tokens, not mockup colors:

- Colors: OKLch tokens in `apps/web/src/styles.css` (--primary, --foreground, etc.)
- Typography: Syne (display) + DM Sans (body) via @fontsource-variable
- Icons: Lucide React
- Components: @ship/primitives (39 components, CVA variants)
- Types: `packages/ui/src/generated.ts` (38 Specta-generated types from Rust)
- Tailwind v4 with `@tailwindcss/vite`

## File References

All approved mockups:
- `.superpowers/brainstorm/approved/agent-detail.png` + `agent-detail-v1.html`
- `.superpowers/brainstorm/approved/skills-ide.png` + `skills-ide-v1.html`
- `.superpowers/brainstorm/approved/registry.png` + `registry-v1.html`
- `.superpowers/brainstorm/approved/mcp-tool-toggle.png` + `mcp-tool-toggle-v1.html`
- `.superpowers/brainstorm/approved/landing-hero.png` + `landing-v1.html`
- `.superpowers/brainstorm/approved/empty-agents.png` + `empty-states-v1.html`
- `.superpowers/brainstorm/approved/settings-account.png`
- `.superpowers/brainstorm/approved/github-import.png`
