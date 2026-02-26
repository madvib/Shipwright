# Shipwright — Refactor Specification

**Version:** 0.1  
**Status:** Active  
**Last Updated:** 2026-02-22  
**Covers:** SQLite runtime layer, Git module, Modes vs branch config, Agent config UI, Notes, Configurable workflows, Global AI CLI config

---

## Overview

This spec covers four interconnected changes to the Shipwright architecture:

1. **SQLite runtime layer** — replace ad-hoc file state with a proper local database
2. **Git module** — hooks, worktrees, branch-scoped agent config generation
3. **Modes vs branch config** — clean separation, clear ownership
4. **Agent config UI** — library-driven selection, no magic strings
5. **Notes feature** — new document type in `plans/`
6. **Configurable workflows** — teams decide their own loop
7. **Global AI CLI config** — when and how Shipwright touches `~/.claude/` etc.

---

## 1. SQLite Runtime Layer

### What Goes Where

The discipline is strict. If a human would ever want to read, diff, or commit it — markdown. If it's ephemeral, concurrent, or machine-generated churn — SQLite.

```
Markdown (git-tracked)          SQLite (gitignored)
──────────────────────          ──────────────────────────────
Specs                           Active agent sessions
Issues                          Worktree registry + status
ADRs                            Orchestration locks
Notes                           Task queues
Modes (config)                  MCP connection state
Feature branch config           Branch context cache
Templates                       Real-time agent progress
Action log (human)              Mode override state
                                Hook execution log
```

### Schema

```sql
-- migrations/001_initial.sql

-- Worktree registry
CREATE TABLE worktrees (
    id              TEXT PRIMARY KEY,        -- "feat-auth-a3f2"
    path            TEXT NOT NULL UNIQUE,    -- absolute path
    branch          TEXT NOT NULL,
    parent_branch   TEXT NOT NULL,
    spec_id         TEXT,                    -- linked spec
    agent           TEXT,                    -- "claude" | "gemini" | "codex"
    status          TEXT NOT NULL DEFAULT 'active',  -- active | merged | archived
    spawned_by      TEXT NOT NULL DEFAULT 'human',
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

-- Agent sessions
CREATE TABLE agent_sessions (
    id              TEXT PRIMARY KEY,
    worktree_id     TEXT REFERENCES worktrees(id),
    agent           TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'running',
    started_at      TEXT NOT NULL,
    ended_at        TEXT,
    summary         TEXT,
    issues_worked   TEXT,                    -- JSON array of issue IDs
    adrs_filed      TEXT                     -- JSON array of ADR IDs
);

-- Orchestration locks (prevents two agents touching same issue)
CREATE TABLE locks (
    resource_id     TEXT PRIMARY KEY,        -- "issue:issue-001"
    worktree_id     TEXT NOT NULL,
    session_id      TEXT NOT NULL,
    acquired_at     TEXT NOT NULL,
    expires_at      TEXT NOT NULL
);

-- Task queue (for future agent orchestration)
CREATE TABLE tasks (
    id              TEXT PRIMARY KEY,
    type            TEXT NOT NULL,
    payload         TEXT NOT NULL,           -- JSON
    status          TEXT NOT NULL DEFAULT 'pending',
    worktree_id     TEXT,
    created_at      TEXT NOT NULL,
    started_at      TEXT,
    completed_at    TEXT,
    error           TEXT
);

-- Branch context cache (invalidated on checkout)
CREATE TABLE branch_context (
    branch          TEXT PRIMARY KEY,
    spec_id         TEXT,
    open_issues     TEXT,                    -- JSON array
    generated_at    TEXT NOT NULL,
    claude_md_hash  TEXT,                    -- detect if regeneration needed
    mcp_config_hash TEXT
);

-- MCP connection state
CREATE TABLE mcp_connections (
    id              TEXT PRIMARY KEY,
    server_id       TEXT NOT NULL,
    worktree_id     TEXT,                    -- null = global
    status          TEXT NOT NULL,           -- connected | disconnected | error
    connected_at    TEXT,
    last_ping       TEXT,
    error           TEXT
);

-- Mode override (when user manually switches mode)
CREATE TABLE mode_state (
    scope           TEXT PRIMARY KEY,        -- "global" | worktree path
    mode_id         TEXT NOT NULL,
    switched_at     TEXT NOT NULL,
    switched_by     TEXT NOT NULL DEFAULT 'human'
);
```

