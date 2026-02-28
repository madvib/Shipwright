+++
id = "DFgCnu7y"
title = "Shipwright Data Model — Canonical Entity Definitions"
created = "2026-02-28T13:54:19.042653854Z"
updated = "2026-02-28T13:54:19.042653854Z"
tags = []
+++

# Shipwright Data Model — Canonical Entity Definitions

## Purpose

Define the canonical data model for all Shipwright entities across the full stack:
runtime (Rust structs), CLI, MCP tools, and Tauri/UI bindings. This spec governs
field names, types, reference conventions, and persistence strategy.

---

## Core Conventions

### Identity
- Every entity has a `id: String` (UUID v4), generated at creation, **never changes**
- Cross-entity references use UUID (`feature_id`, `release_id`, `spec_id`) — never filenames
- Filenames are human-readable handles only: `{YYYYMMDD}-{slug}.md`
  - Example: `20260228-auth-flow.md`
  - Slug derived from title at creation time, may be renamed freely
  - UI always reads `metadata.title`, never parses filenames

### Timestamps
- `created_at: DateTime<Utc>` — set once at creation
- `updated_at: DateTime<Utc>` — updated on every write
- Serialized as RFC3339 strings in frontmatter/JSON

### Status fields
- Always typed enums in Rust, serialized as kebab-case strings
- TypeScript bindings get string literal union types via Specta

### Persistence layers
- **Frontmatter** (TOML between `+++`): canonical for document entities with meaningful body text
- **SQLite**: indexes, relationships, event log, computed/resolved state
- **Filesystem structure**: agent config directory (skills/, rules/, mcp.toml, permissions.toml)
- NDJSON event log is deprecated in favor of SQLite `events` table; kept as export format only

---

## Module: Project

### Vision

Simple document — no structured fields beyond metadata. Body is free-form markdown.
Version history via git. No migration needed.

```
.ship/project/vision.md
```

```rust
pub struct VisionMetadata {
    pub id: String,
    pub title: String,           // default: "Product Vision"
    pub updated_at: DateTime<Utc>,
}
// body: free-form markdown — mission, principles, personas, success metrics
```

### Feature

```rust
pub struct FeatureMetadata {
    pub id: String,
    pub title: String,
    pub status: FeatureStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub description: Option<String>,   // one-paragraph summary for catalog/docs
    pub owner: Option<String>,
    pub release_id: Option<String>,    // UUID ref → Release
    pub spec_id: Option<String>,       // UUID ref → Spec
    pub branch: Option<String>,        // git branch name (set by feature start)
    pub supersedes_id: Option<String>, // UUID ref → Feature this replaces
    pub tags: Vec<String>,
    pub adr_ids: Vec<String>,          // UUID refs → ADRs
    pub agent: Option<FeatureAgentOverride>,
}

pub enum FeatureStatus {
    Planned,     // "planned"
    InProgress,  // "in-progress"   (alias: "active")
    Implemented, // "implemented"   (alias: "complete")
    Deprecated,  // "deprecated"    (alias: "archived")
}

// Thin override — only fields that differ from project/mode defaults
pub struct FeatureAgentOverride {
    pub model: Option<String>,
    pub max_cost_per_session: Option<f64>,
    pub providers: Vec<String>,             // empty = inherit from project
    pub mcp_server_ids: Vec<String>,        // empty = use all project servers
    pub skill_ids: Vec<String>,             // empty = use all project skills
}
```

### Release

```rust
pub struct ReleaseMetadata {
    pub id: String,
    pub version: String,                   // e.g., "v0.1.0-alpha"
    pub status: ReleaseStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub target_date: Option<NaiveDate>,
    pub feature_ids: Vec<String>,          // UUID refs → Features
    pub adr_ids: Vec<String>,              // UUID refs → ADRs
    pub breaking_changes: Vec<String>,     // list of breaking change descriptions
    pub tags: Vec<String>,
}

pub enum ReleaseStatus {
    Planned,    // "planned"
    Active,     // "active"
    Shipped,    // "shipped"
    Archived,   // "archived"
}
```

### ADR (Architecture Decision Record)

```rust
pub struct AdrMetadata {
    pub id: String,
    pub title: String,
    pub status: AdrStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub spec_id: Option<String>,          // UUID ref → related Spec
    pub supersedes_id: Option<String>,    // UUID ref → ADR this replaces
    pub tags: Vec<String>,
}

pub enum AdrStatus {
    Proposed,    // "proposed"
    Accepted,    // "accepted"
    Rejected,    // "rejected"
    Superseded,  // "superseded"
    Deprecated,  // "deprecated"
}
// Body sections (free-form markdown): Context, Decision, Consequences, Alternatives
```

---

## Module: Workflow

### Spec

