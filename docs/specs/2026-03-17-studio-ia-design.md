# Ship Studio — Information Architecture & Component Design
**Date:** 2026-03-17
**Status:** In Progress
**Authors:** Commander + human

---

## Product Framing

**Ship Studio is a platform launchpad — "the best settings page ever."**

- Studio is the config surface: profiles, skills, MCP servers, workflow wiring, export
- It lives on web AND desktop (Tauri)
- It is not Claude Code commander — it is provider-agnostic agent configuration
- Eventually Studio nests inside a Project dashboard (top-level nav item), but today Studio IS the app
- The revelation moment: import your messy scattered agent config → see it consolidated → wire a workflow → realize you've been doing this wrong

---

## Top-Level Navigation

```
[ Ship logo ]  [ Studio (active) ]  [ Projects (future) ]       [ Sign in ]
```

- **Studio** — config surface (current focus)
- **Projects** — future: docs, specs, ADRs, jobs per project. Studio becomes one tab inside a project.
- Projects is NOT a card on the Studio dashboard — it is a peer nav item

---

## Studio Dashboard — Card Layout (C)

Content-first home screen. Minimal top bar. Cards navigate into each section.
No persistent sidebar. Navigate INTO sections, not alongside them.

### Cards

| Card | Color accent | Empty state | Populated state |
|------|-------------|-------------|-----------------|
| **Profiles** | Purple `#7c3aed` | "Create first →" | Count + profile list preview |
| **Workflow** | Amber `#f59e0b` | Locked (greyed) until profiles exist | Active jobs count, role/routing summary |
| **Skills** | Green | Empty user collection | User/team collection — attached to profiles |
| **MCP Servers** | Blue | Empty user collection | User/team collection — attached to profiles |
| **Export** | Grey | Locked until profiles exist | Shows available paths |
| **Presets** | Neutral | Empty | "Browse →" |

### Workflow card lock
Workflow is intentionally locked until at least one profile exists.
Reason: canvas nodes ARE profiles — no profiles = empty canvas = confusing.

---

## First-Run Experience

### Import Project Banner (first-time users only, dismissible)
Shown above cards on first visit. Hidden permanently after dismiss or after first profile created.

**Headline:** "Import an existing project"
**Body:** Ship reads your repo — CLAUDE.md, .mcp.json, GEMINI.md, AGENTS.md, .cursor/, .codex/ and more — and consolidates everything into `.ship/`

**Three import paths (presented as one surface):**

| Path | Friction | Auth required | Notes |
|------|----------|---------------|-------|
| **GitHub URL** | Lowest — paste a URL | None (public repos) | Default option, shown first |
| **GitHub App** | Medium — authorize once | GitHub OAuth | Unlocks private repos, org repos |
| **Local folder** | CLI required | None | Best for power users already on CLI |

Not too much — these are three natural entry points for three user types. Present as tabs or a segmented picker inside one "Import project" modal.

**"Start blank" CTA** — secondary action, opens preset picker:
- Web Developer (React · TypeScript · Tailwind)
- Rust Engineer (CLI · Systems · Cargo)
- Full Stack (API · DB · Frontend)
- Ship Commander (Multi-agent · Orchestration)
- Blank Profile
- Browse all presets →

### Tutorial Stepper (first-time, dismissible)
Linear progress indicator showing the golden path:
`1. Create profile → 2. Add skills + MCP → 3. Wire workflow → 4. Export`

Step 1 is always active on first load. Steps unlock progressively.
Dismissed permanently via "dismiss ×". Never shown to returning users.

---

## Export Surface

Three distinct user states — detected automatically, not tabs. Detect CLI presence before rendering.

### State 1: CLI installed + Ship account
Changes auto-sync when saved. Show sync status per profile (synced / pending), last sync timestamp.
No primary CTA — the work is already done. Escape hatches (GitHub deploy, .zip) are quiet secondary actions.

### State 2: CLI installed, no account
Generate a one-time config link. User runs in terminal:
```
ship use @username/profile-name-abc123
```
UI: generated command with one-click copy. Link expires in 7 days, one-time use.
Upsell: "Auto-sync with a Ship account — no more copy-paste."

