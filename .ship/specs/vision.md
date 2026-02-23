# Ship — Vision: The Project OS

**For:** Internal — staying focused  
**Last Updated:** 2026-02-22

---

## The North Star

**Ship is the operating system for software projects.**

Not a project management tool. Not an AI wrapper. An OS — the layer that every stakeholder, every tool, and every AI agent reads from and writes to. The connective tissue between designers, PMs, customers, developers, and the agents increasingly doing the work alongside them.

When you open a project, you open Ship. When an agent starts a session, it reads Ship. When a decision gets made, it lands in Ship. When a feature ships, Ship knew about it first.

---

## The Problem Worth Solving

Software projects today are a coordination disaster — not because people are bad at their jobs, but because the tools don't share a common substrate.

A designer works in Figma. A PM writes specs in Notion. A developer tracks issues in Linear or Jira. An AI agent works in a context window that evaporates when the session ends. A technical writer documents in Confluence. None of these tools know what the others know. Every handoff loses something. Every stakeholder maintains their own version of reality.

The deeper problem: **AI agents have no memory**. They're extraordinarily capable within a session, but they can't remember what was decided last week, what's currently in progress, or why the architecture looks the way it does. Every session starts from scratch. This isn't a model limitation — it's a missing infrastructure layer.

Ship is that infrastructure layer.

---

## The Insight: Files Are the Universal Interface

Every tool that matters to software development can read and write files. Git runs on files. AI agents read files. Developers live in files. The file system is the one API that has never broken backward compatibility.

Ship's core data model is a folder of markdown files with structured frontmatter. They live next to the code. They travel with the repo. They're readable by every tool without an integration, by every agent without an API call, and by every human without a browser tab.

This isn't a constraint — it's the architectural decision that makes everything else possible.

---

## The Plugin Model: Obsidian for Dev Teams

Ship's core is a minimal runtime:

- A file system convention (`.ship/`)
- A document runtime (parse, validate, render)
- A plugin API
- An MCP server (the AI interface)
- A configuration GUI

**Everything else is a plugin** — including the default document types (Issue, Spec, ADR). The core ships with a sensible default bundle. Teams replace what doesn't fit.

This is the Obsidian model applied to software project management. Obsidian's genius wasn't notes — it was making the plugin surface so powerful that the community built everything the core team didn't. Ship's plugin surface will be equally deep:

**Near-term (V1):** Plugins are first-party internal Rust crates that implement `ShipPlugin`, with optional TypeScript/React UI components in the Tauri frontend. Plugin authors are us. The contract is Rust + TypeScript — the same languages as the core. No new SDK, no new toolchain, no third-party distribution.

**Long-term (V3+):** Once the plugin trait has stabilized from actually building five real plugins against it, a third-party SDK makes sense. The SDK surface will be TypeScript-first (accessible, AI-generatable, familiar to the widest audience), compiling to a distributable format the Rust host can load. The design of that SDK will come from the scar tissue of V1 — not from speculation.

- Register new document types (with schema, templates, MCP tools, UI views)
- Register new workflow statuses and transitions
- Register new MCP tools
- Register new UI panels
- Hook into document lifecycle events (created, updated, moved, deleted)
- Declare git behavior (what gets committed, what gets ignored)
- Spawn and manage agent sessions

No two organizations should have the same Ship configuration. A startup's Ship looks completely different from an enterprise engineering team's Ship. Both are running the same core.

---

## What Plugins Make Possible

This is not a features list — it's a sketch of the possibility space that the plugin model opens up.

**For designers**: A Figma sync plugin that pulls design tokens, component specs, and prototype links directly into specs and issues. Designers work in Figma; the relevant artifacts land in Ship automatically.

**For PMs and founders**: A customer feedback plugin that ingests feedback from Intercom, Zendesk, or raw interviews and creates draft specs for triage. PMs respond to customer reality, not internal speculation.

**For technical writers**: A documentation plugin that generates draft docs from ADRs and completed specs. The documentation writes itself from the decisions already recorded.

**For DevOps**: A CI/CD plugin that updates issue status when deployments complete. The Kanban board reflects production reality, not developer self-reporting.

**For enterprise**: Custom approval workflows, audit logging, SSO, compliance document types, access controls — all as plugins. The enterprise tier isn't a different product. It's a different plugin bundle.

**For AI agents**: A native agent runner that spawns Claude Code sessions in git worktrees, pre-loaded with the relevant spec, open issues, prior ADRs, and project context. The agent works, updates issues as it goes, logs decisions as ADRs, and closes the worktree when done. Ship orchestrates this natively — not as an external integration but as a first-class workflow.

---

## The Agent Session Vision

This is where Ship goes from useful to transformative.

Today, an agent session is a conversation. The human provides context, the agent does work, the session ends, the context evaporates. Ship changes this.

