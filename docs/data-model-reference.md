# Ship Data Model Reference

> Source-grounded reference for runtime structs, SQLite schemas, persistence patterns,
> and Tauri bridge types. Use this for visual regression passes and interface sync.
>
> Last updated: 2026-03-08. Generated from direct source reading, not memory.

---

## Storage Architecture

### Two SQLite databases

<!-- #1 I thought these were located at ~/.ship/state/$project/db and ~/.ship/ship.db?  This must be consistent -->
| Database | Location | Contains |
|---|---|---|
| Project DB | `.ship/db.sqlite` | All project entities + workspace state |
| Global DB | `~/.config/ship/db.sqlite` | Registered projects, user-scoped notes, global state |

### Dual persistence pattern

Several entities live in **both** SQLite and markdown files:

- **SQLite** = canonical read model (fast queries, status filtering, cross-entity joins)
<!-- EXPORTS MUST ALL CONTAIN A MESSAGE STATING THEY ARE GENERATED -->
- **Markdown** = agent export format (git-committed, human-readable, context for AI)

Issues and notes are **SQLite-only** — no markdown file.

<!-- This is incorrect. Features should be be stored in sqlite as text -->
Feature body/content is **markdown-only** — metadata is in SQLite but body is read from the `.md` file.

### Git policy (per `ship.toml`)
<!-- this needs to change. Default we include agents, ship.toml, VISION.md the rest are optional in settings -->
Configurable per category: `releases`, `features`, `specs`, `adrs`, `agents`, `ship.toml` are
committed by default. `issues` and `notes` are local-only by default.

---

## Entities

### Feature

**Persistence:** Split — metadata in SQLite (project DB)
Todos and criteria in separate child tables.

#### Rust types (`feature/types.rs`)


<!-- Do we not have a better date field? A feature can have many linked specs. Where are docs?? They should be attached here. I'd also like to see a link to capabilities in the code (like we listed earlier)-->
```rust
struct FeatureMetadata {
    id: String,                          // nanoid(8)
    title: String,
    description: Option<String>,         // short summary, not the body
    created: String,
    updated: String,
    release_id: Option<String>,
    active_target_id: Option<String>,    // tracks active workspace/target
    spec_id: Option<String>,
    branch: Option<String>,
    agent: Option<FeatureAgentConfig>,   // stored as agent_json in DB
    tags: Vec<String>,
}

struct Feature {
    metadata: FeatureMetadata,
    body: String,           // markdown body — NOT in SQLite, from .md file
    todos: Vec<FeatureTodo>,     // {id, text, completed}
    criteria: Vec<FeatureCriterion>, // {id, text, met}
}

struct FeatureEntry {
    id: String,
    file_name: String,
    path: String,
    status: FeatureStatus,
    feature: Feature,
}
```

**Note:** In list queries, `body`, `todos`, and `criteria` are returned empty. Only populated on `get_feature`.

#### Status enum
`planned | in-progress | implemented | deprecated`
Default: `planned`

#### Doc status enum (`FeatureDocStatus`)
`not-started | draft | reviewed | published`
Default: `not-started`

#### SQLite schemas

**`feature` table:**
```sql
id               TEXT PRIMARY KEY
title            TEXT NOT NULL
description      TEXT
status           TEXT NOT NULL DEFAULT 'planned'
release_id       TEXT
active_target_id TEXT
spec_id          TEXT
branch           TEXT
agent_json       TEXT    -- serialized FeatureAgentConfig
tags_json        TEXT NOT NULL DEFAULT '[]'
created_at       TEXT NOT NULL
updated_at       TEXT NOT NULL
```

**`feature_todo` table:**
```sql
id          TEXT PRIMARY KEY
feature_id  TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE
text        TEXT NOT NULL
completed   INTEGER NOT NULL DEFAULT 0  -- bool as int
ord         INTEGER NOT NULL DEFAULT 0
```

**`feature_criterion` table:**
```sql
id          TEXT PRIMARY KEY
feature_id  TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE
text        TEXT NOT NULL
met         INTEGER NOT NULL DEFAULT 0  -- bool as int
ord         INTEGER NOT NULL DEFAULT 0
```