### State 3: No CLI
Golden path: install Ship CLI (Rust binary, cURL install):
```
curl -fsSL https://getship.dev/install | sh
```
Then `ship login` to link account. Show as a numbered 3-step checklist.
Detect carefully — do not show install prompt if CLI is already present.
Escape hatch: `.zip` download (all provider files) + individual file downloads (CLAUDE.md only, etc.).

### Additional paths (all states)
- **GitHub App deploy** — if GitHub App connected, commit provider files directly to repo
- **.zip download** — always available as escape hatch in all three states

---

## Profile Editor

### Tab structure
Three tabs: **Overview · Providers · Permissions**

### Overview tab
The composer — assembles the profile from parts.

| Field | Type | Notes |
|-------|------|-------|
| **Name** | Text input | Profile identifier |
| **Persona** | Short text | 1–3 sentence agent framing, profile-specific |
| **Rules** | Inline list | Always-loaded global context (tabs vs spaces, conventions). NOT progressive. NOT a skill. Bullet-style inline editor. |
| **Skills** | Chip selector | Progressive context from library. Add → opens shared skill/MCP discovery modal. |
| **MCP Servers** | Chip selector | Same pattern as skills. |
| **Default provider** | Pill selector | Claude / Gemini / Codex / Cursor. Per-provider config → Providers tab. |

**Rules vs Skills distinction:**
- Rules load immediately and always — think global conventions, "use tabs not spaces"
- Skills load progressively (on-demand context injection) — think TDD workflow, code-review checklist
- Rules are inline in the profile. Skills are library items composed into the profile.

### Providers tab
Sub-navigation: **Claude · Gemini · Codex · Cursor**

Each provider gets a full config page targeting 100% compatibility with that tool's settings schema.
Global defaults are set at the project level. The profile page shows overrides only — fields show "global default · override for this profile" hint when not customized.

**Claude page surfaces (priority order):**
- Model (dropdown)
- defaultMode (plan / acceptEdits / auto / dontAsk / bypassPermissions)
- Thinking (toggle)
- Hooks (PostToolUse, PreToolUse — add/remove inline)
- Env vars (key-value list)
- Memory (autoMemoryEnabled toggle)
- Long tail (~40+ settings) collapsed into expandable rows

Provider tab UI is intentionally placeholder until Specta type generation is automated (Rust → `packages/ui/src/bindings.ts`). Full form generation blocked on typed schemas.

### Permissions tab
Standalone component — not inline on overview.

- **Presets:** Strict / Default / Permissive / Custom (pill selector)
- **Allow rules:** chip list + live autocomplete input (suggests `Bash(pnpm:*)`, `Edit(apps/web/**)`, etc.)
- **Deny rules:** chip list + add input
- Selecting a preset populates allow/deny chips; switching to Custom preserves edits

---

## Existing Code Mapping

| Current route/component | Maps to | Notes |
|------------------------|---------|-------|
| `routes/studio.tsx` (1005 lines) | Profiles card + Profile editor | Needs refactor — extract hooks, split to ≤300 lines |
| `routes/canvas.tsx` + `components/canvas/` | Workflow card + Workflow editor | Rename `/canvas` → `/workflow` |
| `routes/compiler.tsx` | Dead — 6 lines | Delete or repurpose as live preview panel |
| `components/ImportDialog.tsx` | Import project modal | Extend with GitHub URL + local folder paths |
| `routes/api/github/import.ts` | GitHub URL import backend | Already exists |
| `routes/api/github/oauth.ts` | GitHub App auth | Already exists |

---

## State Management Architecture

- **TanStack Query** for all server state: profiles from API, auth session, registry data
- **Custom hooks per feature**: `useLibrary`, `useProfile`, `useCompileState`
- **useReducer** only if a hook has 3+ interdependent state transitions
- **No global store** — do not add Zustand unless cross-route sharing becomes unavoidable
- **Components are dumb** — no business logic, no fetch calls inside JSX

---

## Skills

Skills is a **creation environment**, not a registry. This is the differentiator.

