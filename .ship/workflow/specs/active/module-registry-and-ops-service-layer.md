+++
id = "sQ4R83kx"
title = "Module Registry and Ops Service Layer"
created = "2026-03-03T18:04:53.261542447+00:00"
updated = "2026-03-03T18:05:00.000000000+00:00"
author = ""
tags = []
+++

## Overview

MCP and CLI currently contain duplicated business logic — validation, event logging, edge
creation, status enforcement — that belongs in the runtime. Both transports reach directly
into entity-specific modules, bypassing any shared layer. This spec introduces an ops service
layer in `crates/runtime/src/ops/` that both MCP and CLI call. MCP and CLI become thin
transport shims: parse input, call ops, format output.

This is the alpha implementation of the Module Registry described in the runtime primitives
spec (yjC3XiJu). The full trait-based dynamic registry with `Box<dyn ShipModule>` is deferred
to the extensions SDK phase. For alpha, the ops layer achieves the same architectural
separation with idiomatic Rust and no complex generics.

---

## Current State

```
CLI command
  -> get_project_dir_cli()
  -> import entity-specific fn directly (e.g. create_feature from modules/project)
  -> format output
  -> log_action (sometimes, inconsistently)

MCP tool handler
  -> get_effective_project_dir()
  -> import same entity-specific fn directly
  -> format string response
  -> (no log_action call -- inconsistent with CLI)
```

Problems:
- Business logic split across CLI, MCP, and module crates with no single owner
- `log_action` called in CLI but not MCP (or vice versa) -- event log is inconsistent
- No single place to enforce invariants (e.g. a feature can only move to implemented
  if it has a branch set)
- Any new transport (Tauri commands, REST API, future) must re-implement the same logic
- Tests for business logic must go through CLI or MCP, not the runtime

---

## Target State

```
CLI command         MCP tool            Tauri command (future)
  -> parse args       -> deserialise      -> deserialise
       |                    |                    |
       +--------------------+--------------------+
                  runtime::ops::<entity>::<op>(dir, input)
                  |- validation
                  |- business rules / status machine
                  |- sub-entity creation
                  |- SQLite write (via module db layer)
                  |- edge creation (future-ready)
                  |- event log append
                  `- Result<Output>
       |                    |                    |
  -> format + print   -> format string     -> serialise response
```

---

## Ops Layer Structure

```
crates/runtime/src/ops/
  mod.rs          -- pub use all ops modules; common input/output types; ShipModule trait
  feature.rs      -- Feature CRUD + lifecycle ops
  adr.rs          -- ADR CRUD + status transitions
  issue.rs        -- Issue CRUD + priority/status ops
  spec.rs         -- Spec CRUD
  release.rs      -- Release CRUD
  note.rs         -- Note CRUD (project + user scope)
```

All ops functions take `ship_dir: &Path` as first argument. They return typed Results,
never strings. Formatting is the caller's responsibility.

---

## Per-Module Op Signatures

### feature.rs

```rust
pub struct CreateFeatureInput {
    pub title: String,
    pub content: Option<String>,
    pub release_id: Option<String>,
    pub spec_id: Option<String>,
    pub branch: Option<String>,
}

pub fn create(dir: &Path, input: CreateFeatureInput) -> OpsResult<FeatureEntry>;
pub fn list(dir: &Path, status: Option<FeatureStatus>) -> OpsResult<Vec<FeatureEntry>>;
pub fn get(dir: &Path, file_name: &str) -> OpsResult<FeatureEntry>;
pub fn update(dir: &Path, file_name: &str, content: &str) -> OpsResult<FeatureEntry>;
pub fn start(dir: &Path, file_name: &str, branch: &str) -> OpsResult<FeatureEntry>;
pub fn done(dir: &Path, file_name: &str) -> OpsResult<FeatureEntry>;
```

### adr.rs

```rust
pub struct CreateAdrInput {
    pub title: String,
    pub context: Option<String>,
    pub decision: String,
}