### Access Pattern — Repository Trait Per Domain

All SQLite access goes through typed repository structs. No raw SQL outside of `crates/runtime/src/db/`. No SQL in command handlers. No SQL in modules.

```rust
// crates/runtime/src/db/mod.rs

pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub fn worktrees(&self) -> WorktreeRepo { WorktreeRepo::new(&self.pool) }
    pub fn sessions(&self) -> SessionRepo { SessionRepo::new(&self.pool) }
    pub fn locks(&self) -> LockRepo { LockRepo::new(&self.pool) }
    pub fn tasks(&self) -> TaskRepo { TaskRepo::new(&self.pool) }
    pub fn branch_context(&self) -> BranchContextRepo { BranchContextRepo::new(&self.pool) }
    pub fn mcp_connections(&self) -> McpConnectionRepo { McpConnectionRepo::new(&self.pool) }
    pub fn mode_state(&self) -> ModeStateRepo { ModeStateRepo::new(&self.pool) }
}

// Example — WorktreeRepo exposes only what's needed
pub struct WorktreeRepo<'a> { pool: &'a SqlitePool }

impl WorktreeRepo<'_> {
    pub async fn register(&self, w: NewWorktree) -> Result<Worktree>
    pub async fn get(&self, id: &str) -> Result<Option<Worktree>>
    pub async fn get_by_path(&self, path: &Path) -> Result<Option<Worktree>>
    pub async fn list_active(&self) -> Result<Vec<Worktree>>
    pub async fn update_status(&self, id: &str, status: WorktreeStatus) -> Result<()>
    pub async fn archive(&self, id: &str) -> Result<()>
}
```

**Migration management:** `sqlx::migrate!("./migrations")` embedded in the binary. `Db::open()` runs pending migrations automatically. Schema version tracked in `_sqlx_migrations` table. Zero user interaction required.

**Two SQLite files:**
- `~/.shipwright/shipwright.db` — global state (registered projects, global mode, global MCP connections)
- `.ship/ship.db` — project state (worktrees, sessions, locks, branch context) — gitignored

---

## 2. Git Module

### Scope — Workflow layer, not a Git GUI

The Git module owns the workflow layer on top of git. It does not provide a visual git history, diff viewer, or branch graph. It is not competing with Tower or GitKraken.

What it owns:
- Hook installation and management
- Worktree lifecycle (create, register, archive)
- Branch-scoped config generation (CLAUDE.md, .mcp.json)
- Commit message injection from issue context
- Post-merge archival of completed issues

```
crates/modules/git/
├── src/
│   ├── lib.rs               # ShipwrightModule impl
│   ├── hooks.rs             # install/manage/dispatch hooks
│   ├── worktrees.rs         # create, register, list, archive
│   ├── context.rs           # branch → spec → CLAUDE.md + .mcp.json
│   ├── commit.rs            # commit message from active issue
│   └── archival.rs          # post-merge issue archival
└── ui/
    ├── WorktreePanel.tsx    # active worktrees, agent status, actions
    └── CommitFlow.tsx       # guided commit with issue context
```

### Hook Installation

`ship init` installs hooks into `.git/hooks/post-checkout.d/` (not `.git/hooks/post-checkout` directly) to coexist with other hook managers (husky, lefthook, Claude Code, Codex).

```bash
# .git/hooks/post-checkout
#!/bin/sh
# Shipwright hook dispatcher — do not edit directly
run_hooks() {
    dir=".git/hooks/$1.d"
    [ -d "$dir" ] || return 0
    for hook in "$dir"/*; do
        [ -x "$hook" ] && "$hook" "$@"
    done
}
run_hooks post-checkout "$@"
```

Hooks installed by Shipwright:

```
.git/hooks/post-checkout.d/shipwright     # generate CLAUDE.md + .mcp.json, update SQLite
.git/hooks/pre-commit.d/shipwright        # enforce gitignore rules, validate .ship consistency
.git/hooks/prepare-commit-msg.d/shipwright # inject issue context into commit message
.git/hooks/post-merge.d/shipwright        # archive completed issues, update feature status
```

Hook behavior is configurable per project:

```toml
# .ship/config.toml

[git.hooks]
post_checkout.generate_context = true      # CLAUDE.md + .mcp.json on branch switch
post_checkout.update_mode = false          # don't auto-switch Shipwright UI mode
pre_commit.enforce_gitignore = true        # block commits of generated files
pre_commit.validate_consistency = true     # check orphaned issues, broken spec links
prepare_commit_msg.inject_issue = true     # pre-populate from active issue
post_merge.archive_completed = true        # move closed issues to archived/
post_merge.generate_summary = false        # session summary on merge (V2)
```

### What Gets Gitignored — Enforced by Shipwright

Shipwright enforces that generated and runtime files are never committed. The `pre-commit` hook checks and warns (or blocks, configurable) if these appear staged:

```
# .gitignore entries Shipwright ensures exist

# Shipwright runtime — never commit
.ship/ship.db
.ship/ship.db-shm
.ship/ship.db-wal

# Generated agent config — never commit to main
CLAUDE.md
GEMINI.md
.mcp.json
.gemini/settings.json
.codex/config.toml
.claude/

# Worktree identity
.ship/worktree.toml
```

**The rule on `.claude/`, `.gemini/`, `.codex/` in Shipwright projects:**

These directories and files are **generated by Shipwright** from `.ship/config.toml` and the active feature spec. They must not be manually maintained or committed in a Shipwright-managed project because:

1. They will be overwritten on next branch checkout
2. Committing them to main means all branches share the same agent config — defeating the entire feature-branch config system
3. They may contain tokens or env var references that shouldn't be in git history

Shipwright's `pre-commit` hook blocks staging of these paths by default. The user can override this per-project in `.ship/config.toml` if they have a specific reason, but the default is strict. On `ship init` Shipwright appends these paths to `.gitignore` automatically.

### Branch-Scoped Context Generation

The `post-checkout` hook triggers `shipwright git generate-context` which:

1. Reads the current branch name
2. Queries SQLite for cached branch context (checks hash, skips if unchanged)
3. Finds the linked spec (by branch naming convention or explicit link in `.ship/config.toml`)
4. Reads open issues on this branch
5. Reads relevant ADRs
6. Generates CLAUDE.md
7. Generates tool-specific config (.mcp.json, .gemini/settings.json, .codex/config.toml)
8. Updates SQLite branch_context cache

```rust
// crates/modules/git/src/context.rs

pub async fn generate_branch_context(
    branch: &str,
    project: &ProjectContext,
    db: &Db,
) -> Result<GeneratedContext> {

    // Find linked spec
    let spec = find_spec_for_branch(branch, project).await?;

    // Collect context
    let open_issues = project.documents
        .list("issue", DocumentFilter::status_not("archived"))
        .await?;

    let relevant_adrs = if let Some(ref spec) = spec {
        project.relationships
            .get_related(DocRef::new("spec", &spec.id), "adr")
            .await?
    } else {
        vec![]
    };

    // Build CLAUDE.md
    let claude_md = ClaudeMdBuilder::new()
        .project_name(&project.name)
        .spec(spec.as_ref())
        .open_issues(&open_issues)
        .adrs(&relevant_adrs)
        .shipwright_mcp_hint()           // always include Shipwright MCP instructions
        .workflow_config(&project.workflow)
        .build();

    // Write generated files (all gitignored)
    write_gitignored(&project.root.join("CLAUDE.md"), &claude_md)?;

    // Generate MCP configs for managed tools
    let mcp_servers = resolve_mcp_for_branch(branch, spec.as_ref(), project)?;
    for formatter in project.managed_formatters() {
        let config = formatter.format(mcp_servers.clone())?;
        write_gitignored(&formatter.project_config_path(&project.root), &config)?;
    }

    // Cache in SQLite
    db.branch_context().upsert(BranchContext {
        branch: branch.to_string(),
        spec_id: spec.map(|s| s.id),
        open_issues: open_issues.iter().map(|i| i.id.clone()).collect(),
        generated_at: Utc::now(),
        claude_md_hash: hash(&claude_md),
        mcp_config_hash: hash(&mcp_servers),
    }).await?;

    Ok(GeneratedContext { claude_md, mcp_servers })
}
```

### Branch Naming Convention

Shipwright links branches to specs via naming convention (overridable):

```toml
# .ship/config.toml

[git]
branch_prefix = "feature/"     # feature/auth → looks for spec named "auth" or tagged "auth"
spec_link_strategy = "name"    # "name" | "explicit" | "tag"
```