**`feature_doc` table:**
```sql
feature_id       TEXT PRIMARY KEY REFERENCES feature(id) ON DELETE CASCADE
status           TEXT NOT NULL DEFAULT 'not-started'
content          TEXT NOT NULL DEFAULT ''  -- full markdown content
revision         INTEGER NOT NULL DEFAULT 1
last_verified_at TEXT
created_at       TEXT NOT NULL
updated_at       TEXT NOT NULL
```

**`feature_doc_revision` table** (audit trail):
```sql
id          TEXT PRIMARY KEY
feature_id  TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE
revision    INTEGER NOT NULL
status      TEXT NOT NULL
content     TEXT NOT NULL
actor       TEXT NOT NULL DEFAULT 'ship'
created_at  TEXT NOT NULL
```

#### Markdown file location
`.ship/project/features/{status}/{slug}.md`
<!--move away from frontmatter -->
Body is identified by `ship:feature id={id}` marker or `id = "{id}"` in frontmatter.
<!-- this should all be UI editable. MCP as well -->
#### UI fields to render / editable
| Field | Display | Editable |
|---|---|---|
| title | ✅ | ✅ |
| description | ✅ | ✅ |
| status | ✅ (badge) | ⚠️ start/done cmds not confirmed in UI |
| release_id | ✅ (link) | ⚠️ set at create only |
| spec_id | ✅ (link) | ⚠️ set at create only |
| branch | ✅ | ⚠️ |
| tags | ⚠️ | ⚠️ |
| todos | ⚠️ not shown in list | ⚠️ |
| criteria | ⚠️ not shown in list | ⚠️ |
| body (markdown) | ⚠️ | ✅ `update_feature_cmd` |
| feature_doc.content | ❌ not confirmed | ❌ not confirmed |
| agent config | ⚠️ | ⚠️ |

#### Capability + target link tables (runtime v0020)

These tables are now part of the SQLite schema and are intended to support the
“capability map + target slice” model:

```sql
capability_map(
  id TEXT PRIMARY KEY,
  vision_ref TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

capability(
  id TEXT PRIMARY KEY,
  map_id TEXT NOT NULL REFERENCES capability_map(id) ON DELETE CASCADE,
  title TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  parent_capability_id TEXT REFERENCES capability(id) ON DELETE SET NULL,
  status TEXT NOT NULL DEFAULT 'active',
  ord INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
)

feature_capability(
  feature_id TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE,
  capability_id TEXT NOT NULL REFERENCES capability(id) ON DELETE CASCADE,
  is_primary INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  PRIMARY KEY(feature_id, capability_id)
)

target_feature(
  target_id TEXT NOT NULL REFERENCES release(id) ON DELETE CASCADE,
  feature_id TEXT NOT NULL REFERENCES feature(id) ON DELETE CASCADE,
  ord INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  PRIMARY KEY(target_id, feature_id)
)
```

Legacy `feature.release_id` / `feature.active_target_id` still exist for compatibility while
the join-table model is being adopted across app/UI flows.

---

### Spec
<!--bad should be sqlite. markdown export possible manually, not auto-->
**Persistence:** Split — metadata in SQLite, body in markdown file.

#### Rust types (`spec/types.rs`)
<!-- Author does not really exist elsewhere let's remove for now -->
```rust
struct SpecMetadata {
    id: String,
    title: String,
    created: String,
    updated: String,
    author: Option<String>,
    branch: Option<String>,
    workspace_id: Option<String>,
    feature_id: Option<String>,
    release_id: Option<String>,
    tags: Vec<String>,
}

struct Spec {
    metadata: SpecMetadata,
    body: String,   // markdown content — from .md file
}

struct SpecEntry {
    id: String,
    file_name: String,
    path: String,
    status: SpecStatus,
    spec: Spec,
}
```

#### Status enum
`draft | active | archived`
Default: `draft`

#### SQLite schema (from `PROJECT_SCHEMA_ISSUES_SPECS`)

> Note: spec table schema was not directly observed in the DB schema constants reviewed.
> Inferred from CRUD operations and type definitions. Verify against actual migration.

Expected columns based on CRUD: `id, title, status, author, branch, workspace_id, feature_id, release_id, tags_json, created_at, updated_at`