pub fn create(dir: &Path, input: CreateAdrInput) -> OpsResult<AdrEntry>;
pub fn list(dir: &Path) -> OpsResult<Vec<AdrEntry>>;
pub fn get(dir: &Path, id: &str) -> OpsResult<AdrEntry>;
pub fn update(dir: &Path, id: &str, adr: ADR) -> OpsResult<AdrEntry>;
pub fn move_status(dir: &Path, id: &str, status: AdrStatus) -> OpsResult<AdrEntry>;
```

### issue.rs

```rust
pub struct CreateIssueInput {
    pub title: String,
    pub description: String,
    pub status: Option<String>,
    pub priority: Option<IssuePriority>,
    pub feature_id: Option<String>,
    pub spec_id: Option<String>,
}

pub fn create(dir: &Path, input: CreateIssueInput) -> OpsResult<IssueEntry>;
pub fn list(dir: &Path, status: Option<&str>) -> OpsResult<Vec<IssueEntry>>;
pub fn get(dir: &Path, file_name: &str) -> OpsResult<IssueEntry>;
pub fn update(dir: &Path, file_name: &str, title: Option<&str>, description: Option<&str>) -> OpsResult<IssueEntry>;
pub fn move_status(dir: &Path, file_name: &str, from: &str, to: &str) -> OpsResult<IssueEntry>;
pub fn delete(dir: &Path, file_name: &str, status: &str) -> OpsResult<()>;
```

### spec.rs

```rust
pub struct CreateSpecInput {
    pub title: String,
    pub content: Option<String>,
    pub feature_id: Option<String>,
    pub release_id: Option<String>,
}

pub fn create(dir: &Path, input: CreateSpecInput) -> OpsResult<SpecEntry>;
pub fn list(dir: &Path) -> OpsResult<Vec<SpecEntry>>;
pub fn get(dir: &Path, file_name: &str) -> OpsResult<SpecEntry>;
pub fn update(dir: &Path, file_name: &str, content: &str) -> OpsResult<SpecEntry>;
```

### release.rs

```rust
pub struct CreateReleaseInput {
    pub version: String,
    pub content: Option<String>,
}

pub fn create(dir: &Path, input: CreateReleaseInput) -> OpsResult<ReleaseEntry>;
pub fn list(dir: &Path) -> OpsResult<Vec<ReleaseEntry>>;
pub fn get(dir: &Path, file_name: &str) -> OpsResult<ReleaseEntry>;
pub fn update(dir: &Path, file_name: &str, content: &str) -> OpsResult<ReleaseEntry>;
```

### note.rs

```rust
pub fn create(dir: &Path, scope: NoteScope, title: &str, content: Option<&str>) -> OpsResult<NoteEntry>;
pub fn list(dir: &Path, scope: NoteScope) -> OpsResult<Vec<NoteEntry>>;
pub fn get(dir: &Path, scope: NoteScope, file_name: &str) -> OpsResult<NoteEntry>;
pub fn update(dir: &Path, scope: NoteScope, file_name: &str, content: &str) -> OpsResult<NoteEntry>;
```

---

## What Moves Into Ops

### Event logging
Today: CLI calls `log_action` inconsistently. MCP often omits it.
After: ops functions call `runtime::append_event` on every write. Every transport gets
consistent event logging automatically.

### Validation
Today: minimal, scattered across module crud functions.
After: ops layer validates title non-empty, status transition validity, referenced IDs exist.
Returns `OpsError::Validation` with a human-readable message.

### Status machine enforcement
Today: `feature_start` and `feature_done` are standalone functions with no guards.
After: `ops::feature::start` requires current status == Planned or returns
`OpsError::InvalidTransition`. `ops::feature::done` requires InProgress.

### Edge creation (future-ready)
Today: relationships stored as scalar fields (spec_id, release_id).
After: ops layer is the single write point, so adding the edges table (runtime spec yjC3XiJu)
is a one-line change in ops rather than a patch across CLI and MCP.

---

## ShipModule Marker Trait

Defined now to establish the concept. Not yet used for dispatch.

```rust
// crates/runtime/src/ops/mod.rs

