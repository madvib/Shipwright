# Ship Platform Architecture

## Context Firewall — Read Before Touching Code

> **Status**: v0.2 — Current Intent
> **Rule**: If something you're building isn't in the Platform Layer, it belongs in a Workflow Definition.
> **Updated**: 2026-03-19

## Canonical URLs — Do Not Invent These

| Service | URL |
|---------|-----|
| Ship Studio (web app) | https://getship.dev |
| Staging | https://staging.getship.dev |

All hardcoded domain references in source, docs, and config must use `getship.dev`. Never use `ship-studio.com`, `shipstudio.com`, or any other variant.

---

## First Principles

### 1. The Compiler Is the Moat
Ship's core value is a schema-stable compiler that takes a declarative preset and emits provider-specific configuration. The compiler must be trustworthy before anything else matters. Every architectural decision should protect compiler stability.

### 2. The `.ship` File Is the Unit of Truth
Agent provider configs (`.claude`, `.cursor/rules`, `.agents`) are build artifacts. They are derived, ephemeral, and gitignored. The `.ship` file is the source. This inversion — treating provider configs as outputs not inputs — is what makes Ship composable and portable.

### 3. Presets Are Snapshots, Workspaces Are Identity
A preset is a named, versioned configuration snapshot: skills, MCPs, permissions, rules, hooks. It has no runtime behaviour. A workspace is the living identity — an ID, an optional path, a kind, a session history. Workspaces outlive presets. You can swap presets mid-session. The workspace is permanent, the preset is replaceable.

### 4. Sessions Are the Heartbeat
Workspaces contain sessions. Sessions have a lifecycle: start, run, end. Everything observable happens inside a session. Hooks fire on session events. Audit trails are session logs. Drift is measured between sessions. The session is where the platform touches reality.

### 5. Events Are the Primitive, Not Logs
Every state change is an event. Events are immutable, append-only, timestamped. The event log is the database. Sync is event replay from a cursor. Audit trails are event queries. Drift analysis compares event snapshots. If something happened and there's no event, it didn't happen as far as the platform is concerned.

### 6. Workflows Are Guests, Not Hosts
The platform does not know what a "feature" or "release" or "issue" is. These are workflow concepts. A workflow definition is a guest that lives on top of the platform's document and event primitives. Shipflow is one workflow — the reference implementation. superpowers, gstack, and any MIT-licensed markdown-based workflow is a candidate for porting. The platform provides the substrate; workflows provide the semantics. Ship's value compounds as more workflows port to it.

### 7. Local First, Platform Second
The CLI and local SQLite database are the runtime. The platform (registry, analytics, sync, AI features) is additive. A user with no account and no internet connection should be able to `ship init`, `ship use @shipflow/default`, and have a fully functional local agent configuration system. Platform features enhance; they do not gate.

---

## The Two Layers

```
┌─────────────────────────────────────────────────────┐
│                  WORKFLOW LAYER                      │
│  shipflow, gstack, superpowers, user-defined         │
│                                                      │
│  Document types: Feature, Release, Issue, Target...  │
│  Update behaviors: declarative drift / imperative    │
│  Relationships: feature → release, issue → feature  │
│  Lifecycle rules: what "done" means per type        │
└─────────────────────────────────────────────────────┘
         ↓ composes on top of ↓
┌─────────────────────────────────────────────────────┐
│                  PLATFORM LAYER                      │
│                                                      │
│  Workspace    — identity, path, kind, status        │
│  Preset       — config snapshot (skills/MCP/rules)  │
│  Session      — lifecycle, hooks, audit             │
│  Document     — typed content with version history  │
│  Event        — immutable append-only log           │
│  Skill        — callable instruction primitive      │
│  MCP          — tool server configuration           │
│  Permission   — tool/fs/network/agent limits        │
│  Workflow     — definition schema (guest layer)     │
└─────────────────────────────────────────────────────┘
```

---

## Platform Layer — Canonical Types

These types are stable platform contracts. Agents working on Ship core should only reference these.

### Workspace
```
id (nanoid), path?, kind (declarative|imperative|service), status (active|archived),
active_preset, project_id, is_worktree, worktree_path,
resolved_at, last_activated_at, context_hash, config_generation
```
The workspace is keyed by **UUID (`id`), not path or branch**. Path is optional — service workspaces have no path, remote workspaces may have no local path. Branch is a property of a git-bound workspace, not the identity. The three kinds map to fundamentally different work patterns:

- **declarative** — long-running branch, agent edits in place toward a defined end state, drift is meaningful
- **imperative** — ephemeral branch, append-style commits, PR lifecycle, no drift concept
- **service** — persistent process, no git lifecycle, event-triggered, no "done" state

*Note: `branch` field remains for git-bound workspaces but is NOT the primary key. `feature_id` and `target_id` are workflow-layer concerns — they do not belong on the platform Workspace type.*

### Preset (formerly Mode)
```
id, name, description, version,
skills[], mcp_servers[], rules[], hooks[], permissions,
target_providers[], active_tools[]
```
A preset is a reusable, publishable configuration unit. `active_tools[]` controls which skills are loaded — this is not cosmetic. It is how workflow phases get distinct agent capabilities. Renaming from `ModeConfig` → `PresetConfig` is intentional. "Mode" implies runtime behaviour. "Preset" correctly implies a saved configuration snapshot.

### Session
```
id, workspace_id, status (active|ended),
started_at, ended_at, goal, summary,
preset_id, primary_provider,
config_generation_at_start, stale_context
```
Sessions belong to workspaces. A workspace can have many sessions over its lifetime. Only one session is active at a time per workspace. Sessions do not own documents — they reference them via events.