### Two modes

**Browse/Manage (all users)**
- Layout: List + detail panel (A)
- Left: folder tree — each skill is a folder (folder name → SKILL.md + optional nested scripts)
- Right panel: preview content, attached profiles, source info
- "Attached to" pill list with quick attach/detach

**Skill Editor (PRO — badged)**
- Full IDE view: folder tree + CodeMirror editor
- AI-assisted editing (suggest improvements, fill gaps)
- Inline security audit — flags dangerous commands, permission escalation
- Create new skill: name → scaffold SKILL.md + folder structure
- This makes Ship potentially better than any external registry — you build skills here

### Skill structure on disk
```
.ship/skills/
  tdd/
    SKILL.md          # main content
    scripts/          # optional supporting scripts
  code-review/
    SKILL.md
```

### Discovery / Add Skills
Shared modal with MCP discovery (same component, different data source).
- Curated picks (Ship-endorsed)
- Paste GitHub URL → imports SKILL.md from repo
- Browse external (linked out, clearly labelled external)
- Search your existing collection

### PRO badge
Skill editor, AI assist, security audit = PRO tier.
Browse + attach existing skills = free.

---

## MCP Servers

User/team collection. Already have a decent live view.
Discovery shares the add modal with Skills.

**Configuration fields per server:** command, args, env vars (masked), server_type, scope, timeout.
**Attached to:** same pill pattern as skills.

**MCP discovery:** API-driven (modelcontextprotocol.io + Ship curated layer). Icons from registry JSON where available, normalized at the Ship layer. Community can PR additions to Ship curated list; maintainer approves.

---

## Profiles

### Terminology
- Named "Profile" in UI for now. Internal codename "Blueprint" noted for future rename consideration.
- Community-provided profiles = **Templates** (not Presets). Presets card renamed to "Templates".
- Creating a new profile: blank OR "Start from template".

### Visual identity
- Each profile gets a **color accent** (visual differentiation, not personality) and a **tech icon**.
- Icons sourced from **Simple Icons** (3000+ SVG brand icons, searchable) + Devicons for colored variants.
- Icon picker: search by tech name (React, Rust, Git, TypeScript...) + accent color selector.
- No anthropomorphizing. Icon represents function/stack, not a character. No banners, no bios, no social framing.

### Profile list
Card grid. Each card: icon tile (accent color background), profile name, provider + model, skill count, MCP count. Default badge on active profile.

### Profile editor — tab structure
Three tabs: **Overview · Providers · Permissions**

#### Overview tab
| Field | Notes |
|-------|-------|
| Name | Profile identifier |
| Persona | 1–3 sentence agent framing, compiled into provider system prompt |
| Rules | Always-loaded global context (tabs vs spaces, naming conventions). NOT progressive. NOT a skill. Inline bullet list. |
| Skills | Chip selector — progressive context from library. "+ Add" opens shared discovery modal. |
| MCP Servers | Chip selector — same pattern. |
| Default provider | Pill selector. Full per-provider config → Providers tab. |

**Rules vs Skills distinction:**
- Rules load immediately on every session — global conventions ("use tabs not spaces")
- Skills load progressively (on-demand context injection) — TDD workflow, code-review checklist
- Rules are inline in the profile. Skills are library items composed into the profile.

#### Providers tab
Sub-nav: **Claude · Gemini · Codex · Cursor**

Each provider = full config page targeting 100% compatibility. Global defaults shown as baseline; profile page shows per-profile overrides only.

Claude page priority: model, defaultMode (plan/acceptEdits/auto/dontAsk), thinking toggle, hooks, env vars, memory toggle. Long tail (~40+ settings) collapsed into expandable rows.

**Note:** Provider tab UI is placeholder until Specta type generation is automated (Rust → `packages/ui/src/bindings.ts`). Full form generation requires typed schemas.

#### Permissions tab
Standalone component:
- Presets: Strict / Default / Permissive / Custom
- Allow rules: chip list + live autocomplete (suggests `Bash(pnpm:*)`, `Edit(apps/web/**)`, etc.)
- Deny rules: chip list + add input
- Selecting a preset populates chips; Custom preserves manual edits

