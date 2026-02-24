# Shipwright — Vision & Architecture

**Version:** 0.2  
**Status:** Active  
**Last Updated:** 2026-02-22  
**Replaces:** ship-vision-v2.md, shipwright-architecture.md, ship-plugin-monetization.md

---

## North Star

**Shipwright is the operating system for software projects.**

Not a project management tool. Not an AI wrapper. A runtime — the substrate that every stakeholder, every tool, and every AI agent reads from and writes to. The connective tissue between designers, PMs, customers, developers, and the agents increasingly doing the work alongside them.

When you open a project, you open Shipwright. When an agent starts a session, it reads Shipwright. When a decision gets made, it lands in Shipwright. When a feature ships, Shipwright knew about it first.

The core loop that everything serves:

```
Chat → Refine Spec → Extract Issues → Work Issues (human or agent) → Update Issues → Repeat
```

---

## The Problem Worth Solving

Software projects today are a coordination disaster — not because people are bad at their jobs, but because the tools don't share a common substrate.

A designer works in Figma. A PM writes specs in Notion. A developer tracks issues in Linear. An AI agent works in a context window that evaporates when the session ends. A technical writer documents in Confluence. None of these tools know what the others know. Every handoff loses something. Every stakeholder maintains their own version of reality.

The deeper problem: **AI agents have no persistent memory.** They're extraordinarily capable within a session, but they can't remember what was decided last week, what's currently in progress, or why the architecture looks the way it does. Every session starts from scratch. This isn't a model limitation — it's a missing infrastructure layer.

Shipwright is that infrastructure layer.

---

## The Insight: Files Are the Universal Interface

Every tool that matters to software development can read and write files. Git runs on files. AI agents read files. Developers live in files. The file system is the one API that has never broken backward compatibility.

Shipwright's core data model is a folder of markdown files with TOML frontmatter. They live next to the code. They travel with the repo. They're readable by every tool without an integration, by every agent without an API call, and by every human without a browser tab.

This isn't a constraint — it's the architectural decision that makes everything else possible.

---

## Format Standard

One rule, zero exceptions:

| File type | Format |
|-----------|--------|
| All documents (Issues, Specs, ADRs) | Markdown with TOML frontmatter (`+++` delimiters) |
| All config | TOML (`.toml`) |

No YAML. No JSON. No exceptions. TOML is unambiguous, comment-supporting, and already familiar to Rust developers via Cargo.

---

## The Three-Layer Architecture

Shipwright has three distinct layers. Understanding the boundary between them is the key to the whole system.

### Layer 1 — Runtime (Rust, always OSS-adjacent)

The runtime has no opinions about what a project looks like. It provides the substrate and enforces the conventions. It is the OS.

**What the runtime owns:**
- File system conventions (`.ship/` directory structure, TOML frontmatter parsing)
- Document store (read, write, index, query any registered document type)
- Relationship graph (cross-document links, parent/child, blocks/blocked-by)
- Mode manager (active mode, MCP tool filtering)
- MCP server (dynamic tool registration, mode-aware capability surface)
- Event bus (async, cross-module communication)
- Auth + entitlements (JWT, cloud entitlement cache, offline grace period)
- Config export (generates `.claude`, `.gemini`, `.cursor` configs from modes)
- Action log (append-only `log.md`, human + agent readable)
- Per-module store (scoped key-value, backed by files)
- V8 isolate host — **V2, not now** (deno_core, for third-party TypeScript extensions)

**What the runtime does NOT own:**
- What an issue looks like
- What a spec looks like
- What valid statuses are
- Any UI

The test: remove every module. Shipwright should still boot, present an empty canvas, and be fully functional — just with nothing registered.

### Layer 2 — Modules (Rust, compiled into binary, full trust)

Modules are first-party Shipwright code. They implement an internal Rust trait, are compiled directly into the binary, and have full access to runtime internals. They are not sandboxed. The distinction between "core" and "module" is intentionally blurry — Issues is a module the same way the document store is a module.