### Document
```
id, project_id, type (workflow-defined), title,
status (workflow-defined), update_mode (mutate|append),
created_at, updated_at
```
Documents are typed content containers. The platform defines the container; workflows define the types and valid statuses. A document's meaning is determined by its workflow context, not by the platform.

### DocumentVersion
```
id, document_id, content (markdown), created_at, session_id?
```
Always append-only, even for declarative (mutate-mode) documents. Drift analysis is a diff between `versions[-1]` and `versions[-2]`. The document feels mutable; the history is immutable.

### Event
```
seq, timestamp, actor, entity, action, subject, payload_json,
workspace_id?, session_id?, document_id?
```
The unified hook surface. Every platform state change emits an event. Hooks are subscriptions to event types. Sync is event replay. This is the single most important primitive for platform extensibility.

### WorkflowDefinition
```
id, name, version,
document_types: WorkflowDocumentType[],
default_update_behavior (declarative|imperative)
```

### WorkflowDocumentType
```
id, singular_name, plural_name,
update_mode (mutate|append),
statuses: string[],
relationships: WorkflowRelationship[]
```
This is how "Feature", "Release", "Issue" enter the system — as workflow-defined document types, not platform primitives.

---

## The Moat

### 1. Multi-provider compilation
One `.ship/` compiles to Claude, Gemini, Cursor, Codex, Copilot simultaneously. No one else does this. The switching cost compounds as users add providers — leaving Ship means rebuilding config for every provider from scratch.

### 2. Package manager for agents
`ship use @org/preset` is npm for agent config. Lockfile, versioning, registry, dependency resolution. Once teams treat presets the way they treat npm packages — checked into source, reproducible, team-shared — the switching cost is indistinguishable from removing a package manager. This is the revenue model: free for individuals, paid for teams and private registries.

**Registry model (git-native, v0.1):** Packages are published from GitHub repos that contain a `.ship/ship.toml` with a `[module]` section declaring the package id, name, and version. Skills are indexed from the repo contents (`.ship/agents/skills/`). The registry API (getship.dev) indexes these repos; `ship use <id>` fetches and installs from them. R2 stores immutable content blobs (keyed by `skills/:id/:version/SKILL.md`). D1 stores queryable metadata (id, name, description, tags, author, version, r2_key). Durable Objects (Rivet actors) manage per-user and per-org state.

### 3. Project data ownership
Once `.ship/` is in the repo and `ship use` is in the dev workflow, Ship owns: active preset, session history, workspace state, team usage patterns. This is the Linear play — Linear owns issue data and became infrastructure. Ship owns agent config data and becomes infrastructure. Analytics, workflow tools, and AI features compound on top of this data. Tools that own project data don't get replaced; they get extended.

### Distribution strategy
The goal is `.ship/` in every repo on GitHub. Distribution is not a marketplace problem:
- **GitHub PR flow** — Studio extracts existing config from any public repo URL, one-click PR adds `.ship/`. Every merged PR is a distribution event.
- **agentskills.io compliance** — Ship's compiled output (`.agents/skills/`) is the open standard, read natively by every provider. Being in GitHub in the right format IS being in every marketplace — no submission required.
- **Viral spread** — devs who clone repos with `.ship/` encounter it. The PR description is the onboarding copy.

### agentskills.io — a standard Ship participates in, not owns
Launched Dec 2025 by Anthropic, adopted by 26+ platforms. Format: directory with `SKILL.md` (YAML frontmatter + markdown) + optional `scripts/`, `references/`, `assets/`. `.agents/skills/` is the canonical read location across providers. Ship writes there as compiler output AND reads from there on `ship import`. Ship is a first-class participant in this standard; the standard enables Ship's portability story.

### Provider plugin formats
Each provider has a native plugin format. Ship should publish a plugin for each that bundles: `ship mcp` (MCP server), branch-switch hook (auto-runs `ship use`), and ship-workflow skill. No slash commands — MCP + hooks are the high-value surface.

| Provider | Plugin manifest | Ships |
|---|---|---|
| Claude Code | `~/.claude/plugins/<name>/` | skills, MCP, hooks |
| Cursor | `.cursor-plugin/plugin.json` | skills, rules, MCP, hooks, commands |
| Gemini CLI | `gemini-extension.json` | skills, MCP, hooks, commands, context |
| Codex | TBD | — |

The plugin is how users get Ship behavior inside their existing provider UX. Build after CLI is stable.

---

## Skill Filtering — Platform Differentiator

Presets control which skills are active via `active_tools[]`. This is architecturally significant:

- Skills are not globally loaded. An agent only sees the skills its preset enables.
- Phases, roles, and capabilities are all just presets — a discovery preset loads brainstorm + ADR skills, an implementation preset loads coding + git skills, a security preset loads security skills.
- A workflow can decompose into arbitrarily many fine-grained skills without polluting any single agent's context.
- Skills are ONE delivery channel. Ship also delivers context via compiled CLAUDE.md/AGENTS.md output, rules, init hooks, and MCP tools. Workflows use all of these, not just skills.

The key difference from superpowers/gstack: their skills are globally loaded on every agent. Ship's preset-filtered loading means an agent working on implementation genuinely doesn't see discovery-phase instructions, and vice versa.

---

## Workflow Ecosystem

Ship is the substrate. Workflows are guests. This distinction is load-bearing.

### What the platform provides that existing workflows lack

Existing workflows (superpowers, gstack, and others like them) are essentially **well-organized markdown with activation logic**. They are valuable and proven. What they lack:

| Gap | Ship primitive |
|---|---|
| No persistent workspace state | `Workspace` (UUID-keyed, survives sessions) |
| No session audit trail | `Session` + event log |
| No document versioning | `DocumentVersion` (append-only history) |
| No drift detection | diff(versions[-1], versions[-2]) |
| No multi-provider compilation | compiler → CLAUDE.md, AGENTS.md, .cursor/rules |
| No marketplace / analytics | platform registry + event stream |
| Primitive hooks | Ship event system (session.start/end, git.commit, etc.) |