```rust
pub struct SpecMetadata {
    pub id: String,
    pub title: String,
    pub status: SpecStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub author: Option<String>,
    pub feature_id: Option<String>,      // UUID ref → Feature
    pub release_id: Option<String>,      // UUID ref → Release
    pub tags: Vec<String>,
}

pub enum SpecStatus {
    Draft,    // "draft"
    Active,   // "active"
    Archived, // "archived"
}
```

### Issue

```rust
pub struct IssueMetadata {
    pub id: String,
    pub title: String,
    pub priority: IssuePriority,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub assignee: Option<String>,
    pub spec_id: Option<String>,         // UUID ref → Spec
    pub feature_id: Option<String>,      // UUID ref → Feature
    pub tags: Vec<String>,
    pub links: Vec<IssueLink>,
}

pub enum IssuePriority {
    Critical, // "critical"
    High,     // "high"
    Medium,   // "medium"  (default)
    Low,      // "low"
}

pub struct IssueLink {
    pub link_type: String,               // "blocks", "blocked-by", "related"
    pub target_id: String,               // UUID of linked issue
}

// Status is directory-based: backlog / in-progress / blocked / done
// IssueEntry carries status: String derived from directory
```

### Note

```rust
pub struct NoteMetadata {
    pub id: String,
    pub title: String,
    pub scope: NoteScope,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
}

pub enum NoteScope {
    Project, // ".ship/project/notes/"
    User,    // "~/.ship/notes/"
}
```

### Workspace (Branch Session)

Not a document — stored in SQLite only. Represents the active branch context.
Created/updated automatically on git branch checkout via the post-checkout hook.

```rust
pub struct Workspace {
    pub id: String,                       // UUID
    pub branch: String,                   // git branch name
    pub feature_id: Option<String>,       // UUID ref → Feature (if branch is linked)
    pub spec_id: Option<String>,          // UUID ref → Spec (if branch is linked)
    pub active_mode: Option<String>,      // mode id from ship.toml
    pub providers: Vec<String>,           // resolved providers for this branch
    pub resolved_at: DateTime<Utc>,       // when agent config was last resolved
    pub is_worktree: bool,               // true if this is a git worktree
    pub worktree_path: Option<String>,   // absolute path if worktree
}
```

---

## Module: Agents

The agents module is **filesystem as configuration**. No single config file — the directory
tree is the config. `AgentConfig` is the in-memory resolved representation.

```
.ship/agents/
  skills/          ← Skill documents (agentskills.io spec)
  rules/           ← Rule .md files (always active if present in directory)
  mcp.toml         ← MCP server registry
  permissions.toml ← Shipwright permission model
  modes/           ← Mode config TOML files
```

### Skill

Follows agentskills.io specification. Frontmatter + markdown body.

```rust
pub struct Skill {
    pub id: String,              // filename without .md — stable identifier
    pub name: String,            // display name
    pub description: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    pub source: SkillSource,
    pub content: String,         // markdown body — may contain $ARGUMENTS
}

pub enum SkillSource {
    Custom,       // "custom"    — written by user
    AiGenerated,  // "ai-generated"
    Community,    // "community" — from agentskills.io registry
    Imported,     // "imported"  — from external source
}
```

TODO: Align struct fields to agentskills.io/specification before implementation.

### Rule

Simple markdown file in `agents/rules/`. No frontmatter needed — filename is the id,
first `# Heading` is the name, rest is content.

```rust
pub struct Rule {
    pub id: String,              // filename without .md
    pub name: String,            // from first # heading, or filename
    pub content: String,         // full markdown body
}
// Always active if present in the directory. Activation = file exists.
```

### Permissions

```toml
# .ship/agents/permissions.toml

[tools]
allow = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]
deny  = []

[filesystem]
allow = ["src/", "tests/", ".ship/"]
deny  = [".env", "*.pem", "secrets/"]

[commands]
allow = ["cargo", "git", "just", "npm"]
deny  = []

[network]
# "none" | "localhost" | "allow-list" | "unrestricted"
policy = "localhost"
allow_hosts = []

[agent]
max_cost_per_session = 5.00
max_turns = 50
require_confirmation = ["git_push"]
```

```rust
pub struct Permissions {
    pub tools: ToolPermissions,
    pub filesystem: FsPermissions,
    pub commands: CommandPermissions,
    pub network: NetworkPermissions,
    pub agent: AgentLimits,
}

pub struct ToolPermissions {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

pub struct FsPermissions {
    pub allow: Vec<String>,      // glob patterns
    pub deny: Vec<String>,       // glob patterns
}

pub struct CommandPermissions {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
}

pub struct NetworkPermissions {
    pub policy: NetworkPolicy,
    pub allow_hosts: Vec<String>,
}

pub enum NetworkPolicy {
    None,         // "none"
    Localhost,    // "localhost"
    AllowList,    // "allow-list"  — requires allow_hosts
    Unrestricted, // "unrestricted"
}

pub struct AgentLimits {
    pub max_cost_per_session: Option<f64>,
    pub max_turns: Option<u32>,
    pub require_confirmation: Vec<String>,
}
```