**Default module bundle (OSS):**
- Issues module (Kanban UI, issue document type, MCP tools)
- Specs module (split editor, AI conversation, MCP tools)
- ADRs module (list view, immutable record UI, MCP tools)

**Premium modules (compiled in, entitlement gated):**
- GitHub Sync
- Agent Runner
- Team Sync
- Docs Generator
- TBD (5th premium module)

Modules are not called "plugins." They are not installed. They are not loaded at runtime from external files. They are Rust code in `crates/modules/`.

### Layer 3 — Extensions (TypeScript, sandboxed V8, V2+)

Third-party code written by the community. Runs in a sandboxed V8 isolate (deno_core). Communicates with the runtime exclusively via declared host functions. Cannot access Tauri IPC directly. Cannot exceed declared permissions.

**Not built until V2.** The architecture is designed to accommodate extensions without requiring them. The host function surface that extensions will eventually use is the same surface the runtime exposes today — it just isn't accessible from outside the binary yet.

---

## Crate & Package Structure

```
shipwright/
├── crates/
│   ├── runtime/                    # Layer 1 — the OS
│   │   ├── src/
│   │   │   ├── document/           # Generic document model + file store
│   │   │   ├── relationships/      # Cross-document graph
│   │   │   ├── modes/              # Mode manager + MCP tool filtering
│   │   │   ├── mcp/                # MCP server + dynamic tool registry
│   │   │   ├── events/             # Async event bus
│   │   │   ├── store/              # Per-module scoped key-value store
│   │   │   ├── auth/               # JWT + entitlement cache
│   │   │   ├── config/             # Global + project TOML config
│   │   │   ├── export/             # Config export (.claude, .gemini, .cursor)
│   │   │   └── log/                # Action log writer
│   │   └── Cargo.toml
│   │
│   ├── sdk/                        # Module SDK — the internal Rust contract
│   │   ├── src/
│   │   │   ├── module.rs           # ShipwrightModule trait
│   │   │   ├── context.rs          # ModuleContext — runtime API surface
│   │   │   ├── document.rs         # DocumentTypeSpec, GitStrategy
│   │   │   ├── mcp.rs              # McpTool builder types
│   │   │   ├── ui.rs               # UiContribution types
│   │   │   └── manifest.rs         # ModuleManifest
│   │   └── Cargo.toml
│   │
│   ├── cli/                        # CLI — thin layer over runtime
│   │   └── Cargo.toml
│   │
│   ├── ui/                         # Tauri app shell
│   │   ├── src/
│   │   │   ├── shell/              # App shell — slot mounting, mode switcher
│   │   │   │   ├── App.tsx
│   │   │   │   ├── ModeBar.tsx     # Prominent mode switcher — top of UI
│   │   │   │   ├── Sidebar.tsx     # Nav items registered by modules
│   │   │   │   ├── SlotRouter.tsx  # Mounts module UI contributions
│   │   │   │   └── AiPanel.tsx     # Persistent AI conversation panel
│   │   │   └── hooks/              # Tauri + runtime React hooks
│   │   ├── src-tauri/
│   │   │   ├── src/
│   │   │   │   ├── commands.rs     # Tauri commands (specta-typed)
│   │   │   │   └── main.rs
│   │   │   └── Cargo.toml
│   │   └── package.json
│   │
│   └── modules/                    # Layer 2 — first-party modules
│       ├── issues/
│       │   ├── src/
│       │   │   ├── lib.rs          # Implements ShipwrightModule
│       │   │   ├── document.rs     # Issue document type definition
│       │   │   ├── mcp.rs          # MCP tools
│       │   │   └── commands.rs     # Tauri commands for this module
│       │   ├── ui/                 # React components
│       │   │   ├── KanbanView.tsx
│       │   │   ├── IssueDetail.tsx
│       │   │   └── IssueCard.tsx
│       │   └── Cargo.toml
│       │
│       ├── specs/
│       │   ├── src/
│       │   │   ├── lib.rs
│       │   │   ├── document.rs
│       │   │   ├── mcp.rs
│       │   │   └── sampling.rs     # MCP sampling for spec refinement
│       │   ├── ui/
│       │   │   ├── SpecList.tsx
│       │   │   └── SpecEditor.tsx  # Split view: doc + AI panel
│       │   └── Cargo.toml
│       │
│       ├── adrs/
│       │   ├── src/
│       │   │   ├── lib.rs
│       │   │   ├── document.rs
│       │   │   └── mcp.rs
│       │   ├── ui/
│       │   │   ├── AdrList.tsx
│       │   │   └── AdrDetail.tsx
│       │   └── Cargo.toml
│       │
│       └── premium/                # Private repo — entitlement gated
│           ├── github-sync/
│           ├── agent-runner/
│           ├── team-sync/
│           └── docs-gen/
│
├── packages/                       # Shared TypeScript packages
│   └── ui/                         # @shipwright/ui — design system
│       ├── src/
│       │   ├── components/         # Board, Column, Card, Panel, Badge...
│       │   ├── tokens/             # Colors, spacing, typography
│       │   └── index.ts
│       └── package.json            # "@shipwright/ui"
│
└── Cargo.toml                      # Workspace
```