#### Markdown file location
`.ship/project/specs/{status}/{slug}.md`
<!-- should all be editable in UI. links are lacking across the board -->
#### UI fields to render / editable
| Field | Display | Editable |
|---|---|---|
| title | ✅ | ✅ `update_spec_cmd` |
| body | ✅ | ✅ `update_spec_cmd` |
| status | ✅ | ⚠️ no confirmed move path |
| feature_id | ⚠️ | ⚠️ |
| release_id | ⚠️ | ⚠️ |
| tags | ⚠️ | ⚠️ |

---

### ADR (Architecture Decision Record)

**Persistence:** SQLite (project DB) + markdown export.

#### Rust types (`adr/types.rs`)

<!-- Specs are being thrown around everywhere, and yet have no visibility in the UI and honestly are not core workflow. Specs = commit message and audit trail. Most are archived. So we can link specs to other doc types but there is no 1-1 relationship between specs and anything else. -->

```rust
struct AdrMetadata {
    id: String,
    title: String,
    date: String,
    tags: Vec<String>,
    spec_id: Option<String>,
    supersedes_id: Option<String>,
}

struct ADR {
    metadata: AdrMetadata,
    context: String,    // background, constraints — freeform markdown
    decision: String,   // the committed decision — freeform markdown
}

struct AdrEntry {
    id: String,
    file_name: String,
    path: String,
    status: AdrStatus,
    adr: ADR,
}
```

#### Status enum
`proposed | accepted | rejected | superseded | deprecated`
Default: `proposed`

#### SQLite schema (`adr` table)
```sql
id            TEXT PRIMARY KEY
title         TEXT NOT NULL
status        TEXT NOT NULL DEFAULT 'proposed'
date          TEXT NOT NULL
context       TEXT NOT NULL DEFAULT ''
decision      TEXT NOT NULL DEFAULT ''
tags_json     TEXT NOT NULL DEFAULT '[]'
spec_id       TEXT
supersedes_id TEXT
created_at    TEXT NOT NULL
updated_at    TEXT NOT NULL
```

**`adr_option` table** (structured alternatives — defined in schema, not yet exposed in Rust struct):
```sql
id                TEXT PRIMARY KEY
adr_id            TEXT NOT NULL REFERENCES adr(id) ON DELETE CASCADE
title             TEXT NOT NULL
arguments_for     TEXT NOT NULL DEFAULT ''
arguments_against TEXT NOT NULL DEFAULT ''
ord               INTEGER NOT NULL DEFAULT 0
```
> ⚠️ `adr_option` table exists in schema but `ADR` struct has no `options` field.
> Either unused or reserved for future structured options UI.

#### Markdown file location
`.ship/project/adrs/{status}/{slug}.md`

#### UI fields to render / editable
| Field | Display | Editable |
|---|---|---|
| title | ✅ | ✅ `update_adr_cmd` |
| context | ✅ | ✅ `update_adr_cmd` |
| decision | ✅ | ✅ `update_adr_cmd` |
| status | ✅ | ✅ `move_adr_cmd` |
| tags | ⚠️ | ⚠️ |
| spec_id | ⚠️ | ⚠️ |
| supersedes_id | ⚠️ | ⚠️ |

---

### Release

**Persistence:** Split — metadata in SQLite, body in markdown file. Breaking changes in child table.

#### Rust types (`release/types.rs`)

```rust
struct ReleaseMetadata {
    id: String,
    version: String,
    status: ReleaseStatus,
    created: String,
    updated: String,
    supported: Option<bool>,
    target_date: Option<String>,
    tags: Vec<String>,
}

struct Release {
    metadata: ReleaseMetadata,
    body: String,                           // markdown — from .md file
    breaking_changes: Vec<ReleaseBreakingChange>,  // {id, text}
}

struct ReleaseEntry {
    id: String,
    file_name: String,
    path: String,
    version: String,
    status: ReleaseStatus,
    release: Release,
}
```
<!-- this is confusing, we do not need backwards compat at this stage. what is the difference between active and shipped? I feel like upcoming, active, deprecated works fine. open to rebuttal -->
#### Status enum
`planned | active | shipped | archived`
Default: `planned`
FromStr also accepts: `upcoming` → `planned`, `released` → `shipped`, `deprecated` → `archived`