### Two integration paths

**Full port** — workflow definition file + skill packages in agentskills.io format. Workflow gets all platform primitives. Requires a one-time migration of skill files (mostly renaming and adding frontmatter). MIT-licensed markdown workflows like superpowers port in hours, not days.

**MCP integration** — workflow stays as-is, agents call Ship MCP tools to become platform-aware. `ship.start_session()`, `ship.log_progress()`, `ship.create_document()`, `ship.load_skill()`. Zero migration required. gstack agents could query workspace state and session context via MCP without changing their skill files at all.

### The marketplace play

Every ported workflow becomes a registry package. Ship provides:
- Discovery (`ship install @superpowers/default`, `ship install @gstack/dev`)
- Analytics — which skills activate most, session outcomes per workflow, team-level usage
- Private registry for teams (paid tier)
- Community ratings + fork/extend model

Workflows that port get distribution. Ship gets content. The ecosystem compounds.

### Dogfooding consideration

Before building Shipflow from scratch, we should run on a ported superpowers or gstack workflow. This validates the platform primitives against a real proven workflow before we invest in building our own opinionated loop. Shipflow emerges from that experience with sharper edges.

---

## Workflow Layer — Shipflow

Shipflow is Ship's reference workflow implementation and the primary dogfood target. It is architecturally a guest on the platform — demonstrating what the platform can do, not defining the platform.

**What makes shipflow distinct from superpowers/gstack:**
- Declarative workspace kind — long-running branches with drift awareness (superpowers only does imperative)
- An idea queue that routes upstream of planning (superpowers starts at brainstorm, shipflow starts earlier)
- All three workspace kinds (service for coordination, imperative for tasks, declarative for sustained work)
- Phase-aware preset switching — discovery, planning, implementation, and integration each get distinct agent contexts
- The full platform stack: sessions, documents, events, hooks, drift

**Phases are lifecycle hooks, not roles.** The superpowers brainstorm/plan/execute loop maps directly onto shipflow phases. Role-based skills (developer, QA, CEO lens, security) compose on top of the phase structure — a discovery phase might load both a strategic lens and a technical feasibility lens simultaneously. These are orthogonal axes.

### The Loop

```
┌──────────────────────────────────────────────────────┐
│  DISCOVERY                                           │
│  Brainstorm → decisions → ADRs → documented intent  │
│  Workspace: service (persistent idea router)         │
│  Input: raw ideas, questions, constraints            │
│  Output: Goal + documented decisions (ADRs)          │
│  Idea queue routes output → Planning                 │
└──────────────────────────────────────────────────────┘
                        ↓
┌──────────────────────────────────────────────────────┐
│  PLANNING  (pre-session)                             │
│  Refine Goal → acceptance criteria → workspace kind  │
│  Create/activate workspace → select preset           │
│  Output: workspace ready, goal scoped                │
└──────────────────────────────────────────────────────┘
                        ↓
┌──────────────────────────────────────────────────────┐
│  IMPLEMENTATION  (session loop)                      │
│  ship session start --goal "..."                     │
│  → sub-agents run in parallel imperative workspaces  │
│  → implement → log progress → commit/checkpoint      │
│  Hooks: session.start, git.commit, session.progress  │
│  Output: code + auditable session log                │
└──────────────────────────────────────────────────────┘
                        ↓
┌──────────────────────────────────────────────────────┐
│  INTEGRATION  (post-session)                         │
│  ship session end --summary "..."                    │
│  → update documents → PR/merge (imperative)          │
│     OR checkpoint + continue (declarative)           │
│  → route next item from idea queue                   │
└──────────────────────────────────────────────────────┘
```

### Shipflow document types (workflow-layer, not platform)
- `Vision` — singular, mutate, no statuses. North star.
- `Target` — mutate, statuses: `draft|active|complete`. Milestone.
- `Feature` — mutate, statuses: `planned|active|done`. Long-lived, 1:1 with declarative workspace.
- `ADR` — append (supersede creates new), statuses: `proposed|accepted|rejected|superseded`.
- `Note` — append, no statuses. Ephemeral capture scoped to session.

### v0.1 scope
The loop above, sessions, documents, ADRs, idea queue, basic hooks. No embedded test automation, no LSP references, no drift detection — those are v0.2+ and require DocumentVersion to be production-ready.

**What agents need to know:** Shipflow types are not platform types. Feature, Release, Issue, Spec, Vision live in the shipflow workflow package. Do not add them to platform code.

---

## Naming Conventions

| Old Name | Current Name | Reason |
|---|---|---|
| `ModeConfig` | `PresetConfig` | Preset implies saved config, not runtime state |
| `active_mode` | `active_preset` | Consistency |
| `mode_id` | `preset_id` | Consistency |
| `listModesCmd` | `listPresetsCmd` | Consistency |
| `setActiveModeCmd` | `setActivePresetCmd` | Consistency |
| `ShipWorkspaceKind::Feature` | `Declarative` | Removes collision with workflow-layer Feature document |
| `ShipWorkspaceKind::Patch` | `Imperative` | Clearer intent |
| `Issue` | *(workflow-layer only)* | Not a platform primitive |
| `FeatureDocument` | *(workflow-layer only)* | Not a platform primitive |
| `ReleaseDocument` | *(workflow-layer only)* | Not a platform primitive |
| `VisionDocument` | *(workflow-layer only)* | Not a platform primitive |
| `SpecEntry` | *(workflow-layer only)* | Not a platform primitive |

---

## ProjectConfig — Intended Separation