**Key structural decisions:**
- `packages/ui` is extracted as its own package. Both the app shell (`crates/ui`) and all module UI components import from `@shipwright/ui`. This is the design system boundary. It is extracted now, while cheap, not later when it's painful.
- `specta` generates TypeScript types from Tauri commands automatically. Every Tauri command is the typed bridge between Rust and TypeScript. This is Tauri doing what Tauri was designed for — no custom IPC layer needed.
- Module UI components live inside their module crate (`crates/modules/issues/ui/`). They're co-located with the Rust logic they display. The app shell mounts them via the slot system.

---

## The Module Trait

```rust
// crates/sdk/src/module.rs

pub trait ShipwrightModule: Send + Sync {
    // Identity
    fn manifest(&self) -> &ModuleManifest;

    // Lifecycle
    fn on_load(&self, ctx: &ModuleContext) -> Result<()> { Ok(()) }
    fn on_unload(&self, ctx: &ModuleContext) -> Result<()> { Ok(()) }

    // Contributions — what this module registers into the runtime
    fn document_types(&self) -> Vec<DocumentTypeSpec> { vec![] }
    fn mcp_tools(&self) -> Vec<McpToolSpec> { vec![] }
    fn cli_commands(&self) -> Vec<CliCommandSpec> { vec![] }
    fn ui_contributions(&self) -> UiContributions { UiContributions::default() }
    fn settings_schema(&self) -> Vec<SettingSpec> { vec![] }
    fn mode_contributions(&self) -> Vec<ModeSpec> { vec![] }

    // Event handlers
    fn on_document_created(&self, ctx: &ModuleContext, doc: &Document) -> Result<()> { Ok(()) }
    fn on_document_updated(&self, ctx: &ModuleContext, doc: &Document) -> Result<()> { Ok(()) }
    fn on_document_moved(&self, ctx: &ModuleContext, doc: &Document, from: &str, to: &str) -> Result<()> { Ok(()) }
    fn on_document_deleted(&self, ctx: &ModuleContext, doc: &Document) -> Result<()> { Ok(()) }
}

pub struct ModuleManifest {
    pub id: &'static str,                        // "shipwright.issues"
    pub name: &'static str,                      // "Issues"
    pub version: &'static str,
    pub description: &'static str,
    pub entitlement: Option<&'static str>,       // None = always free
}
```

---

## ModuleContext — The Runtime API

Everything a module can do goes through `ModuleContext`. It is the boundary between module code and the runtime.