#### SQLite schema

**`release` table:**
```sql
id          TEXT PRIMARY KEY
version     TEXT NOT NULL
status      TEXT NOT NULL DEFAULT 'planned'
target_date TEXT
supported   INTEGER    -- nullable bool
created_at  TEXT NOT NULL
updated_at  TEXT NOT NULL
```
> ⚠️ `tags_json` is in the Rust struct (`ReleaseMetadata.tags`) but NOT in the DB schema.
> Tags for releases may be markdown-only (frontmatter), not queryable.

**`release_breaking_change` table:**
```sql
id         TEXT PRIMARY KEY
release_id TEXT NOT NULL REFERENCES release(id) ON DELETE CASCADE
text       TEXT NOT NULL
ord        INTEGER NOT NULL DEFAULT 0
```

#### Markdown file location
`.ship/project/releases/` (exact status subdirectory structure — verify against `releases_dir()`)

#### UI fields to render / editable
| Field | Display | Editable |
|---|---|---|
| version | ✅ | ⚠️ |
| status | ✅ (badge) | ⚠️ no confirmed move path in UI |
| target_date | ⚠️ | ⚠️ |
| supported | ⚠️ | ⚠️ |
| body | ✅ | ✅ `update_release_cmd` (full replace) |
| breaking_changes | ⚠️ | ⚠️ no structured edit path confirmed |
| tags | ⚠️ | ⚠️ not in DB |

---

### Note

**Persistence:** SQLite only. Project-scoped notes in project DB, user-scoped in global DB.

#### Rust types (`note/types.rs`)

```rust
struct Note {
    id: String,
    title: String,
    content: String,    // raw markdown, no frontmatter
    tags: Vec<String>,
    scope: NoteScope,   // project | user
    created_at: String,
    updated_at: String,
}

struct NoteEntry {      // list shape
    id: String,
    title: String,
    scope: NoteScope,
    updated: String,
}
```

#### SQLite schema (`note` table)
```sql
id         TEXT PRIMARY KEY
title      TEXT NOT NULL
content    TEXT NOT NULL DEFAULT ''
tags_json  TEXT NOT NULL DEFAULT '[]'
scope      TEXT NOT NULL DEFAULT 'project'  -- 'project' | 'user'
created_at TEXT NOT NULL
updated_at TEXT NOT NULL
```

#### UI fields to render / editable
| Field | Display | Editable |
|---|---|---|
| title | ✅ | ✅ |
| content | ✅ | ✅ `update_note_cmd` |
| scope | ✅ | read-only after create |
| tags | ⚠️ | ⚠️ |

---

### Workspace
<!-- Large refactor incoming, replacing original content!! -->
**Persistence:** SQLite only (project DB). No markdown file.

#### Rust types (`workspace.rs`)
<!--
============================================================
WORKSPACE
============================================================
DECISION: branch → not PK. Use id (nanoid) as PK.
branch should be UNIQUE NOT NULL for Feature/Hotfix,
NULL for Process when you add it. Current migration:
add UNIQUE INDEX on branch, drop PRIMARY KEY constraint.

RENAME: WorkspaceType → WorkspaceKind (more precise)
COLLAPSE: Refactor + Experiment → remove both
RENAME: Project → Process (deferred to post-alpha)

STATUS: Current variants map to new model as:
  Idle     → Active (workspace exists, no session running)
  Active   → Active (same — session state is on Session now)
  Paused   → remove (this is SessionStatus::Paused)
  Archived → Clean (for git workspaces) OR keep Archived
             for explicit user-initiated hide. See note below.

NOTE on Archived vs Clean: Archived currently implies
deprecation. Real end states are:
  Feature/Hotfix: Clean (no changes) → Review → Merged
  Process: Operational | Degraded | Offline | Error (post-alpha)