### MCP Registry

```toml
# .ship/agents/mcp.toml

[[servers]]
id = "ship"
name = "Shipwright"
command = "ship-mcp"
args = []
type = "stdio"     # "stdio" | "sse" | "http"

[[servers]]
id = "filesystem"
name = "Filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "."]
type = "stdio"
```

### AgentConfig (resolved, in-memory only)

Not stored as a file. Computed from project config + active mode + feature override.
Snapshot stored in Workspace (SQLite) for UI display.

```rust
pub struct AgentConfig {
    pub providers: Vec<String>,
    pub model: Option<String>,
    pub max_cost_per_session: Option<f64>,
    pub max_turns: Option<u32>,
    pub mcp_servers: Vec<McpServerConfig>,
    pub skills: Vec<Skill>,
    pub rules: Vec<Rule>,
    pub permissions: Permissions,
    pub active_mode: Option<String>,
}

// Resolution order (highest wins):
// 1. Project defaults (ship.toml + agents/ directory)
// 2. Active mode overrides (agents/modes/<id>.toml)
// 3. Feature [agent] block (thin overrides only)
pub fn resolve_agent_config(
    project: &ProjectConfig,
    mode: Option<&ModeConfig>,
    feature_override: Option<&FeatureAgentOverride>,
    ship_dir: &Path,
) -> Result<AgentConfig>;
```

---

## Events (SQLite)

Replace NDJSON append-only log with SQLite `events` table. NDJSON kept as export format.

```sql
CREATE TABLE events (
    seq         INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp   TEXT NOT NULL,    -- RFC3339
    actor       TEXT NOT NULL,    -- "user", "agent", "hook", "mcp", etc.
    entity_type TEXT NOT NULL,    -- EventEntity as string
    action      TEXT NOT NULL,    -- EventAction as string
    entity_id   TEXT,             -- UUID of affected entity (when known)
    subject     TEXT NOT NULL,    -- human-readable subject
    details     TEXT              -- optional JSON blob
);
CREATE INDEX events_entity ON events(entity_type, entity_id);
CREATE INDEX events_timestamp ON events(timestamp);
```

```rust
pub struct EventRecord {
    pub seq: u64,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub entity_type: EventEntity,
    pub action: EventAction,
    pub entity_id: Option<String>,  // NEW — UUID for cross-referencing
    pub subject: String,
    pub details: Option<serde_json::Value>, // structured, not just a string
}
```

---

## Tauri/UI Exposure

### Typed Specta Events (Rust → TypeScript push)

Replace raw string `app_handle.emit("ship://...")` with typed events:

```rust
#[derive(Clone, Serialize, Type, tauri_specta::Event)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ShipEvent {
    WorkspaceChanged { workspace: WorkspaceInfo },
    IssuesMoved { from: String, to: String, issue_id: String },
    FeatureUpdated { feature_id: String },
    AgentConfigChanged,
    ProjectChanged,
}
```

### Status enums in UI

All status fields must use the Rust enum type (not `String`) so Specta generates
typed string literals in TypeScript:

```typescript
// Generated by Specta:
type FeatureStatus = "planned" | "in-progress" | "implemented" | "deprecated"
type IssueStatus = "backlog" | "in-progress" | "blocked" | "done"
type AdrStatus = "proposed" | "accepted" | "rejected" | "superseded" | "deprecated"
```

### Tauri commands to add

Missing from current surface:
- `get_vision_cmd / update_vision_cmd`
- `list_notes_cmd / get_note_cmd / create_note_cmd / update_note_cmd / delete_note_cmd`
- `list_rules_cmd / get_rule_cmd / create_rule_cmd / update_rule_cmd / delete_rule_cmd`
- `get_workspace_cmd` — current branch Workspace from SQLite
- `get_permissions_cmd / save_permissions_cmd`
- `get_agent_config_cmd` — resolved AgentConfig for current workspace
- `list_mcp_servers_from_file_cmd` — read agents/mcp.toml (distinct from ship.toml servers)

---

## Filename Convention

Format: `{YYYYMMDD}-{slug}.md`

- Slug derived from title at creation: lowercase, spaces→hyphens, special chars stripped
- Example: title "Auth Flow Feature" → `20260228-auth-flow-feature.md`
- Files may be renamed; identity is always the `id` UUID in frontmatter
- UI never displays or parses filenames directly

### Migration path for existing files

Non-breaking: existing files without date prefix continue to work.
`init_project` on existing projects does not rename files.
New files created after this spec follow the convention.

---

## Open Questions

- [ ] Align `Skill` struct fields to agentskills.io/specification before implementation
- [ ] Decide: does `agents/mcp.toml` replace `ship.toml` MCP server list, or supplement?
- [ ] Workspace `resolved_agent_config`: store full JSON blob in SQLite or just the key IDs?
- [ ] Events migration: how to preserve existing NDJSON history on first run?