```rust
// crates/sdk/src/context.rs

pub struct ModuleContext {
    pub documents: DocumentApi,       // Read/write any registered document type
    pub relationships: RelationshipApi, // Cross-document links
    pub mcp: McpApi,                  // Dynamic MCP tool registration
    pub events: EventBusApi,          // Emit and subscribe to events
    pub store: ModuleStoreApi,        // Scoped key-value persistence
    pub config: ConfigApi,            // Read project + global config
    pub log: LogApi,                  // Write to action log
    pub http: HttpApi,                // Network (modules: unrestricted)
    pub sampling: SamplingApi,        // MCP sampling — BYOM AI calls
}
```

### Document API

```rust
impl DocumentApi {
    pub async fn get(&self, type_id: &str, id: &str) -> Result<Option<Document>>;
    pub async fn list(&self, type_id: &str, filter: DocumentFilter) -> Result<Vec<Document>>;
    pub async fn create(&self, type_id: &str, frontmatter: Frontmatter, body: &str) -> Result<Document>;
    pub async fn update(&self, type_id: &str, id: &str, patch: DocumentPatch) -> Result<Document>;
    pub async fn move_to(&self, type_id: &str, id: &str, status: &str) -> Result<Document>;
    pub async fn delete(&self, type_id: &str, id: &str) -> Result<()>;
}
```

### MCP API — Dynamic Registration

```rust
impl McpApi {
    // Register a tool into the live MCP server
    // Broadcasts tools/list_changed to all connected agents immediately
    pub async fn register(&self, tool: McpTool) -> Result<()>;
    pub async fn unregister(&self, tool_name: &str) -> Result<()>;
}
```

### Event Bus

Modules communicate via events, never by importing each other. The event bus is the only cross-module communication channel.

```rust
impl EventBusApi {
    pub fn emit(&self, event: &str, payload: serde_json::Value);
    pub fn on(&self, event: &str, handler: impl Fn(Event) + Send + Sync + 'static);
}
```

**Well-known events (stable contract):**
```
document.created          { type_id, id, doc }
document.updated          { type_id, id, doc, patch }
document.moved            { type_id, id, from, to, doc }
document.deleted          { type_id, id }
document.extract_requested { content, suggested_type, parent_ref }
mode.changed              { from, to }
session.started           { session_id, mode }
session.completed         { session_id, summary }
auth.login                { plan, entitlements }
auth.logout               {}
```

### Sampling API — BYOM

```rust
impl SamplingApi {
    // Request a completion from the user's connected AI client
    // No API key. No model cost through Shipwright.
    // Uses MCP sampling protocol.
    pub async fn complete(&self, prompt: &str, ctx: SampleContext) -> Result<String>;
    
    // Convenience: complete with document context pre-loaded
    pub async fn complete_with_doc(&self, prompt: &str, doc: &Document) -> Result<String>;
}
```

---

## Document Type Registration

A module fully defines its document types — schema, template, valid statuses, git strategy.

```rust
// crates/modules/issues/src/document.rs

pub fn issue_document_type() -> DocumentTypeSpec {
    DocumentTypeSpec {
        id: "issue",
        name: "Issue",
        plural: "Issues",
        folder: "issues",
        id_prefix: "issue",
        template: include_str!("../templates/ISSUE.md"),
        schema: IssueSchema::json_schema(),

        statuses: vec![
            Status { id: "backlog",     name: "Backlog",     color: "gray",   folder: "backlog",      hidden_in_kanban: false },
            Status { id: "in-progress", name: "In Progress", color: "blue",   folder: "in-progress",  hidden_in_kanban: false },
            Status { id: "review",      name: "Review",      color: "yellow", folder: "review",       hidden_in_kanban: false },
            Status { id: "blocked",     name: "Blocked",     color: "red",    folder: "blocked",      hidden_in_kanban: false },
            Status { id: "closed",      name: "Closed",      color: "green",  folder: "closed",       hidden_in_kanban: false },
            Status { id: "archived",    name: "Archived",    color: "muted",  folder: "archived",     hidden_in_kanban: true },
        ],

        // Teams override this in .ship/config.toml [modules.issues]
        default_git_strategy: GitStrategy::Manifest {
            manifest_path: "issues/.manifest.toml",
        },
    }
}

pub enum GitStrategy {
    Ignore,                                      // never commit — pure local
    Commit,                                      // always commit individual files
    ArchiveOnly,                                 // only commit when archived/closed
    Manifest { manifest_path: String },          // commit a summary, not individual files
}
```

