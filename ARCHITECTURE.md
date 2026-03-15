# Ship Platform Architecture

## Context Firewall — Read Before Touching Code

> **Status**: v0.2 — Current Intent
> **Rule**: If something you're building isn't in the Platform Layer, it belongs in a Workflow Definition.
> **Updated**: 2026-03-15

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

*Update this document when architectural decisions change. It is the context firewall between current intent and prior intent.*