/// Marker trait for Shipwright module types.
/// Implemented by all first-party entity types.
/// Full dynamic registry deferred to extensions SDK.
pub trait ShipModule: Send + Sync + 'static {
    fn module_type_id() -> &'static str where Self: Sized;
}
```

Feature, ADR, Issue, Spec, Release, Note implement this trait.

---

## OpsError

```rust
#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Invalid status transition: {0} -> {1}")]
    InvalidTransition(String, String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub type OpsResult<T> = std::result::Result<T, OpsError>;
```

---

## CLI Before / After

Before:
```rust
Commands::Feature(FeatureCommands::Create { title, content, .. }) => {
    let project_dir = get_project_dir_cli()?;
    match create_feature(&project_dir, &title, &content.unwrap_or_default(), ..) {
        Ok(entry) => println!("Feature created: {}", entry.file_name),
        Err(e) => eprintln!("Error: {}", e),
    }
    log_action(&project_dir, "feature create", &title).ok();
}
```

After:
```rust
Commands::Feature(FeatureCommands::Create { title, content, release_id, spec_id, branch }) => {
    let dir = get_project_dir_cli()?;
    let entry = runtime::ops::feature::create(&dir, CreateFeatureInput {
        title, content, release_id, spec_id, branch,
    })?;
    println!("Feature created: {}", entry.file_name);
    // log_action gone -- ops::feature::create calls append_event internally
}
```

---

## MCP Before / After

Before:
```rust
async fn create_feature(&self, Parameters(req): Parameters<CreateFeatureRequest>) -> String {
    let project_dir = match self.get_effective_project_dir().await { .. };
    match create_feature(&project_dir, &req.title, ..) {
        Ok(entry) => format!("Created feature '{}'", entry.title),
        Err(e) => format!("Error: {}", e),
    }
}
```

After:
```rust
async fn create_feature(&self, Parameters(req): Parameters<CreateFeatureRequest>) -> String {
    let dir = match self.get_effective_project_dir().await { .. };
    match runtime::ops::feature::create(&dir, CreateFeatureInput {
        title: req.title,
        content: req.content,
        release_id: req.release_id,
        spec_id: req.spec_id,
        branch: req.branch,
    }) {
        Ok(entry) => format!("Created feature '{}' ({})", entry.title, entry.file_name),
        Err(e) => format!("Error: {}", e),
    }
}
```

---

## Migration Phases

### Phase 1 -- Feature (Day 1, proof of concept)
1. Create `crates/runtime/src/ops/mod.rs` with ShipModule trait, OpsError, OpsResult
2. Create `crates/runtime/src/ops/feature.rs` with all feature ops
3. Update CLI feature commands to call ops::feature
4. Update MCP feature tools to call ops::feature
5. Run e2e tests, fix regressions

Feature first: most complex lifecycle, most callers, most e2e coverage.

### Phase 2 -- Remaining modules (Day 2)
Roll out to: adr, issue, spec, release, note. Same pattern per module.

### Phase 3 -- Tauri alignment (follow-on)
Tauri commands currently call module functions directly. Update to call ops layer.
Tauri-specific return types (Specta bindings) wrap ops output.

---

## Test Strategy

### Unit tests in ops/*.rs
- create returns correct entry with all fields populated
- list filters by status correctly
- invalid status transitions return OpsError::InvalidTransition
- event appended after each write op

### Integration (existing e2e suite)
- All 16 branch_config tests pass
- workflow.rs e2e tests pass
- No behaviour change visible to CLI/MCP callers

---

## Acceptance Criteria

- `crates/runtime/src/ops/` exists with a module per entity type
- `ShipModule` marker trait defined, implemented by Feature, ADR, Issue, Spec, Release, Note
- `OpsError` and `OpsResult` used consistently across all ops
- All CLI entity commands call ops layer -- no direct module/project imports
- All MCP entity tools call ops layer -- no direct module/project imports
- Event log entry created by ops layer for every write -- not by CLI or MCP
- `log_action` calls in CLI removed (replaced by ops-internal append_event)
- All existing e2e and unit tests pass without modification
- Tauri commands still compile (may call module functions directly until Phase 3)

---

## Files Touched

```
crates/runtime/src/ops/mod.rs          (new)
crates/runtime/src/ops/feature.rs      (new)
crates/runtime/src/ops/adr.rs          (new)
crates/runtime/src/ops/issue.rs        (new)
crates/runtime/src/ops/spec.rs         (new)
crates/runtime/src/ops/release.rs      (new)
crates/runtime/src/ops/note.rs         (new)
crates/runtime/src/lib.rs              (pub mod ops)
crates/cli/src/lib.rs                  (all entity commands -> ops)
crates/mcp/src/lib.rs                  (all entity tools -> ops)
```