**The `hidden_in_kanban` flag** solves clutter directly. Archived issues exist, are searchable, and are referenced — they just don't appear as a Kanban column. "Closed" is the terminal working state. "Archived" is long-term storage.

---

## Cross-Module Communication

Modules never import each other. The runtime and event bus broker all communication.

### Example: Spec → Extract Issue

```
User clicks "Extract Issue" in SpecEditor
        ↓
Specs module emits:
  events.emit("document.extract_requested", {
    content: "selected text or full spec section",
    suggested_type: "issue",
    parent_ref: { type: "spec", id: "spec-001" }
  })
        ↓
Runtime routes to all subscribers
        ↓
Issues module handler:
  ctx.documents.create("issue", {
    title: derive_title(content),
    spec: "spec-001",
    tags: [],
  }, content)
        ↓
  ctx.relationships.link(
    DocRef::new("spec", "spec-001"),
    "parent",
    DocRef::new("issue", new_issue.id)
  )
        ↓
Issue created. Relationship recorded. Kanban updates.
```

Neither module knows the other exists. The runtime routes the event.

---

## Modes

Modes are a first-class concept in Shipwright. A mode defines:
- Which MCP tools are active (capability surface for AI agents)
- Which AI context files are pre-loaded
- Which UI layout is default
- How the AI conversation panel is scoped

Switching modes changes what AI agents can do — immediately, without reconnecting. The MCP server broadcasts `tools/list_changed` on mode switch.

```toml
# .ship/config.toml

[[modes]]
id = "planning"
name = "Planning"
description = "Spec writing and issue creation with AI assistance"
mcp_tools = [
  "ship_list_specs",
  "ship_create_spec",
  "ship_refine_spec",
  "ship_extract_issues",
  "ship_list_issues",
  "ship_create_issue",
]
ai_context = ["AGENTS.md", "specs/", "adrs/"]
ui_layout = "spec-editor"

[[modes]]
id = "execution"
name = "Execution"
description = "Working issues — human or agent"
mcp_tools = [
  "ship_list_issues",
  "ship_get_issue",
  "ship_move_issue",
  "ship_update_issue",
  "ship_link_issues",
  "ship_get_log",
]
ai_context = ["AGENTS.md", "issues/in-progress/"]
ui_layout = "kanban"

[[modes]]
id = "review"
name = "Review"
description = "Architecture review and decision recording"
mcp_tools = [
  "ship_list_adrs",
  "ship_get_adr",
  "ship_create_adr",
  "ship_draft_adr",
  "ship_list_specs",
  "ship_get_spec",
]
ai_context = ["AGENTS.md", "adrs/", "specs/"]
ui_layout = "adr-list"
```

The mode switcher is prominent in the UI — top bar, always visible. It is not a setting. It is a primary navigation concept.

**Capability-based security via modes:** An agent connected in execution mode literally cannot create ADRs — the tool doesn't exist in its tool list. An agent in planning mode cannot move issues. Modes express workflow intent and enforce it at the protocol level.

---

## Config Export

Shipwright is the source of truth for AI configuration across all tools. Users define their workflow once in Shipwright. Shipwright generates the right config for whatever AI tools they use.

```bash
shipwright modes export --target claude    # → CLAUDE.md + MCP server config
shipwright modes export --target gemini    # → .gemini/config.json
shipwright modes export --target cursor    # → .cursorrules
shipwright modes export --target all       # → all of the above
```

This positions Shipwright as the **unified AI config layer** — not competing with Claude or Cursor, but configuring them. The mode definitions, context files, and MCP tool lists translate directly into each tool's native format.

Generated configs reference the Shipwright MCP server so agents always have access to project state regardless of which tool they're running in.

---

## AI Integration in the UI