Explicit linking (when naming doesn't match):

```toml
# .ship/config.toml

[git.branch_specs]
"feature/redesign-2026" = "spec-047"
"hotfix/auth-regression" = "spec-023"
```

### Worktree Lifecycle

```rust
// crates/modules/git/src/worktrees.rs

pub async fn create_worktree(
    spec_id: &str,
    agent: AgentCli,
    project: &ProjectContext,
    db: &Db,
) -> Result<Worktree> {

    // Derive branch name from spec
    let spec = project.documents.get("spec", spec_id).await?
        .ok_or(Error::SpecNotFound)?;
    let branch = derive_branch_name(&spec, &project.config.git)?;

    // Create git worktree
    let worktree_path = project.root
        .parent().unwrap()
        .join(format!("{}-worktree-{}", project.name, &branch[..8]));

    Command::new("git")
        .args(["worktree", "add", "-b", &branch, worktree_path.to_str().unwrap()])
        .current_dir(&project.root)
        .output()?;

    // Register in SQLite
    let worktree = db.worktrees().register(NewWorktree {
        id: generate_id(),
        path: worktree_path.clone(),
        branch: branch.clone(),
        parent_branch: current_branch(&project.root)?,
        spec_id: Some(spec_id.to_string()),
        agent: agent.id().to_string(),
        spawned_by: "human".to_string(),
    }).await?;

    // Generate context in the new worktree
    generate_branch_context(&branch, &project.with_root(worktree_path), db).await?;

    Ok(worktree)
}
```

---

## 3. Modes vs Branch Config — Clean Separation

### Modes (manual, UI/workflow intent)

Modes are manually switched by the human. They control:
- Shipwright UI layout and default view
- Which Shipwright MCP tools are surfaced to agents
- AI conversation panel context scope
- Nothing about external MCP servers (that's branch config)

Stored in: SQLite `mode_state` table (current mode), `.ship/modes/` (mode definitions, git-tracked, team-shared)

```toml
# .ship/modes/planning.toml
id = "planning"
name = "Planning"
ui_layout = "spec-editor"
shipwright_tools = ["list_specs", "create_spec", "list_issues", "create_issue"]
ai_context_files = ["AGENTS.md", "specs/"]
color = "#6366f1"
```

Modes do NOT contain:
- External MCP server definitions
- Model selection
- Max cost
- Skills
- Prompts

All of that belongs to branch/feature config.

### Branch Config (automatic, agent environment)

Branch config is derived from the spec and generated on checkout. It controls the agent's environment — never the human's UI. Stored in: generated gitignored files + SQLite `branch_context` cache.

Branch config IS the thing that configures:
- Which external MCP servers are active
- Which skills are loaded
- Which model to use
- Max cost limit
- Which prompts/instructions to include
- Which context files to load beyond the spec

### When Does Shipwright Touch Global AI CLI Config?

Global config (`~/.claude/`, `~/.gemini/`, `~/.codex/`) is only written when the user explicitly sets global defaults in Shipwright. It is **never** written automatically as a side effect of branch operations.

Three scenarios:

**1. Global defaults (explicit user action)**
User opens Settings → Agents → Global Defaults and sets base MCP servers that should be present everywhere. Shipwright writes to `~/.claude/mcp.json` etc. This is intentional and shown clearly in the UI.

**2. Project-managed (explicit user action)**
User opens Settings → Agents → for a specific project and clicks "Manage Claude Code for this project." Shipwright starts writing `.mcp.json` in the project root on every branch checkout. Still explicit, never automatic.

**3. Branch-scoped (automatic)**
Always automatic, always gitignored, never touches global config. This is the default behavior — no global config pollution.

The mental model:

```
Global config       → explicit, user-managed, touches ~/.claude/ etc.
Project config      → explicit, user-managed, generates on checkout
Branch config       → automatic, derived from spec, always gitignored
```

---

## 4. Agent Config UI — No Magic Strings

### The Library Concept

Every configurable value in branch/feature config is selected from a library, not typed. The library has four sections:

**MCP Servers** — all servers the user has ever defined or imported, plus well-known suggestions. Add once, reuse everywhere.

**Skills** — reusable instruction sets. In Claude terms, these are prompt fragments that get injected into CLAUDE.md. Versioned, shareable, community-sourced via marketplace.

**Prompts** — saved prompt templates for specific workflows. "When working on auth, always check for OWASP top 10." Scoped to features or global.

**Models** — available models from active AI providers. Populated by querying each installed CLI. No hardcoded model list.

### Feature Branch Config UI

When a feature branch is created (or when the user clicks "Configure" on an existing branch), a panel opens:

```
Feature: Authentication Redesign
Branch:  feature/auth

MCP SERVERS ──────────────────────────────────────────────────
  ✓ shipwright        always included
  ✓ github            [×]
  ✓ postgres          [×]
  + Add server        [Select from library ▾]
                         ┌─────────────────────────────┐
                         │ 🔍 Search servers...         │
                         │ ─────────────────────────── │
                         │ ★ github     execution       │
                         │ ★ postgres   backend         │
                         │   figma      planning        │
                         │   linear     planning        │
                         │   filesystem all             │
                         │ ─────────────────────────── │
                         │ + Define new server          │
                         └─────────────────────────────┘

SKILLS ────────────────────────────────────────────────────────
  ✓ shipwright-workflow     always included
  ✓ nextjs-conventions      [×]
  + Add skill           [Select from library ▾]

CONTEXT ───────────────────────────────────────────────────────
  Auto-included:
  ✓ specs/spec-023-auth.md  (linked spec)
  ✓ issues/in-progress/     (open issues)
  ✓ adrs/                   (relevant ADRs)

  Additional:
  + Add files           [Browse project ▾]

MODEL ─────────────────────────────────────────────────────────
  claude-opus-4-5           [Change ▾]
                               ┌─────────────────────────────┐
                               │ CLAUDE                       │
                               │   claude-opus-4-5      ●    │
                               │   claude-sonnet-4-5         │
                               │   claude-haiku-4-5          │
                               │ GEMINI                       │
                               │   gemini-2.0-flash          │
                               │   gemini-2.0-pro            │
                               │ CODEX                        │
                               │   codex-mini                │
                               └─────────────────────────────┘

MAX COST ──────────────────────────────────────────────────────
  $  [    5.00    ]   per session   (optional)

PROMPTS ───────────────────────────────────────────────────────
  + Add prompt          [Select from library ▾]

──────────────────────────────────────────────────────────────
[Cancel]                              [Save + Generate Config]
```

"Save + Generate Config" writes the branch config to `.ship/branches/<branch-name>.toml` (git-tracked, so teammates share the same agent config for this feature), then immediately generates the gitignored files (CLAUDE.md, .mcp.json etc.).

### Branch Config File

```toml
# .ship/branches/feature-auth.toml — git-tracked, team-shared

[branch]
name = "feature/auth"
spec = "spec-023"

[agent]
model = "claude-opus-4-5"
max_cost_per_session = 5.00

[[mcp_servers]]
id = "github"

[[mcp_servers]]
id = "postgres"

[[skills]]
id = "nextjs-conventions"

[[prompts]]
id = "owasp-checklist"

[context]
additional_files = ["docs/auth-architecture.md"]
```

Server and skill definitions live in the global library (`.ship/config.toml`), referenced by ID. The branch config is just IDs — no duplication of definitions.

### The Skills Library

Skills are the Shipwright equivalent of Claude's custom instructions, but versioned, shareable, and composable. A skill is a markdown file:

```markdown
<!-- .ship/skills/nextjs-conventions.md -->
---
id = "nextjs-conventions"
name = "Next.js Conventions"
description = "Project-specific Next.js patterns and conventions"
version = "1.0"
tags = ["frontend", "nextjs"]
---

## Next.js Conventions for This Project

- Use App Router, never Pages Router
- Server components by default, client components only when needed
- Colocate tests with components in __tests__/ subdirectory
- Use server actions for mutations, never API routes for internal data
```

When CLAUDE.md is generated, all active skills are included as sections. The shipwright-workflow skill is always included — it tells the agent how to use Shipwright MCP tools, how to update issues, how to file ADRs.

### Model Population

Models are not hardcoded. Shipwright queries each installed CLI for available models:

```rust
pub async fn available_models() -> Result<Vec<AvailableModel>> {
    let mut models = vec![];

    if which("claude").is_ok() {
        // claude models list --json
        if let Ok(output) = Command::new("claude")
            .args(["models", "list", "--json"])
            .output() {
            models.extend(parse_claude_models(&output.stdout)?);
        }
    }

    if which("gemini").is_ok() {
        if let Ok(output) = Command::new("gemini")
            .args(["models", "list", "--json"])
            .output() {
            models.extend(parse_gemini_models(&output.stdout)?);
        }
    }

    if which("codex").is_ok() {
        if let Ok(output) = Command::new("codex")
            .args(["models"])
            .output() {
            models.extend(parse_codex_models(&output.stdout)?);
        }
    }

    Ok(models)
}
```

Results are cached in SQLite with a 24h TTL. UI shows models grouped by provider. If a CLI isn't installed, that provider section doesn't appear.

---

## 5. Notes Feature

Notes are the lightweight capture layer before something becomes a spec. Freeform markdown, no required frontmatter, no status workflow. They live in `plans/`.

```
.ship/plans/
├── notes/
│   ├── 2026-02-22-auth-rethink.md
│   ├── 2026-02-23-mobile-idea.md
│   └── scratchpad.md
└── archive/
    └── 2026-01-15-old-approach.md
```

### Document Type

```rust
pub fn notes_document_type() -> DocumentTypeSpec {
    DocumentTypeSpec {
        id: "note",
        name: "Note",
        plural: "Notes",
        folder: "plans/notes",
        id_prefix: "note",
        template: include_str!("../templates/NOTE.md"),
        schema: NoteSchema::json_schema(),
        statuses: vec![
            Status { id: "active",   folder: "notes",   hidden: false },
            Status { id: "archived", folder: "archive", hidden: true  },
        ],
        default_git_strategy: GitStrategy::Commit,
    }
}
```

### Template

```markdown
+++
id = ""
title = ""
created = ""
updated = ""
tags = []
+++

<!-- Write freely. No structure required. -->
<!-- When ready, use "Convert to Spec" to promote this note. -->
```

### Minimum UI

- Notes list — chronological, searchable, no Kanban (notes have no workflow status)
- Note editor — full markdown, no split AI panel (keep it lightweight)
- "Promote to Spec" button — converts note to spec, moves file, opens spec editor
- Quick capture — `ship note "quick thought"` from CLI creates a timestamped note

### MCP Tools

```
ship_list_notes
ship_get_note
ship_create_note
ship_update_note
ship_promote_note_to_spec
```

---

## 6. Configurable Workflows

Teams define their own loop. The workflow is set in `.ship/config.toml` and controls which document types are active, which transitions are valid, and which git integrations are enabled.

### Workflow Presets

```toml
# .ship/config.toml

# Option A: Solo developer — simple loop, no features
[workflow]
preset = "solo"
# Enables: Notes → Specs → Issues → Kanban
# Disables: Features, worktrees (available but not default)
# Git: basic hooks, no branch-spec linking

# Option B: Small team — feature branches, optional worktrees
[workflow]
preset = "team"
# Enables: Notes → Specs → Features → Issues → Kanban → Worktrees
# Git: full hooks, branch-spec linking, context generation

# Option C: Custom
[workflow]
preset = "custom"
document_types = ["note", "spec", "issue"]   # adr optional
git.branch_spec_linking = false
git.worktree_management = false
git.generate_claude_md = true
git.generate_mcp_config = true
transitions.note_to_spec = true
transitions.spec_to_feature = false          # go straight to issues
transitions.feature_to_worktree = false
```

### Workflow Transitions

Each transition is configurable and can be disabled:

```toml
[workflow.transitions]
note_to_spec = true           # "Promote" button on notes
spec_to_feature = true        # "Create feature branch" on specs
spec_to_issues = true         # "Extract issues" on specs (always available)
feature_to_worktree = true    # "Start agent session" creates worktree
issue_to_worktree = true      # direct issue → worktree without feature
worktree_to_pr = false        # V2 — GitHub Sync required
```

### The Minimal Loop

A user who wants `spec → issue` and nothing else:

```toml
[workflow]
preset = "custom"
document_types = ["spec", "issue"]
git.generate_claude_md = true
git.generate_mcp_config = true
git.branch_spec_linking = false
git.worktree_management = false

[workflow.transitions]
note_to_spec = false
spec_to_feature = false
spec_to_issues = true
feature_to_worktree = false
issue_to_worktree = false
```

UI shows only Specs and Issues. No Notes, no Features, no worktree panel. Git hooks still generate CLAUDE.md and .mcp.json on checkout — those are always useful regardless of workflow complexity.

### The Feature-Only Loop

```toml
[workflow]
preset = "custom"
document_types = ["spec", "issue"]
git.branch_spec_linking = true
git.worktree_management = true

[workflow.transitions]
spec_to_feature = true        # feature branch straight from spec
feature_to_worktree = true    # agents work in worktrees
issue_to_worktree = false     # issues only created within features
```

No standalone issue tracking. Issues only exist inside feature branches. Kanban is per-feature-branch.

---

## 7. Directory Structure (Updated)

```
~/.shipwright/
├── config.toml              # global user config
├── shipwright.db            # global SQLite (projects registry, global mode)
├── entitlements.toml        # auth cache
└── registry.toml            # registered projects

/project-root/
└── .ship/
    ├── config.toml          # project config — git tracked
    ├── ship.db              # project SQLite — GITIGNORED
    ├── templates/           # git tracked
    │   ├── ISSUE.md
    │   ├── SPEC.md
    │   ├── ADR.md
    │   └── NOTE.md
    ├── skills/              # git tracked — team-shared
    │   ├── shipwright-workflow.md   # always included in CLAUDE.md
    │   └── <custom>.md
    ├── prompts/             # git tracked
    ├── modes/               # git tracked — team-shared
    │   ├── planning.toml
    │   └── execution.toml
    ├── branches/            # git tracked — feature branch configs
    │   └── feature-auth.toml
    ├── plans/               # git tracked
    │   ├── notes/
    │   └── archive/
    ├── specs/               # git tracked
    ├── adrs/                # git tracked
    ├── issues/              # git tracked
    │   ├── backlog/
    │   ├── in-progress/
    │   ├── review/
    │   ├── blocked/
    │   ├── closed/
    │   └── archived/
    └── log.md               # git tracked (configurable)

# Generated — GITIGNORED — never commit these in a Shipwright project
CLAUDE.md
GEMINI.md
.mcp.json
.gemini/settings.json
.codex/config.toml
.claude/
```

---

## 8. Refactor Checklist

In priority order for implementation:

**SQLite foundation**
- [ ] Add `sqlx` dependency to runtime crate
- [ ] Create `crates/runtime/src/db/` module with schema and repositories
- [ ] Migration files in `crates/runtime/migrations/`
- [ ] `Db::open()` with auto-migration
- [ ] Remove any file-based runtime state (mode overrides, session tracking)
- [ ] Update all runtime code to use repository pattern

**Git module**
- [ ] Create `crates/modules/git/` crate
- [ ] Hook dispatcher installation on `ship init`
- [ ] `post-checkout` hook — branch context generation
- [ ] `pre-commit` hook — gitignore enforcement for generated files
- [ ] `prepare-commit-msg` hook — issue context injection
- [ ] `ship init` appends generated file paths to `.gitignore`
- [ ] Worktree create/register/archive commands
- [ ] Branch-spec linking (naming convention + explicit config)
- [ ] CLAUDE.md generation from spec + issues + ADRs + skills
- [ ] MCP config generation per managed tool

**Branch config**
- [ ] `.ship/branches/<name>.toml` schema and parsing
- [ ] Branch config resolution (branch config → MCP servers, skills, model)
- [ ] Skills library (`.ship/skills/`, shipwright-workflow.md always included)

**Notes**
- [ ] Notes document type registration
- [ ] `plans/notes/` and `plans/archive/` directory structure
- [ ] Note template
- [ ] `ship note` CLI command (quick capture)
- [ ] Notes list UI
- [ ] Note editor (simple — no split AI panel)
- [ ] "Promote to Spec" action

**Agent config UI**
- [ ] MCP server library (global, add-once reuse everywhere)
- [ ] Skills library UI (list, add, edit)
- [ ] Model population from installed CLIs (cached in SQLite)
- [ ] Feature branch config panel (select from library, no magic strings)
- [ ] "Save + Generate Config" flow

**Configurable workflows**
- [ ] Workflow presets in config (solo, team, custom)
- [ ] Transition toggles
- [ ] UI responds to workflow config (hide disabled document types)

**Gitignore enforcement**
- [ ] `ship init` writes generated file paths to `.gitignore`
- [ ] `pre-commit` hook warns/blocks on staged generated files
- [ ] Documentation: why `.claude/` etc. must not be committed in Shipwright projects

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 0.1 | 2026-02-22 | Initial — SQLite, git module, modes vs branch config, agent config UI, notes, configurable workflows, global config policy |
