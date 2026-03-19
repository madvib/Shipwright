# Ship Platform Architecture

## Context Firewall — Read Before Touching Code

> **Status**: v0.1.0 — Current Intent
> **Rule**: If something you're building isn't in the Platform Layer, it belongs in a Workflow Definition.
> **Updated**: 2026-03-18
> **Reference tables** (schemas, provider matrix, CLI commands, MCP tools): see REFERENCE.md

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

## Current State vs Target (v0.1.0 → v0.2.0)

### Known Violations — Workflow Types in the Platform Layer

The following SDLC/workflow types currently exist in platform code. They should not be there. Do not add new references to them in platform code. They are scheduled for extraction in 0.2.0.

**`state_db/schema_ext.rs`** contains tables that are Shipflow document types, not platform primitives:
- `feature`, `feature_todo`, `feature_criterion`
- `release`, `release_breaking_change`
- `feature_doc`, `feature_doc_revision`
- `spec`, `adr`, `adr_option`

**`WorkspaceDbRow`** carries `feature_id` and `updated_feature_ids` — workflow-layer foreign keys on a platform type.

**`git_workspace` table** has `feature_id` and `release_id` columns — workflow-layer joins in a platform table.

**`feature_capability` and `target_feature`** are workflow-layer join tables that have no place in the platform schema.

**Rule:** Do not add new references to these types in platform code. Do not add new SDLC concepts (`feature_id`, `release_id`, `spec_id`, etc.) to any platform type. These will be extracted in 0.2.0.

---

### Data Access Rule

Agents and tooling must access Ship data through the MCP server tools or the `ship` CLI. Direct SQLite3 queries against the platform database are prohibited. The schema is not a stable API — it evolves across releases.

**Allowed:**
- MCP tools (`list_jobs`, `list_workspaces`, `list_capabilities`, etc.)
- `ship` CLI commands

**Not allowed:**
- `sqlite3` CLI against the platform DB
- Python or any language's sqlite3 bindings against the platform DB
- Any raw SQL outside of the runtime crate itself

---

### 0.2.0 Cleanup Plan

The following changes will complete the platform/workflow separation:

- `schema_ext.rs` becomes Shipflow's schema extension, loaded conditionally when Shipflow is the active workflow — not compiled into the platform unconditionally.
- `workspace` loses hardcoded `feature_id`/`target_id` columns, replaced by a generic `workflow_context_json` field that workflows populate with their own keying.
- `workflow.toml` introduced as the customization layer for the project management platform — the boundary file that separates what the platform owns from what a workflow package provides.
- Shipflow declared as a workflow package that extends the platform via `workflow.toml`, making the guest/host relationship structurally enforced rather than just documented.

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

## Registry — Git-Native Package Model

Ship uses a git-native registry model. Dependencies are git repositories, not a central blob store. There is no R2, D1, or Durable Objects in the registry path.

### ship.toml — Project Manifest

Every `.ship/` directory contains a `ship.toml` with three sections:

```toml
[module]
name = "github.com/owner/repo"
version = "0.1.0"
description = "..."
license = "MIT"

[dependencies]
# Dep skill packages by git path. Resolved by `ship install`.
# "github.com/org/pkg" = "v1.2.0"

[exports]
# Paths to first-party skills and agents published from this repo.
skills = [
  "agents/skills/my-skill",
]
agents = [
  "agents/profiles/default.toml",
]
```

### ship.lock — Dependency Lockfile

`ship.lock` is committed to git. It pins every resolved dependency:

```toml
[deps."github.com/org/pkg"]
path = "github.com/org/pkg"
version = "v1.2.0"
commit = "abc123def456..."
hash = "sha256:..."
```

- `path` — canonical module path (matches `[dependencies]` key)
- `version` — resolved semver tag
- `commit` — exact git commit SHA (reproducible fetch)
- `hash` — sha256 of fetched content tree (integrity check)

Fetched package content is stored at `~/.ship/cache/objects/<sha256>/`.

### Dep skill resolution

A skill ref in an agent TOML prefixed with `github.com/` is a dep ref:
- `github.com/owner/pkg/skills/name` → resolved via `ship.lock` → fetched to cache
- Unprefixed refs resolve from `.ship/agents/skills/` (local scope)

Cache miss during compile produces an actionable error: `dependency not in cache — run ship install`.

### Post-checkout hook (planned, not yet implemented)

`ship init` will install a git post-checkout hook that runs `ship use <stored-profile>` on branch switch. When implemented: `branch_config` table in platform.db stores the last profile per branch; the hook looks it up and re-emits provider files silently.

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

`ProjectConfig` currently conflates platform config, preset config, and agent config. The target separation mirrors the registry manifest format:

**`ship.toml` — registry manifest (new format):**
```toml
[module]
name = "github.com/owner/repo"
version = "0.1.0"
description = "..."

[dependencies]
# dep skill packages

[exports]
skills = ["agents/skills/..."]
agents = ["agents/profiles/..."]
```

**Preset definitions** — separate, referenced by id, live in `.ship/agents/presets/`

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
- Profile/preset files live in .ship/agents/presets/ (not modes/).
  Use profiles/ or agents/ when referring to these paths in docs and code.
- ship.toml uses [module]/[dependencies]/[exports] format. Legacy fields
  (statuses, namespaces, active_mode) are transitional — do not add new ones.
- Registry is git-native: no R2, D1, or Durable Objects in the registry path.
- File length cap: 300 lines. If a file needs more, it needs review.
- No shipping untested code for new modules. Existing code is exempt.
- See REFERENCE.md for provider matrix, platform.db schema, MCP tools, CLI commands.
```

---

*Update this document when architectural decisions change. It is the context firewall between current intent and prior intent.*