Every document type gets a consistent AI panel pattern. This is not a sidebar afterthought — it is a first-class split view present across the entire application.

```
Any document (Issue, Spec, ADR, or future module document types)
├── Left panel: document editor
│   ├── Markdown with live preview
│   ├── Frontmatter as editable form fields
│   └── Full keyboard editing
└── Right panel: AI conversation
    ├── Scoped context (this document + related documents)
    ├── Mode-aware (only offers actions valid in current mode)
    ├── BYOM via MCP sampling (no Shipwright API key, no model cost)
    └── Actions (applied directly to the document):
        ├── "Apply suggestion" — patches markdown in place
        ├── "Extract issue" — creates issue from selected content
        ├── "Create ADR" — extracts decision into new ADR
        ├── "Suggest tasks" — generates task checklist from description
        └── "Find related" — queries document relationship graph
```

The AI panel is powered by MCP sampling — the user's connected AI client (Claude, Cursor, Windsurf) provides completions. Shipwright sends the document content and project context as the sampling prompt. The response comes back to Shipwright, which renders it in the panel and offers inline apply actions.

**Generative editing** — "Apply suggestion" directly patches the markdown document, not just the chat. The AI and the document are in dialogue, not parallel.

---

## Dynamic MCP

The MCP server is a live capability surface that changes based on:
- Which modules are loaded (and their entitlements)
- Which mode is active
- Which tools modules have dynamically registered via `ctx.mcp.register()`

```rust
// crates/runtime/src/mcp/registry.rs

pub struct McpRegistry {
    tools: Arc<RwLock<HashMap<String, McpTool>>>,
    active_mode: Arc<RwLock<String>>,
    subscribers: Arc<RwLock<Vec<McpClientHandle>>>,
}

impl McpRegistry {
    pub async fn register(&self, tool: McpTool) -> Result<()> {
        self.tools.write().await.insert(tool.name.clone(), tool);
        self.broadcast_tools_changed().await;
        Ok(())
    }

    pub async fn active_tools(&self) -> Vec<McpTool> {
        let mode = self.active_mode.read().await;
        let tools = self.tools.read().await;
        // Filter to tools permitted in current mode
        tools.values()
            .filter(|t| mode_permits(&mode, &t.name))
            .cloned()
            .collect()
    }

    async fn broadcast_tools_changed(&self) {
        // MCP spec: notifications/tools/list_changed
        // Connected agents re-fetch tool list immediately
        for client in self.subscribers.read().await.iter() {
            client.send(McpNotification::ToolsListChanged).await.ok();
        }
    }
}
```

---

## UI Architecture

The app shell is a slot system. It has no hardcoded routes or views. Everything visible is registered by a module at load time.

```tsx
// crates/ui/src/shell/App.tsx

export function App() {
  const { modules, activeMode } = useRuntime()

  return (
    <Layout>
      <ModeBar />                              {/* Always visible — mode switcher */}
      <Sidebar>
        {modules.navItems.map(item => (
          <NavItem key={item.id} {...item} />  {/* Registered by modules */}
        ))}
      </Sidebar>
      <Main>
        <SlotRouter routes={modules.routes} /> {/* Module views mounted here */}
      </Main>
      <AiPanel />                              {/* Persistent — always available */}
    </Layout>
  )
}
```

Module UI uses `@shipwright/ui` components and Tauri commands for data access:

```tsx
// crates/modules/issues/ui/KanbanView.tsx

import { useDocuments } from '../../hooks/useDocuments'
import { Board, Column, Card, Badge } from '@shipwright/ui'

export function KanbanView() {
  const { documents, moveDocument } = useDocuments('issue')
  const statuses = useDocumentType('issue')
    .statuses
    .filter(s => !s.hidden_in_kanban)

  return (
    <Board>
      {statuses.map(status => (
        <Column
          key={status.id}
          title={status.name}
          color={status.color}
          onDrop={(issueId) => moveDocument(issueId, status.id)}
        >
          {documents
            .filter(d => d.status === status.id)
            .map(issue => (
              <Card key={issue.id} onClick={() => openDetail(issue.id)}>
                <span>{issue.frontmatter.title}</span>
                {issue.frontmatter.tags.map(tag => (
                  <Badge key={tag}>{tag}</Badge>
                ))}
              </Card>
            ))}
        </Column>
      ))}
    </Board>
  )
}
```