For alpha: simplify to Active | Archived. Merged is stretch goal.
-->
```rust
enum WorkspaceKind {
    Feature,
    Hotfix,     // absorbs Experiment — one-off, scoped, temporary
    // Process  // DEFERRED: post-alpha. Shares primitive but
                // has different lifecycle, status model, and
                // no git scope. Model as separate table extension
                // when surfaced in UI.
}

enum WorkspaceStatus {
    Active,     // replaces Idle + Active (session state lives on Session)
    Archived,   // user-initiated hide. Rename candidate: Clean or Merged later.
}

struct Workspace {
    id: String,              // PK — nanoid. branch is now a unique index.
    branch: Option<String>,  // UNIQUE NOT NULL for Feature/Hotfix in practice.
                             // Option to future-proof for Process.
    kind: WorkspaceKind,     // was workspace_type
    status: WorkspaceStatus,

    feature_id: Option<String>,   // keep — 1:1 for Feature workspaces
    spec_id: Option<String>,      // REVISIT: specs are not 1:1 with workspaces.
                                  // This field implies ownership that doesn't exist.
                                  // Consider removing and querying via SessionSpec
                                  // junction instead. For alpha: keep but don't rely on it.
    release_id: Option<String>,   // keep

    // MODE: active_mode is doing two jobs:
    //   1. Workspace preset/seed (applied at creation)
    //   2. Runtime capability toggle (edit/plan/review)
    // RENAME to preset_id. Runtime capability moves to Session.
    // See AgentMode table — this references that.
    preset_id: Option<String>,    // was active_mode / mode_id

    providers_json: Vec<String>,  // RENAME to providers. The _json suffix
                                  // is misleading on a typed field.
                                  // In SQL keep as TEXT, deserialize in app.

    // WORKTREE: collapse is_worktree + worktree_path
    // worktree_path.is_some() IS is_worktree. Drop the bool.
    worktree_path: Option<String>,  // presence implies is_worktree = true

    last_activated_at: Option<DateTime<Utc>>,  // was String — use proper type

    // COMPILATION STATE: these four belong together.
    // Extract to WorkspaceCompilation table post-alpha.
    // For now keep inline but treat as a unit:
    //   config_generation increments on every compile.
    //   Staleness check = session.compile_generation != workspace.config_generation
    //   resolved_at is confusingly named — RENAME to compiled_at or drop.
    //   compiled_at and resolved_at are duplicates. Pick one.
    config_generation: i64,
    compiled_at: Option<DateTime<Utc>>,  // was String
    compile_error: Option<String>,
    // resolved_at: REMOVE — duplicate of compiled_at with unclear semantics.
    // context_hash: keep — useful for staleness detection beyond generation int.
    context_hash: Option<String>,
}```

#### SQLite schema (migrations applied in order)

**Base (`workspace` table, v1 + v2 + compile_state):**
```sql
-- workspace table
-- MIGRATION NOTES:
-- v3: add id column if not exists (done in v2 per schema)
-- v3: add UNIQUE constraint on branch
-- v3: rename workspace_type → kind (or alias in queries for now)
-- v3: drop resolved_at OR rename to compiled_at (duplicates exist)
-- v3: rename active_mode → preset_id
-- v3: drop is_worktree (redundant with worktree_path IS NOT NULL)
-- v3: status DEFAULT changes from 'idle' to 'active'

id                TEXT NOT NULL           -- PK post-migration
branch            TEXT UNIQUE             -- was PK, now unique index
                                          -- NULL-able for future Process support
kind              TEXT NOT NULL DEFAULT 'feature'   -- was workspace_type
status            TEXT NOT NULL DEFAULT 'active'    -- was 'idle'
feature_id        TEXT
spec_id           TEXT                    -- keep for alpha, revisit
release_id        TEXT
preset_id         TEXT                    -- was active_mode
providers         TEXT NOT NULL DEFAULT '[]'        -- was providers_json
worktree_path     TEXT                    -- presence = is_worktree true
                                          -- DROP is_worktree column
last_activated_at TEXT
context_hash      TEXT
config_generation INTEGER NOT NULL DEFAULT 0
compiled_at       TEXT                    -- DROP resolved_at, keep this
compile_error     TEXT
created_at        TEXT NOT NULL
updated_at        TEXT NOT NULL
```

