+++
id = "yjC3XiJu"
title = "Shipwright Runtime — Core Primitives and Hook Model"
created = "2026-03-03T15:41:38.105254107+00:00"
updated = "2026-03-03T16:30:00.000000000+00:00"
author = ""
tags = []
+++

## Overview

The Shipwright runtime is a domain-agnostic intent-to-output engine. Apps (Ship, Cowork, site generator, blog) are thin domain layers: they register Modules, provide hook implementations, and get sync, site generation, artifact pipelines, and context compilation for free.

The runtime knows nothing about "issues", "features", or "ADRs". It knows about Modules, Edges, Workspaces, and how to compile them into agent context.

---

## Alpha vs. Future

| Primitive | Alpha (1-2 days) | Future |
|---|---|---|
| Module trait + registry | Define trait, Ship registers its modules | Full plugin loading, Deno extensions |
| Status machine | Transition validation in registry | Guards, computed triggers |
| Sub-entities | Formal sub-entity API on Module trait | Cross-module sub-entity queries |
| Context compiler | Hookable select + context_repr | Relevance scoring, budget optimisation |
| Relationship graph | Typed edges table in SQLite | Full graph traversal API |
| MCP/CLI as shims | Thin service layer both route through | Stable external SDK |
| Sync engine | — | Cloud tier |
| Site generator | — | v0.3 |
| Artifact engine | — | v0.3 |
| Approval gates | — | Agent orchestration feature |
| @shipwright/ui | — | Ongoing alongside features |

---

## Intent Primitives

### Module *(not Document — richer aggregate)*

Not a flat record. A Module is a full domain aggregate with sub-entities, a status machine, computed properties, agent integration points, and its own UI surface. "Document" undersells this — Feature, ADR, and Release are modules in their own right.

Ship's modules:

- **Feature** — sub-entities: Todos, AcceptanceCriteria, AgentConfig. Computed: completion %, readiness score. Lifecycle: planned → in-progress → implemented → deprecated.
- **ADR** — sub-entities: Alternatives (weighted scoring matrix), Consequences. Lifecycle: proposed → accepted → rejected → superseded.
- **Release** — sub-entities: ChangelogEntries, BreakingChanges, FeatureLinks. Computed: readiness (% of linked features implemented).
- **Issue** — sub-entities: Comments, Labels. Priority signal for context compiler.
- **Spec** — AcceptanceCriteria as structured sub-entities, not free text.
- **Note** — lightweight, minimal lifecycle.

Other domains register their own modules:
- Site generator: Page, Post, Section
- Cowork: Thread, Brief, Decision
- Blog: Post, Author, Category

### Module Registry

The runtime's primary extension point. Modules register themselves; the runtime never hardcodes entity types.

**Alpha:** Rust trait `ShipModule` implemented by each entity type. Runtime holds a registry of boxed trait objects. MCP and CLI call registry operations, not entity-specific functions.

```rust
trait ShipModule {
    fn type_id() -> &'static str;
    fn schema() -> ModuleSchema;
    fn sub_entities() -> Vec<SubEntityDef>;
    fn lifecycle() -> StatusMachine;
    fn computed() -> Vec<ComputedFieldDef>;
    fn relationships() -> Vec<RelationshipDef>;
}
```

### Edge

A typed, directed relationship between two Module instances. Carries `edge_type`, metadata, `created_at`. Forms the intent graph the context compiler traverses.

Current state: entity links exist as scalar fields (`spec_id`, `release_id`). **Alpha:** migrate to an explicit `edges` table in SQLite. Enables graph traversal, cross-module queries, and richer context compilation.

### Workspace

The active scoped context: a subset of the module graph relevant to the current task. Determined by branch, active module, user focus, or explicit selection. The context compiler operates on a Workspace, not the full graph.

Partially implemented (branch → feature mapping). **Alpha:** formalise as a first-class struct the context compiler accepts, replacing ad-hoc path resolution.

### Principal

An identity (user or org) that owns Module instances and against which permissions are evaluated. Auth implementation is NOT runtime — the runtime consumes a validated Principal token from the cloud layer (Clerk/WorkOS). The runtime owns the permission evaluation model; the cloud layer owns identity.

---

## Complete Hook Matrix

Every hook point the runtime exposes. Domain apps implement hooks relevant to their modules.

### Module Registration Hooks

| Hook | Signature | Required | Description |
|---|---|---|---|
| type_id | () → &str | ✓ | Unique module type identifier |
| schema | () → ModuleSchema | ✓ | Field definitions and types |
| sub_entities | () → SubEntityDef[] | | Sub-aggregate definitions |
| lifecycle | () → StatusMachine | | Valid statuses and guarded transitions |
| computed | () → ComputedFieldDef[] | | Derived fields computed from sub-entities |
| relationships | () → RelationshipDef[] | | Valid edge types to other module types |

### Context Compiler Hooks