In Ship's future, a developer looks at a spec, selects a set of issues, and hits "Start Session." Ship:

1. Creates a git worktree for the session
2. Spins up an agent (Claude Code, or whatever the user has configured) with the spec, issues, relevant ADRs, and codebase context pre-loaded
3. Monitors the session — tracking which issues the agent is working, what decisions it's making
4. As the agent completes tasks, issues update in real time on the Kanban board
5. When the agent makes an architectural decision, it files an ADR
6. When the session completes, Ship presents a summary: what was done, what ADRs were filed, what issues remain, what the agent flagged for human review
7. The developer reviews, merges the worktree, and the project state is updated

This works locally (worktrees) or in the cloud (remote agent execution). Ship manages the context hand-off so no information is lost between sessions. An agent can pick up mid-task with full awareness of what every prior agent and human has done.

This is not science fiction. The MCP primitives for this exist today. The architecture decisions made in alpha directly enable this in v2.

---

## The Stakeholder Vision

Ship eventually becomes the single shared reality for every role involved in building software:

**Developers** work in the CLI and their editor. Their AI agents read and write Ship via MCP. Issues update as code gets written.

**PMs and founders** work in the desktop app. They write specs through conversation, watch issues flow through the Kanban, and have visibility into what agents are working on without needing to understand the codebase.

**Designers** sync their work into Ship via plugin. Design decisions land in specs and ADRs. Component links appear in issues.

**Technical writers** get generated doc drafts from completed ADRs. Documentation stays in sync with the architecture.

**Customers** (eventually, via plugin) submit feedback that flows directly into draft specs. The gap between customer problem and engineering backlog shrinks from weeks to hours.

**AI agents** treat Ship as their native environment. They don't need to be told where the project is, what's in progress, or what decisions have been made. Ship is their memory.

---

## The Monetization Arc

**Free forever**: The core tool, local, for individuals and small teams. No account required. This is how trust is built.

**Plugins are the enterprise tier**: Not a separate product — a richer plugin bundle. Custom document types, approval workflows, audit logs, SSO, compliance integrations. Teams that need these pay for them. The free tier users who grow into needing them upgrade without switching tools.

**Cloud as infrastructure**: Sync, real-time collaboration, cloud agent execution. These require accounts and infrastructure. They're worth paying for. But they're never required for core functionality.

**The marketplace**: Community plugins with a revenue share. The long-term flywheel — the more useful the plugin ecosystem, the more valuable Ship becomes, the more developers build plugins, the more organizations adopt Ship.

---

## The Roadmap Arc

### Alpha (Now): Dogfood the Core Loop

_MD todos in a git repo with a nice UI and an MCP server that doesn't forget._

- `ship init` → spec → issues → Kanban → MCP
- One binary, no account, no internet required
- Good enough that the builder uses it every day

### V1: Plugin Runtime + Premium Plugins

_No two teams should look the same._

- Plugin API activated — first-party premium plugins ship as internal Rust crates implementing `ShipPlugin`
- Five premium plugins targeting SaaS/enterprise workflows (GitHub sync, agent runner, team sync, docs generation, TBD)
- Plugin authors write Rust + TypeScript — same languages as the core, no new SDK or toolchain
- Template customization via `.ship/templates/`
- Configurable git behavior
- Settings GUI for all configuration
- Issue, Spec, and ADR remain first-class core types — not extracted into plugins

### V2: Agent Sessions

_Ship runs the agents._

- Native agent runner (local worktrees)
- Session orchestration (context loading, issue tracking, ADR filing)
- Session summaries and review flows
- Cloud agent execution (optional, paid)

### V3: Stakeholder Expansion

_The whole team lives here._

- First-party plugins: Figma sync, GitHub sync, documentation generation
- Plugin marketplace (beta)
- Cloud sync and real-time collaboration
- Team Hub (shared project visibility)

### V4: Enterprise

_The plugin bundle for organizations that need more._

- Enterprise plugin bundle: SSO, audit logs, approval workflows, compliance types
- Admin controls and access management
- Advanced analytics
- Dedicated support

---

## What We Will Not Build

Being clear about this is as important as the vision:

- **Not a code editor.** Ship gives agents context. Editors and agents do the coding.
- **Not a Notion replacement.** Wikis and general docs are not Ship's domain.
- **Not an AI model.** Ship provides memory and structure. Models are brought by the user.
- **Not a SaaS-first product.** Local-first is permanent, not a temporary constraint.
- **Not enterprise-first.** The free tier has to be genuinely great. Enterprise is growth, not the foundation.

---

## The North Star Question

When evaluating any feature, roadmap decision, or architectural choice:

**Does this make the full development workflow — across every stakeholder, every tool, every AI agent — more continuous, more persistent, and less lossy?**

If yes, it belongs in Ship. If no, it probably doesn't.