#### `branch_context` table (branch ↔ entity link for git hook)
```sql
-- branch_context table
-- This is clean as-is. One note:
-- link_type 'feature' | 'spec' — consider whether 'workspace'
-- should be a link_type here too, since workspace now has its
-- own id separate from branch. The git hook probably wants to
-- resolve branch → workspace_id directly.
-- ADDCOLUMN: workspace_id TEXT for direct workspace resolution.

branch        TEXT PRIMARY KEY
workspace_id  TEXT                 -- ADD: direct workspace resolution
link_type     TEXT NOT NULL
link_id       TEXT NOT NULL
last_synced   TEXT NOT NULL
```

---

### Workspace Session

**Persistence:** SQLite only (project DB).

#### Rust types
```rust
// ============================================================
// WORKSPACE SESSION
// ============================================================
// workspace_branch: REMOVE — redundant, get via workspace_id join.
//   Only reason to keep is query performance. If needed,
//   make it a computed/cached field not a source of truth.
//
// status: promote to enum
//   'active' | 'ended' is too coarse.
//   Ended how? Cleanly, with error, interrupted?
//   Add: Error, Interrupted for ops center diagnostics.

enum SessionStatus {
    Active,
    Paused,
    Ended,       // clean completion
    Error,       // failed
    Interrupted, // process killed, connection lost etc
}

// mode_id → capability: SessionCapability
// Runtime toggle belongs on session, preset belongs on workspace.
// These are currently conflated via mode_id on both.

enum SessionCapability {
    Edit,     // full read/write — default
    Plan,     // read-only, generates specs/plans
    Review,   // read-only, audit focus  
}

// goal + summary → DEFERRED COLLAPSE
// Ideally goal = Spec.title, summary = Commit.body.
// For alpha: keep both fields. They're doing useful work
// even without formal Spec/Commit records surfaced in UI.
// Post-alpha: when Specs are surfaced, add spec_id to session
// and derive goal from spec. summary becomes commit body.
//
// compile_generation: ADD THIS FIELD.
// Staleness = session.compile_generation != workspace.config_generation
// This replaces the current compiled_at comparison which has
// timezone/precision edge cases.

struct WorkspaceSession {
    id: String,
    workspace_id: String,
    // workspace_branch: REMOVE or keep as denormalized cache only
    status: SessionStatus,           // was String
    capability: SessionCapability,   // was mode_id
    primary_provider: Option<String>,
    goal: Option<String>,            // keep for alpha
    summary: Option<String>,         // keep for alpha
    compile_generation: i64,         // ADD — staleness detection
    compile_error: Option<String>,
    // compiled_at: REMOVE from session — generation int is sufficient
    // updated_feature_ids: keep as JSON for alpha.
    //   Post-alpha: migrate to SessionFeature junction table.
    // updated_spec_ids: same — migrate to SessionSpec junction post-alpha.
    updated_feature_ids: Vec<String>,
    updated_spec_ids: Vec<String>,
    started_at: DateTime<Utc>,       // was String
    ended_at: Option<DateTime<Utc>>, // was String
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}```

---

### Agent Mode
<!--
============================================================
AGENT MODE → RENAME to WorkspacePreset
============================================================
Current AgentMode is doing two jobs:
  1. Workspace seed/template (applied at creation)
  2. Runtime capability config (active during session)

For alpha: keep as-is, rename to WorkspacePreset.
Post-alpha: split into:
  WorkspacePreset — creation template
  SessionProfile  — runtime capability bundle

hooks_json: this is where your compiled hook config lives.
This feeds into the ship-hook binary via settings.json output.
Make sure this is the source of truth for hook compilation,
not a secondary copy.

target_agents_json: good — this is your provider targeting.
Consider renaming to providers_json for consistency with
Workspace.providers field.

permissions_json: this becomes the seed for .ship/envelope.json
at workspace activation. Make that relationship explicit in code.

============================================================
AGENT ARTIFACT REGISTRY
============================================================
This is clean and well-designed. A few notes:

content_hash: this is your tamper detection primitive.
If hash doesn't match file on disk, artifact is dirty.
Make sure your compilation step checks this before
deciding whether to recompile. Avoid unnecessary recompiles.

kind field values — document these explicitly somewhere:
  'claude_md'    → CLAUDE.md
  'mcp_json'     → .mcp.json  
  'agents_md'    → AGENTS.md
  'envelope'     → .ship/envelope.json
  'hooks_config' → .claude/settings.json
  'gemini_hooks' → .gemini/settings.json