`ProjectConfig` currently conflates platform config, preset config, and agent config. The intended separation:

**`ship.toml` (ProjectConfig) — platform config only:**
```toml
id = "..."
name = "ship"
description = "..."
providers = ["claude", "gemini"]
active_preset = "default"
workflow = "shipflow"        # references a WorkflowDefinition
```

**Preset definitions** — separate, referenced by id, live in `.ship/presets/`

**Agent layer config** — already broken out correctly as `AgentLayerConfig`, keep as-is

---

## Archive Boundary

```
/src          ← platform code. Agents work here.
/archive/v0   ← prior intent. Reference only. Never modify.
```

Types that are **prior intent only** (do not port to platform layer):
- `Issue`, `IssueEntry`, `IssueStatus`, `IssueMetadata`
- `FeatureDocument`, `FeatureInfo`, `FeatureCriterionItem`, `FeatureTodoItem`
- `ReleaseDocument`, `ReleaseInfo`
- `SpecEntry`, `Spec`, `SpecMetadata`, `SpecStatus`
- `VisionDocument`

These will re-emerge as workflow-layer types inside the shipflow workflow package. Not deleted — relocated.

Types that **are** load-bearing and should be ported (with renames noted above):
- All Workspace types ✓ (remove `feature_id`, `target_id`, `updated_feature_ids`)
- All Session types ✓
- ModeConfig → PresetConfig ✓
- HookConfig, HookTrigger ✓
- Permissions and sub-types ✓
- McpServerConfig, McpProbeReport, McpValidationReport ✓
- Skill, SkillSource, CatalogEntry ✓
- EventRecord, EventEntity, EventAction ✓
- AgentConfig, AgentDiscoveryCache ✓
- ProjectConfig (slimmed per above) ✓
- ProviderInfo, ModelInfo ✓
- NoteDocument, NoteInfo ✓ (platform-layer, scoped by project_id)
- AdrEntry, ADR, AdrMetadata ✓ (platform-layer documents, scoped by project_id)

---

## Agent Rules

```
- This document is current intent. archive/v0 is prior intent.
- Platform layer lives in /src. Do not reference /archive.
- Issue, Feature, Release, Spec, Vision are workflow-layer types (shipflow).
  Do not add them to platform code.
- Workspace identity is UUID (nanoid), not branch, not path.
  branch is a property of git-bound workspaces only.
- Workspace kinds are: declarative, imperative, service.
  feature and patch are prior names — do not use them.
- ModeConfig is now PresetConfig. active_mode is now active_preset.
- active_tools[] on a Preset controls which skills are loaded.
  This is the mechanism for phase-aware agent capability filtering.
- Events are append-only. Never update or delete event records.
- DocumentVersion is append-only. Drift = diff(versions[-1], versions[-2]).
- WorkflowDefinition is a guest on the platform. The platform does not
  know what a Feature or Release is.
- File length cap: 300 lines. If a file needs more, it needs review.
- No shipping untested code for new modules. Existing code is exempt.
```

---

## Reference

This section is the single reference for types, config formats, file locations, and contracts. It consolidates what was previously in SPEC.md.

### Artifact Taxonomy

Ship manages three versioned artifact types:

| Type | Format | Atomic? | Description |
|---|---|---|---|
| **Skill** | `.md` (frontmatter + markdown) | yes | Single-purpose agent instruction |
| **Profile** | `.toml` | — | Named config: references skills + MCP + permissions |
| **Workflow** | `.toml` (planned) | — | Orchestration: references profiles + execution logic |

Skills are atoms. Profiles compose skills. Workflows compose profiles.

Every installed artifact (registry or local) is tracked in `~/.ship/ship.lock`:

```toml
[skills."rust-idioms@1.2.0"]
source = "registry"
r2_key = "skills/rust-idioms/1.2.0/SKILL.md"
checksum = "sha256:abc123"
installed_at = "2026-03-15T10:00:00Z"

[skills."my-deploy-flow"]
source = "local"
# no version, no key — authored locally, not published

[profiles."ship-studio-default@2.1.0"]
source = "registry"
r2_key = "profiles/ship-studio-default/2.1.0/profile.toml"
checksum = "sha256:def456"
skills = ["rust-idioms@1.2.0"]
installed_at = "2026-03-15T10:00:00Z"
```

`source = "local"` means authored on this machine, not fetched from registry.
`source = "registry"` means fetched — re-fetchable, content-addressed by checksum.

### Storage Model

```
Registry (getship.dev)
  R2  — artifact content blobs (immutable, CDN-served)
        skills/:id/:version/SKILL.md
        profiles/:id/:version/profile.toml
        workflows/:id/:version/workflow.toml
  D1  — artifact metadata (queryable)
        skills table: id, name, description, tags, author, version, r2_key, downloads
        profiles table: id, name, description, tags, author, version, r2_key, skill_refs
        workflows table: id, name, description, tags, author, version, r2_key, profile_refs
  DO  — user/org state (Rivet actors, self-hostable)
        UserActor: profile, personal skills (authored), installed manifest, usage
        OrgActor: members, shared profiles, billing

Local (~/.ship/)
  ship.lock     — installed artifact manifest (source, version, checksum, r2_key)
  skills/       — installed skill content (registry-fetched + locally authored)
  profiles/     — installed profile content (legacy: ~/.ship/modes/)
  cache/        — download cache (R2 objects, keyed by r2_key, LRU eviction)
  config.toml   — identity + defaults
  state/<slug>/platform.db  — per-project SQLite DB (see platform.db section)
  mcp/registry.toml         — named MCP server definitions

Project (.ship/, committed to git)
  ship.toml               — project identity + active profile ref
  agents/
    presets/*.toml         — project-scoped profiles
    skills/<id>/SKILL.md  — project-scoped skills
    rules/*.md             — always-on rules compiled into every output
    mcp.toml               — project MCP server definitions
    permissions.toml       — base permissions (profiles layer on top)
    hooks.toml             — event hook definitions
  modes/                   — legacy (renamed to presets; still read by CLI)
```