**UI trust boundary:**

| | First-party modules | Third-party extensions (V2+) |
|---|---|---|
| React components | Direct mount in app shell | Sandboxed iframe |
| Design system | `@shipwright/ui` direct import | `@shipwright/ui` via bundle |
| Data access | `useDocuments()` Tauri hook | postMessage API only |
| IPC | Direct `invoke()` | No Tauri access |

---

## Directory Structure

```
~/.shipwright/
├── config.toml           # Global user config
├── entitlements.toml     # Cached entitlements (written by auth)
└── registry.toml         # Registered projects

/project-root/
└── .ship/
    ├── config.toml       # Project config (modules, modes, statuses, tags)
    ├── templates/
    │   ├── ISSUE.md      # Editable — teams customize these
    │   ├── SPEC.md
    │   └── ADR.md
    ├── issues/
    │   ├── backlog/
    │   ├── in-progress/
    │   ├── review/
    │   ├── blocked/
    │   ├── closed/
    │   └── archived/     # hidden_in_kanban = true
    ├── specs/
    ├── adrs/
    ├── modules/          # Per-module scoped stores
    │   └── github-sync/
    │       └── store.toml
    └── log.md            # Append-only action log
```

---

## Project Config

```toml
# .ship/config.toml

version = "1"
name = "my-project"
description = ""

# Module enable/disable — all enabled by default if entitled
[modules]
issues.enabled = true
specs.enabled = true
adrs.enabled = true

# Module-specific config — each module defines its own schema
[modules.issues]
git_strategy = "manifest"
statuses = [
  { id = "backlog",     name = "Backlog",     color = "gray"   },
  { id = "in-progress", name = "In Progress", color = "blue"   },
  { id = "review",      name = "Review",      color = "yellow" },
  { id = "blocked",     name = "Blocked",     color = "red"    },
  { id = "closed",      name = "Closed",      color = "green"  },
  { id = "archived",    name = "Archived",    color = "muted", hidden_in_kanban = true },
]

[modules.github-sync]
repo = "owner/repo"
sync_direction = "both"
# token in keychain, not here

# Modes — teams define their own or override defaults
[[modes]]
id = "planning"
name = "Planning"
mcp_tools = ["ship_list_specs", "ship_create_spec", "ship_extract_issues"]
ai_context = ["AGENTS.md", "specs/"]
ui_layout = "spec-editor"

[[modes]]
id = "execution"
name = "Execution"
mcp_tools = ["ship_list_issues", "ship_move_issue", "ship_update_issue"]
ai_context = ["AGENTS.md", "issues/in-progress/"]
ui_layout = "kanban"
```

---

## Monetization

### Model

One binary ships to all users. Premium modules are compiled in. Cloud entitlements are the only gate. No second download. No reinstall on upgrade. Upgrade = entitlement change = immediate unlock.

### Entitlements

```rust
pub struct Entitlements {
    pub plan: Plan,
    pub modules: Vec<String>,        // ["github-sync", "agent-runner"]
    pub expires_at: DateTime<Utc>,
    pub cached_at: DateTime<Utc>,
}
```

Fetched from `https://api.shipwright.dev/v1/entitlements` on startup. Cached in `~/.shipwright/entitlements.toml`. 30-day offline grace period before premium features lock — handles travel, connectivity issues, conferences.

### Plans (Illustrative)

| Plan | Price | Modules |
|------|-------|---------|
| Free | $0 | Issues, Specs, ADRs — no account needed |
| Pro | ~$9/mo | + GitHub Sync, Agent Runner |
| Team | ~$19/seat/mo | + Team Sync, Docs Generator |
| Enterprise | Custom | + SSO, audit logs, custom modules |