The registry is the authoritative list of what Ship owns on disk.
This is important for your security story — Ship knows exactly
what files it wrote and can detect external modification.
-->
---

### Event Log

**Persistence:** SQLite only (project DB). NDJSON is an export format, not the source of truth.

#### SQLite schema (`events` table — inferred from `events.rs`)
Columns include: `id, seq, actor, entity, action, entity_id, details_json, created_at`

#### KV State (general-purpose key-value store)
<!-- 
-- events table
-- This is your audit trail and ops center feed. Solid design.
-- 
-- seq: keep — ordered sequence is important for replay and
-- ops center chronological display. Make sure this is
-- autoincrement not application-assigned.
--
-- actor: document the actor vocabulary explicitly somewhere:
--   'user' | 'agent' | 'system' | 'hook' | 'compiler'
-- This becomes important for security audit — knowing whether
-- an action was human or agent-initiated is valuable.
--
-- entity + action: consider a compound index on (entity, action)
-- for ops center queries like "all tool-calls in this session"
-- or "all compile events for this workspace".
--
-- details_json: unstructured is fine. Consider versioning the
-- schema per entity/action pair as your event vocabulary grows.
-- A details_version column costs nothing and saves pain later.
--
-- NDJSON export: clean separation of concerns. Export is a
-- projection of this table, not the source. Document this
-- explicitly so future contributors don't accidentally treat
-- the export as authoritative.

id           TEXT PRIMARY KEY
seq          INTEGER NOT NULL    -- autoincrement, never application-assigned
actor        TEXT NOT NULL       -- 'user' | 'agent' | 'system' | 'hook' | 'compiler'
entity       TEXT NOT NULL       -- 'workspace' | 'session' | 'feature' | 'spec' etc
action       TEXT NOT NULL       -- 'created' | 'compiled' | 'tool-call' | 'status-changed' etc
entity_id    TEXT NOT NULL       -- nanoid of affected entity
details_json TEXT                -- nullable is fine, not every event needs details
created_at   TEXT NOT NULL

-- Recommended indexes:
-- CREATE INDEX idx_events_entity ON events(entity, action);
-- CREATE INDEX idx_events_entity_id ON events(entity_id, seq DESC);
-- CREATE INDEX idx_events_actor ON events(actor, seq DESC);
-- CREATE INDEX idx_events_seq ON events(seq DESC);  -- ops center feed


-- kv_state table
-- Clean general-purpose store. A few notes:
--
-- namespace vocabulary: document what namespaces exist.
-- Undocumented KV stores become mystery boxes.
-- Suggested namespaces:
--   'workspace:{id}'  → per-workspace runtime state
--   'session:{id}'    → per-session ephemeral state  
--   'process:{id}'    → per-process operational state (post-alpha)
--   'app'             → global app state
--   'compiler'        → compilation cache
--
-- updated_at as String: fine for KV, low stakes.
--
-- Consider a ttl_at column for ephemeral entries.
-- Session state that should auto-expire when session ends
-- currently requires manual cleanup. TTL handles this passively.
-- Optional, but useful when you have 50 concurrent sessions.

namespace   TEXT NOT NULL
key         TEXT NOT NULL
value_json  TEXT NOT NULL
updated_at  TEXT NOT NULL
ttl_at      TEXT            -- ADD: optional expiry for ephemeral state
PRIMARY KEY (namespace, key)
```

**Cross-entity relationships — this needs the most attention:**
```
-- CURRENT:
Feature ──── spec_id ────→ Spec       -- implies 1:1, actually M:M
Spec ─────── feature_id ─→ Feature    -- BIDIRECTIONAL — one of these is redundant
ADR ──────── spec_id ────→ Spec       -- implies 1:1, ADRs can span multiple specs
Workspace ── spec_id ────→ Spec       -- discussed above — remove for alpha
WorkspaceSession → Workspace (workspace_branch FK)  -- change to workspace_id FK

-- RECOMMENDED:
-- The bidirectional Feature↔Spec relationship is the main issue.
-- Pick one direction as canonical or use a junction table.
-- Given specs are "units of work" not "feature children",
-- the junction approach is more honest:



-- spec_feature (junction — deferred post-alpha)
spec_id     TEXT NOT NULL
feature_id  TEXT NOT NULL
PRIMARY KEY (spec_id, feature_id)

-- For alpha: keep spec_id on Feature as a "primary spec" 
-- shortcut. Document it as denormalized convenience, not
-- source of truth.

-- ADR → Spec relationship:
-- ADRs document decisions made during work on a spec.
-- But an ADR might be relevant to multiple specs.
-- spec_id on ADR is fine as "the spec that prompted this decision"
-- Just don't treat it as exclusive ownership.

-- branch_context:
-- ADD workspace_id column as discussed.
-- link_type + link_id is flexible but undiscoverable.
-- Document the full link_type vocabulary in a comment or
-- in your schema migrations.
-->

---

## Tauri Push Events (`ShipEvent`)

The UI receives these typed events from the backend file watcher:

```rust
enum ShipEvent {
    IssuesChanged,    // issue DB or virtual files changed
    SpecsChanged,     // spec files changed
    AdrsChanged,      // ADR files changed
    FeaturesChanged,  // feature files changed
    ReleasesChanged,  // release files changed
    ConfigChanged,    // ship.toml changed
    EventsChanged,    // new events ingested
    LogChanged,       // action log changed
    NotesChanged,     // note DB entries changed
    
```
<!-- ADD: workspace and session lifecycle events
        Currently missing — UI has no way to know when to
        refresh workspace state without polling.
        WorkspacesChanged,      // workspace compiled, status changed
        SessionsChanged,        // session started, ended, status changed
        ADD: ops center real-time feed
       Hook events come in via HTTP sidecar but need to
       reach the frontend. Either add specific events here
       or add a generic HookEvent(HookEventKind) variant.
        HookEvent(HookEventKind),  // tool-call, scope-violation, conflict etc
        
         ADD: compile state changes
         UI currently has no push signal for compilation completing.
         The "session context is stale" warning needs this.
        CompileStateChanged {
            workspace_id: String,
            generation: i64,
        },
    }
}
// Payload consideration:
// Current pattern: no payload, UI re-fetches.
// This is fine for most events but HookEvent and CompileStateChanged
// benefit from carrying minimal payload to avoid unnecessary
// round-trips. Especially at 50 concurrent sessions —
// you don't want 50 re-fetches on every tool call.
// 
// Suggested: keep no-payload pattern for entity changes,
// add minimal payload for high-frequency ops center events.
-->

Each event is `{ type: "issues-changed" }` on the TypeScript side (kebab-case tag).
No payload data — UI should re-fetch on receipt.

---

## Visual Regression Checklist

For each entity UI screen, verify these match the data model:

### List views
- [ ] Status badge uses correct enum values (not free-form strings)
- [ ] Sorted by `updated_at DESC` (matches DB default ordering)
- [ ] Optional fields (`priority`, `assignee`, `tags`) gracefully absent when null
- [ ] Cross-entity links (release_id, spec_id, feature_id) shown as resolved names, not raw IDs

### Detail / edit views
- [ ] All nullable `Option<>` fields handled — no empty string substituted for null
- [ ] `body` content from markdown file, not DB metadata
- [ ] `todos` and `criteria` only present on feature detail (not list)
- [ ] Feature doc is a separate entity — requires separate fetch (`get_feature_documentation`)
- [ ] Status transition validates against enum — no free-form status entry
- [ ] `release` tags field is NOT in DB — markdown frontmatter only

### Entity-specific
- [ ] **Issue:** `backlog | in-progress | blocked | done` exactly (hyphen, lowercase)
- [ ] **Feature:** `planned | in-progress | implemented | deprecated`
- [ ] **Spec:** `draft | active | archived`
- [ ] **ADR:** `proposed | accepted | rejected | superseded | deprecated`
- [ ] **Release:** `planned | active | shipped | archived`
- [ ] **Release:** `breaking_changes` come from child table, not body parsing
- [ ] **ADR:** `adr_option` table exists but is NOT in the Rust struct — don't try to render options
- [ ] **Note:** No markdown file — always fetched from SQLite
- [ ] **Workspace:** `branch` is the primary key, `id` was added in v2 (may equal branch for old rows)