| Hook | Signature | Required | Description |
|---|---|---|---|
| select_modules | (workspace, task_type) → Module[] | ✓ | What enters the compiler |
| relevance_score | (module, task) → f32 | | Override default ranking |
| context_repr | (module) → ContextFragment | | How this module presents to agents |

**`context_repr` is the richest hook.** A Feature with 8/10 todos complete and 2 open blockers produces materially different context than a blank Feature. ADR alternatives with weighted scores produce richer signal than the decision text alone. The runtime cannot mine this without the module's own representation logic. This is where structured compilation beats RAG — the module knows what signal matters.

### Sync Engine Hooks

| Hook | Signature | Required | Description |
|---|---|---|---|
| sync_filter | (workspace) → Module[] | | What participates in sync; default: all |
| conflict_resolution | (local, remote) → Module | | Override last-write-wins per module type |

### Site Generator Hooks

| Hook | Signature | Required | Description |
|---|---|---|---|
| select_modules | (workspace) → Module[] | ✓ | Content that feeds site generation |
| page_schema | () → Schema | ✓ | Site structure definition |
| template | () → Theme | ✓ | Visual theme and layout |
| section_repr | (module) → SectionContent | | How a module renders into a site section |

### Renderer Pipeline Hooks

| Hook | Signature | Required | Description |
|---|---|---|---|
| output_format | () → ArtifactType | ✓ | Target format (code, markdown, HTML, JSON) |
| transform | (agent_output) → Artifact | | Post-process raw agent output |
| validate | (artifact, workspace) → Result | | Verify output satisfies intent |

### Execution Hooks

| Hook | Signature | Required | Description |
|---|---|---|---|
| approval_required | (task, checkpoint) → bool | | When to pause for human review |
| cost_limit | () → Budget | | Per-task spending ceiling |

---

## MCP and CLI as Transport Shims

MCP and CLI are transports over runtime operations — not business logic owners. Today they contain duplicated logic that belongs in the runtime. The target:

```
ship feature create "X"              mcp::create_feature(req)
  ↓ parse args                         ↓ deserialise
  └──────────────────────────────────────────────────┐
              runtime.registry.create(FeatureModule, data)
              validation, sub-entities, edge creation,
              event log, status machine — all here
              ↑──────────────────────────────────────┘
  ↓ format output                      ↓ serialise response
```

**Alpha:** introduce a thin service layer in `crates/runtime` that both MCP and CLI call. Not a full rewrite — extract shared logic duplicated between the two transports today.

---

## Runtime-Owned Features

### Sync Engine

Syncs Module instances and Edges across devices and users. Operates at the Module level — no knowledge of Ship-specific types. Every app on the runtime gets sync for free by virtue of using Module primitives.

Transport: Rivet actors on Cloudflare Durable Objects (cloud) or self-hosted Rivet (enterprise). See sync architecture ADR.

| Tier | Sync |
|---|---|
| Local only (free) | No sync engine active |
| Personal cloud | Rivet on Cloudflare DO |
| Enterprise self-hosted | Self-hosted Rivet cluster |

### Site Generator

Transforms a Module collection into a static site. Runtime owns the interaction model: streaming side-by-side editing UI, live preview, intervention, artifact versioning. Domain provides content strategy via hooks.

Ship uses it for documentation and marketing sites. Cowork for session summaries. A blog app for posts. Same UI, same pipeline, different hooks.

---

## Shared Frontend (`@shipwright/ui`)

Component library operating on runtime primitives, not Ship-specific types. Every app assembles from the same kit.

| Component | Operates on | Notes |
|---|---|---|
| Module editor | ModuleSchema | Renders any registered schema as structured editor |
| Graph explorer | Edge[] | Relationship browser; works for feature deps or site structure |
| Compilation inspector | ContextFragment[] | Context window contents ranked by relevance |
| Pipeline monitor | Task + streaming output | Approval gate UI, cost meter, tool call display |
| Artifact viewer | Artifact | Diff, markdown, HTML preview, side-by-side |
| Agent chat | Task | Streaming chat with tool calls and approval UI |
| Status machine UI | StatusMachine | Renders any module's lifecycle as interactive board |

---

## Acceptance Criteria

### Alpha
- `ShipModule` trait defined in `crates/runtime`
- Feature, Issue, ADR, Spec, Release, Note implement the trait
- Module registry holds trait objects; MCP and CLI route create/update/list through it
- `edges` table in SQLite; existing scalar links (`spec_id`, `release_id` etc.) preserved as convenience fields but backed by edges
- Context compiler accepts `select_modules` and `context_repr` hooks
- Workspace struct is the compiler's input, not ad-hoc path resolution

### Future
- Runtime crate exposes stable public API for external Module registration
- Sync engine operates at Module level with no Ship-specific knowledge
- Site generator renders any Module collection via registered hooks
- `@shipwright/ui` components typed against runtime primitives, not Ship entities
- Second app built on runtime validates hook model without runtime changes