### OSS Strategy

The free tier is a complete, real product. Not a trial. Not crippled. The core loop works fully without an account. This is how developer trust is built.

Premium module code being visible in the binary is acceptable — value is in the cloud features and integrations, not in the algorithm. Anyone stripping the entitlement check to self-host was never going to pay.

The `ShipwrightModule` trait is OSS. When third-party extensions open up in V2, the contract is already public. The runtime is not.

### Auth Flow

```bash
shipwright auth login    # Opens browser → OAuth → writes JWT to config
shipwright auth status   # Shows plan, active modules, expiry  
shipwright auth logout   # Clears JWT and cache → premium modules deactivate
```

---

## Action Log

Every mutation — from CLI, desktop app, or MCP server — appends to `.ship/log.md`:

```
2026-02-22T14:30:00Z [human]        issue-001 moved in-progress → review
2026-02-22T14:35:00Z [agent:claude] issue-001 updated: added task breakdown
2026-02-22T14:36:00Z [agent:claude] adr-003 created: "Use Redis for session storage"
2026-02-22T14:40:00Z [human]        spec-001 updated via AI conversation
2026-02-22T14:42:00Z [human]        mode changed: execution → review
```

Append-only. Human-readable. Gitignored by default (configurable per project). Gives agents project history without diffing files. Powers the "recent activity" view in the UI without a database.

---

## Roadmap

### Alpha — Core Loop (Now)
*Markdown todos in a git repo with a clean UI and an MCP server that doesn't forget.*

- `shipwright init` → spec → issues → Kanban → MCP
- Three modules: Issues, Specs, ADRs
- One binary, no account, no internet required
- Specta-generated types for all Tauri commands
- `@shipwright/ui` extracted as standalone package
- AI conversation panel (BYOM via MCP sampling)
- Modes defined in config, basic mode switching

### V1 — Premium Modules + Auth
*Pay for what you actually use.*

- Auth flow (`shipwright auth login`)
- Entitlement system + cloud API (Cloudflare Worker + Stripe)
- Five premium modules compiled in + gated
- Config export (`shipwright modes export`)
- Marketplace UI in Settings
- Module-level git strategy config

### V2 — Extension Runtime + Agent Sessions
*The platform opens up.*

- TypeScript extension runtime (deno_core embedded)
- Extension SDK (`@shipwright/extension-sdk`)
- `shipwright extension dev` build tooling
- Native agent runner (local worktrees)
- Session orchestration + summaries
- Extension marketplace

### V3 — Stakeholder Expansion
*The whole team lives here.*

- First-party integrations: Figma, CI/CD, customer feedback
- Cloud sync + real-time collaboration
- Mobile companion app (monitor agent sessions)
- Plugin creator module (vibe-code extensions against the SDK)

### V4 — Enterprise
*The module bundle for organizations.*

- SSO, audit logs, approval workflows
- Admin controls + access management
- Compliance document types
- Dedicated support

---

## What Shipwright Is Not

- **Not a code editor.** Shipwright gives agents context. Editors and agents do the coding.
- **Not a Notion replacement.** General wikis and docs are not Shipwright's domain.
- **Not an AI model.** Shipwright provides memory and structure. Models are brought by the user.
- **Not SaaS-first.** Local-first is permanent, not a temporary constraint.
- **Not enterprise-first.** The free tier must be genuinely great. Enterprise is growth, not the foundation.

---

## The North Star Question

When evaluating any feature, roadmap decision, or architectural choice:

**Does this make the full development workflow — across every stakeholder, every tool, every AI agent — more continuous, more persistent, and less lossy?**

If yes, it belongs in Shipwright. If no, it probably doesn't.

---

## Document History

| Version | Date | Changes |
|---------|------|---------|
| 0.1 | 2026-02-22 | Initial consolidated doc |
| 0.2 | 2026-02-22 | Added modes, AI integration, config export, dynamic MCP, UI architecture, extracted @shipwright/ui |