Rules:
- R2 stores content. D1 stores metadata + R2 keys. Never blob-store content in D1.
- `~/.ship/cache/` is transparent — populated on `ship use`, evictable at any time.
- `~/.ship/skills/` is the installed layer — analogous to global node_modules.
- Local authored skills (`source = "local"`) are never synced automatically. Publishing is an explicit `ship publish` action.
- Compiled provider files (CLAUDE.md, .mcp.json, etc.) are generated artifacts — gitignored, never committed. `.ship/` is the source of truth.

### .ship/ File Layout

Every path under `.ship/`, its owner, format, and who reads/writes it.

| Path | Format | Written by | Read by | Purpose |
|---|---|---|---|---|
| `ship.toml` | TOML | `ship init` / user | CLI, compiler, MCP | Project identity, default profile, providers |
| `agents/presets/<id>.toml` | TOML | user / `ship profile create` | CLI, compiler | Project-scoped profile definitions |
| `agents/skills/<id>/SKILL.md` | Markdown + frontmatter | user / `ship skill create` | CLI, compiler | Project-scoped skills |
| `agents/rules/*.md` | Markdown | user | compiler | Always-on rules, included in every output |
| `agents/mcp.toml` | TOML | user | CLI, compiler | Project MCP server definitions |
| `agents/permissions.toml` | TOML | user | compiler | Base permissions applied to all profiles |
| `agents/hooks.toml` | TOML | user | compiler | Event hook definitions |
| `modes/<id>.toml` | TOML | legacy | CLI (legacy path) | Legacy profile location; still resolved |
| `worktrees/<branch>/` | dir | MCP `create_workspace` | MCP, git | Git worktrees for imperative/declarative workspaces |
| `worktrees/<branch>/workspace.toml` | TOML | MCP `create_workspace` | MCP `complete_workspace` | Workspace name, kind, profile_id |
| `sessions/<workspace_id>/handoff.md` | Markdown | MCP `complete_workspace` | agents | Session handoff document |

Global paths (`~/.ship/`):

| Path | Format | Written by | Read by | Purpose |
|---|---|---|---|---|
| `config.toml` | TOML | `ship init --global` | CLI | Identity (name, email) + defaults |
| `ship.lock` | TOML | `ship use` | CLI | Installed artifact manifest |
| `profiles/<id>.toml` | TOML | `ship use` / registry | CLI, compiler | Installed/authored profiles |
| `skills/<id>/SKILL.md` | Markdown | `ship use` / user | CLI, compiler | Installed/authored skills |
| `modes/<id>.toml` | TOML | legacy | CLI (legacy) | Legacy global profile location |
| `mcp/registry.toml` | TOML | user | CLI, compiler | Named MCP server definitions |
| `cache/` | blobs | `ship use` | CLI | R2 download cache |
| `state/<slug>/platform.db` | SQLite | runtime | runtime, MCP | Per-project workspace/session/event DB |

`platform.db` location: `~/.ship/state/<project-slug>/platform.db` where `<project-slug>` is derived from the project's `.ship/` directory path. The DB is stored globally (outside the repo) and never committed to git.

### Config Schemas

**`~/.ship/config.toml`**

```toml
[identity]
name = "Alice"
email = "alice@example.com"

[defaults]
provider = "claude"
profile = "rust-expert"
```

**`.ship/ship.toml`** — `ProjectConfig` fields (`crates/core/compiler/src/types/config.rs`):

| Field | Type | Default | Notes |
|---|---|---|---|
| `version` | string | `"1"` | Schema version |
| `id` | string | `""` | nanoid, set by `ship init` |
| `name` | string? | — | Human name |
| `description` | string? | — | — |
| `providers` | string[] | `[]` | e.g. `["claude", "gemini"]` |
| `ai.provider` | string? | `"claude"` | Default AI provider |
| `ai.model` | string? | — | Model override |
| `ai.cli_path` | string? | — | CLI binary path override |
| `modes` | ModeConfig[] | `[]` | Inline mode definitions |
| `active_mode` | string? | — | Currently active mode id |
| `mcp_servers` | McpServerConfig[] | `[]` | Inline MCP server definitions |
| `hooks` | HookConfig[] | `[]` | Inline hook definitions |
| `git.ignore` | string[] | `[]` | Extra gitignore patterns |
| `git.commit` | string[] | `["agents","ship.toml",...]` | Paths committed by default |
| `statuses` | StatusConfig[] | backlog/in-progress/blocked/done | Workflow status definitions |

### Profile TOML Schema

File: `.ship/agents/presets/<id>.toml` or `~/.ship/profiles/<id>.toml`

```toml
[profile]
id = "rust-runtime"           # required; kebab-case identifier
name = "Rust Runtime"         # required; human display name
version = "0.1.0"             # optional; semver string
description = "..."           # optional
providers = ["claude"]        # optional; overrides project providers if set

[skills]
refs = ["ship-coordination"]  # optional; skill ids to activate. empty = all installed

[mcp]
servers = ["ship"]            # optional; MCP server ids to activate. empty = all configured

[plugins]
install = [
  "superpowers@claude-plugins-official",
]
scope = "project"             # "project" (default) or "user"

[permissions]
preset = "ship-guarded"       # ship-standard | ship-guarded | read-only | full-access
tools_deny = []               # additional deny patterns (glob)
tools_ask = ["Bash(rm -rf*)"] # patterns that require confirmation
default_mode = "default"      # "default" | "acceptEdits" | "plan" | "bypassPermissions"

[rules]
inline = """
Freeform rule text injected directly into the context output.
"""
```