### Discovery modal (shared — Skills + MCP)
Same component, different data source per context.
- Search bar: text search of your collection + paste GitHub URL
- Your collection first (with "added ✓" state)
- Curated picks (Ship-endorsed, maintained in Ship's git repo)
- "Browse external →" clearly labelled as external
- Icons shown where available (Simple Icons for skills, registry icons for MCP)

### Templates (formerly Presets)
Filter chips (All / Frontend / Rust / Multi-agent / Full Stack / ...) + list rows with "Use" button. "Use" creates a new profile from the template. New profile creation flow: blank OR pick template.

---

## Workflow / Canvas

### Platform boundary
The **platform** provides: config compiler, Docs API (Document + DocumentVersion + Event), sessions, workspaces, lifecycle hooks, analytics, sync (event replay). These are stable primitives.

The **workflow layer** is a guest on top of platform primitives. A WorkflowDefinition defines doc types, editing behaviors, roles, routing — but does not touch platform internals.

### Can users mix workflows?
**Answer: one primary workflow per project, additional workflows as skill-packs only.**

- One workflow defines the doc shape (doc types, editing modes, statuses)
- Other workflows (superpowers, gstack, etc.) can be loaded as skill bundles — they bring skills/tools but no doc types
- No doc type collision possible
- UX: most users never pick a workflow. Shipflow is the default, invisible. Power users can swap the primary.

### Workflow shape — required fields
A WorkflowDefinition must specify:
1. **Doc types** — name, update_mode (mutate|append), valid statuses, relationships between types
2. **Roles** — human | agent-preset, what each role can create/edit/gate
3. **Routing rules** — which role handles which work type (message bus)
4. **Gate criteria** — what "done" means per doc/work type
5. **Context injection** — what loads at session start per role

### Canvas UI
The Workflow card in Studio opens the canvas. Canvas = visual authoring of the WorkflowDefinition.
- **Nodes** = roles (profile cards for agents, human icon for humans)
- **Edges** = routing rules (message bus connections between roles)
- **Gate annotations** on edges = acceptance criteria
- **Context annotations** on nodes = what loads at session start for that role
- Workflow card locked in dashboard until at least one profile exists (nodes ARE profiles — empty canvas is meaningless)

---

## Visual Design Reference

Brainstorm mockups (HTML, viewable in browser) saved to `docs/brainstorm/`:
- `dashboard-populated.html` — returning user dashboard
- `dashboard-home.html` — first-run dashboard with import banner
- `skill-editor.html` — skills folder tree + PRO editor panel
- `profile-composer.html` — profile overview + providers tab side by side
- `export-states.html` — three export user states
- `remaining-ui.html` — profile list, discovery modal, presets, empty states
- `blueprints.html` — profile card grid with tech icons (Simple Icons)
- `detail-panel.html` — skills/MCP detail panel variants

Server the directory locally to view: `npx serve docs/brainstorm`

---

## Type Generation

Currently: TypeScript types in `packages/ui/src/types.ts` are **manually** maintained mirrors of Rust types.
Specta is wired into `crates/core/runtime` and `crates/modules/project` but no `bindings.ts` is generated into `apps/web`.
**Decision needed:** automate Specta export → `packages/ui/src/bindings.ts` as part of build.
Runtime types (workspace, session, job, capability) have no frontend representation yet — needed once Projects lands.

---

## Open Decisions

- [ ] Export tab name: "Export" vs "Deploy" vs "Publish" vs "Compile"
- [ ] Import modal: tabs vs segmented picker vs sequential flow for the 3 paths
- [ ] Tutorial stepper: pill steps (current) vs checklist vs progress bar
- [ ] Automate Specta type generation into web package
- [ ] `/canvas` rename to `/workflow`
- [ ] `routes/compiler.tsx` — delete or repurpose as live preview

---

## Not In Scope (Studio v1)

- Projects dashboard (future top-level nav item)
- Docs, ADRs, specs per project (Projects feature)
- Real-time collaboration
- Marketplace / paid tiers