Profile section fields:

| Section | Field | Type | Default | Notes |
|---|---|---|---|---|
| `[profile]` | `id` | string | — | required; kebab-case; unique in scope |
| `[profile]` | `name` | string | — | required; human display name |
| `[profile]` | `version` | string | — | semver string |
| `[profile]` | `description` | string | — | — |
| `[profile]` | `providers` | string[] | — | overrides project `providers` when set |
| `[skills]` | `refs` | string[] | `[]` | skill ids; empty = all installed skills |
| `[mcp]` | `servers` | string[] | `[]` | server ids; empty = all configured |
| `[plugins]` | `install` | string[] | `[]` | plugin ids in `<id>@<marketplace>` format |
| `[plugins]` | `scope` | string | `"project"` | `"project"` or `"user"` |
| `[permissions]` | `preset` | string | — | named preset; built-in: `ship-standard` \| `ship-guarded` \| `read-only` \| `full-access` |
| `[permissions]` | `tools_deny` | string[] | `[]` | additional deny glob patterns |
| `[permissions]` | `tools_ask` | string[] | `[]` | confirmation-required patterns |
| `[permissions]` | `default_mode` | string | from preset | `default` \| `acceptEdits` \| `plan` \| `bypassPermissions` |
| `[rules]` | `inline` | string | — | freeform text injected into context output |
| `[provider_settings.claude]` | any | object | — | merged verbatim into `.claude/settings.json` |

**Permission resolution order** (highest wins):
1. `[permissions] default_mode` in profile TOML
2. `default_mode` in the named preset section in `agents/permissions.toml`
3. Base `Permissions::default()`

**Built-in permission tiers:**
- `ship-standard` — base tools, `default_mode = "acceptEdits"`
- `ship-guarded` — base + deny `mcp__*__delete*` / `mcp__*__drop*`, `default_mode = "default"`
- `read-only` — allow `Read`, `Glob`, `LS` only
- `full-access` — allow `*`, `default_mode = "bypassPermissions"`

**Global Claude approval:** when ship is in a profile's MCP servers list and the `claude` provider is compiled, `ship use` writes `mcp__ship__*` to `~/.claude/settings.json` permissions allow. This avoids per-session approval prompts.

**Skill resolution order:** `.ship/agents/skills/` → `~/.ship/skills/` → cache → registry.
**Server resolution order:** `agents/mcp.toml` (project) → `~/.ship/mcp/registry.toml` (global).

### MCP Server Config Fields (`agents/mcp.toml` or inline in `ship.toml`)

| Field | Type | Default | Notes |
|---|---|---|---|
| `id` / `name` | string | — | identifier + human name |
| `command` | string | — | binary to execute (stdio) |
| `args` / `env` | string[] / map | `[]` / `{}` | arguments + environment |
| `scope` | string | `"global"` | `"global"` or `"project"` |
| `server_type` | enum | `stdio` | `stdio` \| `sse` \| `http` |
| `url` | string? | — | URL for SSE/HTTP transport |
| `disabled` | bool | `false` | exclude from compile output |
| `timeout_secs` | u32? | — | connection timeout |

Hook `trigger` values: `PreToolUse` \| `PostToolUse` \| `Notification` \| `Stop` \| `SubagentStop` \| `PreCompact`.

### Skill Format (`SKILL.md`)

File: `<skills-dir>/<id>/SKILL.md`

```markdown
---
name: Rust Idioms
id: rust-idioms
version: 0.1.0
description: Idiomatic Rust patterns and error handling
author: ship
---

# Rust Idioms

Use `?` for error propagation. Prefer `thiserror` over `anyhow` for library crates.
```

Frontmatter fields:

| Field | Type | Required | Notes |
|---|---|---|---|
| `name` | string | yes | Human display name |
| `id` | string | no | Kebab-case; inferred from directory name if omitted |
| `version` | string | no | semver string |
| `description` | string | no | Short summary |
| `author` | string | no | Author identifier |

Skill id constraints: lowercase ASCII, digits, and `-` only; 1–64 chars; no leading/trailing `-`; no `--`.

Skill body is freeform markdown. The compiler writes the full file content into the provider's skills directory.

**Skill resolution order:**
1. `.ship/agents/skills/<id>/SKILL.md` — project scope
2. `~/.ship/skills/<id>/SKILL.md` — global installed
3. `~/.ship/cache/` — cached registry fetch
4. Registry API — network fetch

### Compiler — Input / Output Contract

**Input: `ProjectLibrary` (JSON)**

```json
{
  "modes": [{ "id": "...", "name": "...", "active_tools": [], "skills": [], "mcp_servers": [], "rules": [], "hooks": [], "permissions": {} }],
  "active_mode": null,
  "mcp_servers": [{ "id": "...", "name": "...", "command": "...", "args": [], "env": {}, "scope": "global", "server_type": "stdio" }],
  "skills": [{ "id": "...", "name": "...", "description": null, "version": null, "content": "...", "source": "custom" }],
  "rules": [{ "name": "...", "content": "..." }],
  "permissions": { "tools": { "allow": ["*"], "ask": [], "deny": [] }, "filesystem": { "allow": [], "deny": [] }, "commands": { "allow": [], "deny": [] }, "network": { "policy": "none", "allow_hosts": [] }, "agent": { "require_confirmation": [] }, "default_mode": null },
  "hooks": [{ "id": "...", "trigger": "PreToolUse", "matcher": null, "command": "..." }],
  "plugins": { "install": [], "scope": "project" }
}
```

**WASM API (`packages/compiler` / `@ship/compiler`)**

```typescript
compileLibrary(library_json: string, provider: string, active_mode?: string): string
compileLibraryAll(library_json: string, active_mode?: string): string
listProviders(): string[]   // ["claude", "gemini", "codex", "cursor", "windsurf"]
```

**`CompileResult` shape (JSON returned by WASM)**

| Field | Type | Notes |
|---|---|---|
| `provider` | string | Provider id |
| `context_content` | string? | CLAUDE.md / GEMINI.md / AGENTS.md content |
| `mcp_servers` | JSON | MCP server entries object |
| `mcp_config_path` | string? | Relative path where MCP config is written |
| `skill_files` | map | `path → content` for each skill file |
| `rule_files` | map | `path → content` for per-file rules (Cursor .mdc) |
| `claude_settings_patch` | JSON? | `permissions`, `hooks`, agent limits (claude only) |
| `codex_config_patch` | string? | TOML `[mcp_servers.<id>]` entries (codex only) |
| `gemini_settings_patch` | JSON? | `hooks` section for `.gemini/settings.json` (gemini only) |
| `gemini_policy_patch` | string? | TOML policy file for `.gemini/policies/ship.toml` (gemini only) |
| `cursor_hooks_patch` | JSON? | Full `.cursor/hooks.json` content (cursor only) |
| `cursor_cli_permissions` | JSON? | `.cursor/cli.json` permissions (cursor only) |
| `plugins_manifest` | object | `{ install: [{id, provider}], scope }` |

**Provider Output Matrix**

| Provider | Context file | MCP config | Skills dir | Settings |
|---|---|---|---|---|
| `claude` | `CLAUDE.md` | `.mcp.json` | `.claude/skills/<id>/SKILL.md` | `.claude/settings.json` patch |
| `gemini` | `GEMINI.md` | `.gemini/settings.json` (nested) | `.agents/skills/<id>/SKILL.md` | `.gemini/settings.json` + `.gemini/policies/ship.toml` |
| `codex` | `AGENTS.md` | `.codex/config.toml` | `.agents/skills/<id>/SKILL.md` | — |
| `cursor` | — (per-file `.mdc`) | `.cursor/mcp.json` | `.cursor/skills/<id>/SKILL.md` | `.cursor/cli.json` + `.cursor/hooks.json` |
| `windsurf` | `.windsurfrules` | — | `.agents/skills/<id>/SKILL.md` | — |

Cursor uses `.cursor/rules/*.mdc` (one file per rule) instead of a single context file. Windsurf uses `.windsurfrules` (single markdown file).

**Provider Feature Matrix** (governed by `ProviderFeatureFlags` in the Rust compiler)

| Provider | `supports_mcp` | `supports_hooks` | `supports_tool_permissions` | `supports_memory` |
|---|---|---|---|---|
| `claude` | yes | yes | yes | yes (`CLAUDE.md`) |
| `gemini` | yes | yes | yes | yes (`GEMINI.md`) |
| `codex` | yes | — | — | yes (`AGENTS.md`) |
| `cursor` | yes | yes | yes | — (per-file `.mdc` rules) |
| `windsurf` | — | — | — | yes (`.windsurfrules`) |

### Generated Files (gitignored — never commit)

```
CLAUDE.md              ← claude context
AGENTS.md              ← codex/openai/gemini fallback context
GEMINI.md              ← gemini context
.windsurfrules         ← windsurf rules file
.mcp.json              ← claude MCP config
.cursor/               ← cursor rules, mcp, hooks, permissions
.codex/config.toml     ← codex MCP + config patch
.gemini/               ← gemini settings + policies
.claude/skills/        ← compiled skills for claude
.agents/skills/        ← compiled skills for codex/gemini/windsurf
.cursor/skills/        ← compiled skills for cursor
```

### platform.db Schema

Location: `~/.ship/state/<project-slug>/platform.db` (SQLite, WAL mode)

| Table | Key columns | Purpose |
|---|---|---|
| `schema_migrations` | `version TEXT PK`, `applied_at TEXT` | Migration tracking |
| `kv_state` | `(namespace, key) PK`, `value_json`, `updated_at` | Generic key-value store |
| `event_log` | `seq INTEGER PK AUTOINCREMENT`, `timestamp`, `actor`, `entity`, `action`, `subject`, `details?` | Append-only event log |
| `workspace` | `id TEXT PK`, `branch TEXT UNIQUE`, `worktree_path?`, `workspace_type`, `status`, `active_profile?`, `providers_json`, `skills_json`, `mcp_servers_json`, `plugins_json`, `compiled_at?`, `compile_error?`, `created_at`, `updated_at` | Workspace records |
| `workspace_session` | `id TEXT PK`, `workspace_id FK`, `branch`, `status`, `profile_id?`, `primary_provider?`, `goal?`, `summary?`, `started_at`, `ended_at?`, `created_at`, `updated_at` | Session records |
| `branch_config` | `branch TEXT PK`, `profile_id`, `workspace_id? FK`, `plugins_json`, `compiled_at`, `updated_at` | Last-compiled profile per branch |
| `job` | `id TEXT PK`, `kind`, `status`, `branch?`, `payload_json`, `created_by?`, `created_at`, `updated_at` | Coordination jobs |
| `job_log` | `id INTEGER PK AUTOINCREMENT`, `job_id? FK`, `branch?`, `message`, `actor?`, `created_at` | Job log entries |
| `note` | `id TEXT PK`, `title`, `content`, `tags_json`, `branch?`, `synced_at?`, `created_at`, `updated_at` | Project notes |
| `adr` | `id TEXT PK`, `title`, `status`, `date`, `context`, `decision`, `tags_json`, `supersedes_id?`, `created_at`, `updated_at` | Architecture decision records |

`workspace.workspace_type` values: `declarative` \| `imperative` \| `service` (default: `declarative`)
`workspace_session.status` values: `active` \| `ended`
`job.status` values: `pending` \| `running` \| `complete` \| `failed`
`adr.status` values: `proposed` \| `accepted` \| `rejected` \| `superseded`

**Job payload standard fields:**

| Field | Type | Notes |
|---|---|---|
| `description` | string | Human-readable job description |
| `requesting_workspace` | string? | Branch/id of the workspace that created the job |
| `title` | string? | Short title |
| `milestone` | string? | Target milestone or branch |

### MCP Tools

Server: `ship-mcp` binary (`apps/mcp/`). Core tools always available; non-core tools require an active mode listing the tool in `active_tools`.

| Tool | Purpose |
|---|---|
| `open_project` | Set active project for subsequent calls |
| `create_note` | Create a note in platform.db |
| `list_notes_tool` | List project notes |
| `create_adr` | Create an ADR record |
| `list_adrs_tool` | List ADRs |
| `activate_workspace` | Activate workspace by branch, optionally set mode |
| `create_workspace` | Create workspace + git worktree |
| `complete_workspace` | Write handoff.md + optionally prune worktree |
| `list_stale_worktrees` | List worktrees idle beyond threshold |
| `set_mode` | Activate or clear active mode |
| `sync_workspace` | Sync workspace to current branch context |
| `repair_workspace` | Detect and repair compile/config drift |
| `list_workspaces` | List all workspaces, optionally filter by status |
| `start_session` | Start a workspace session |
| `end_session` | End active session with summary |
| `log_progress` | Record progress note in active session |
| `list_skills` | List available skills |
| `create_job` | Create coordination job |
| `update_job` | Update job status |
| `list_jobs` | List jobs, filter by branch/status |
| `append_job_log` | Append log entry to a job |

MCP Resources: `ship://project_info`, `ship://adrs`, `ship://adrs/{id}` — read-only context snapshots.

### CLI Commands

```
ship init [--global]               # scaffold .ship/ or ~/.ship/
ship login / logout / whoami

ship use [<profile-id>]            # activate profile + emit provider files
                                   # no args = re-emit current profile
ship use --list                    # list available profiles (local + registry)
ship status                        # show active profile, providers, last built

ship skill list                    # local + registry
ship skill add <source>            # install from registry or local path
ship skill create <id>             # scaffold new skill
ship skill publish <id>            # publish local skill to registry

ship profile list
ship profile add <id>              # install from registry
ship profile create <id>
ship profile publish <id>

ship import                        # detect existing provider configs, import to .ship/
ship mcp list | add | remove

ship publish                       # publish active library to registry (requires auth)
ship sync                          # sync personal skills/profiles to account (requires auth)
ship cache clean                   # evict ~/.ship/cache/
```

`ship use` is the primary command. It installs any missing deps, activates the profile, and emits all provider files. Called automatically on branch switch via git post-checkout hook.

### Workspace Tracking

Ship tracks workspace state in `platform.db` (`~/.ship/state/<slug>/platform.db`). No project state lives in git-tracked files.

`ship.toml` carries a stable `id` (nanoid). This is the cross-machine project key — cloning on multiple machines shares the same `id` because `ship.toml` is committed.

Branch-profile flow:
1. `ship init` — creates project, writes `ship.toml` with nanoid
2. `ship use <profile>` — compiles profile, upserts `branch_config` for current branch
3. Post-checkout git hook — on branch switch: look up `branch_config`; if found, `ship use <stored_profile_id>` silently; if not found, inherit from base branch or `[defaults] profile`

### GitHub Integration

**Import (unauthenticated, public repos)**

`POST /api/github/import { url: "https://github.com/owner/repo" }` — fetches and extracts `CLAUDE.md`, `.mcp.json`, `.cursor/rules/`, `AGENTS.md`, `.gemini/` from the repo. Returns a `ProjectLibrary` JSON ready for the Studio compiler.

**PR Flow (requires GitHub App OAuth)**

`POST /api/github/pr { repo: "owner/repo", library: ProjectLibrary }` — creates a PR adding `.ship/` scaffold and a `.gitignore` patch. Provider files are NOT in the PR — they are generated locally after `ship use`.

### Ownership Map

| What | Owner |
|---|---|
| Compiler types + WASM | `crates/core/compiler` |
| CLI commands + config types | `apps/ship-studio-cli` |
| Studio web UI | `apps/web` |
| Shared UI primitives | `packages/primitives` |
| WASM package | `packages/compiler` |
| Auth + API endpoints | `apps/web/src/routes/api/` (Cloudflare Workers) |
| D1 schema | `apps/web/src/db/` |
| Platform runtime types + DB | `crates/core/runtime` |
| MCP server | `apps/mcp` |
| CLI path helpers | `apps/ship-studio-cli/src/paths.rs` |
| Workflow types | shipflow package (not yet built) |

**Platform owns:** Workspace, Profile, Session, Skill, MCP, Permission, Hook, Event
**Workflow owns:** Feature, Release, Issue, Spec, Vision — not in platform code

### `ship init` Scaffolding

```
.ship/
  ship.toml             # project identity, no profile active by default
  .gitignore            # CLAUDE.md, AGENTS.md, .mcp.json, .cursor/, .codex/, .gemini/
  agents/
    rules/              # always-on rules (.md files)
    skills/             # project-specific skills
    presets/            # project-specific profiles
    mcp.toml            # MCP server definitions
    permissions.toml    # base permissions
```

`ship init --global` creates `~/.ship/` with `config.toml`, empty `profiles/`, `skills/`, `modes/`, `mcp/`, `cache/`.

---

*Update this document when architectural decisions change. It is the context firewall between current intent and prior intent.*
