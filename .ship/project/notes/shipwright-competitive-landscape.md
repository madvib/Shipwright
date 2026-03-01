+++
id = "eWm4eVZV"
title = "Shipwright — Competitive Landscape"
created = "2026-02-28T21:10:00Z"
updated = "2026-02-28T21:10:00Z"
tags = []
+++

# Shipwright — Competitive Landscape

**Last Updated:** 2026-02-26

---

## Vibe Kanban

**Repo:** github.com/BloopAI/vibe-kanban  
**Stars:** 21k  
**Stack:** Rust + TypeScript  
**Model:** Open source + cloud SaaS  
**Status:** Funded, 60 contributors, actively hiring, shipping fast

### What They Do Well

**Parallel agent execution.** The core insight is solving the dead time while a single agent works. Each task runs in its own git worktree — agents never conflict, work happens in parallel, you review the diff when they're done.

**Worktree lifecycle management.** Setup scripts run before the agent executes. Dev server scripts run on preview. Cleanup scripts run after. A copy list handles files that can't be in git (`.env`, local configs) — they're copied from the main project into the worktree after creation and before setup. This makes worktrees actually usable in practice.

**Diff review flow.** After an agent session completes you review changes before merging. Inline feedback, iteration, running dev server preview. The review-before-merge loop is the UX they've nailed.

**Message editing with state revert.** Edit a previous message in an agent conversation and the agent reverts its work to that point and replays from the edit. Time-travel debugging for agent sessions.

**Multi-repo workspaces.** A single workspace can span multiple repositories — frontend and backend in separate repos, one workspace, agents across both.

**Distribution.** `npx vibe-kanban` and you're running. No install, no account. 21k stars because the barrier is zero.

### Where They Stop

**No project memory.** No specs, no ADRs, no structured context that survives across tasks. Agents start every task cold — whatever context exists is in the task description. There is no layer that accumulates project knowledge over time.

**Tasks as the single workflow unit.** This is the architectural decision that caps what Vibe Kanban can become. There is no concept of features, releases, specs, or any structured documentation above the task level. Planning is a flat list. Their own users are filing issues asking for Backlog.md integration — they know this is a problem and don't have a clean answer for it.

**Not git-native in the document sense.** Project state lives in SQLite with a web UI on top. You cannot grep your project state or have an agent read it without going through their MCP server.

**SaaS trajectory.** Local product exists and works but the business is cloud, organizations, teams, GitHub integration. Local-first is a distribution strategy for them, not a commitment.

**No branch-scoped context.** Agents are configured globally or per-agent-profile. There's no concept of "this feature branch needs these MCP servers and this context because it's linked to this spec."

### What to Take From Them

**Worktree setup/copy/cleanup scripts.** The copy list (files to pull from main into the worktree before setup — `.env`, local configs) is the specific thing that makes worktrees practical. Adopt directly as fields on the feature doc or project config.

**Diff review panel.** Build a version scoped to Shipwright's context — diff alongside the linked issue and spec together. The review-before-merge flow is what makes agent output trustworthy.

**The attempt pattern in the data model.** One issue, multiple competing worktrees. The `worktrees` table should have no unique constraint on `issue_id` from day one. The UI for comparing ships in V2 but the data model must allow it earlier.

**Port pool management for dev servers.** A daemon that owns port allocation and exposes it as an MCP tool. Agents ask for a port, get a free one. Needed for V2 agent runner.

**SQLx compile-time query verification.** Schema drift becomes a build error. Adopt explicitly in `crates/runtime/src/db/`.

### Positioning Against Them

Vibe Kanban is a better agent task runner today. Shipwright is building the layer underneath that makes any agent — including ones running inside Vibe Kanban — dramatically more effective. They are not mutually exclusive. The pitch to a Vibe Kanban user: "Your agents are executing better. They're still starting every task cold. Shipwright fixes that."

---

## Taskmaster (claude-task-master)

**Repo:** github.com/eyaltoledano/claude-task-master  
**Distribution:** npm (`npx task-master-ai`) + MCP server  
**Stack:** Node.js / TypeScript  
**Model:** MIT with Commons Clause (no commercial use without license)  
**Status:** Active, community-driven, recently integrating a cloud product called Hamster

### What They Do

Taskmaster is an AI-powered task management system that drops into any AI chat — Cursor, Windsurf, Claude Code, Codex. The core workflow: write a PRD, parse it into tasks via MCP tool or CLI, work through tasks with your AI assistant one at a time. It lives inside the editor rather than as a separate application.

The PRD → tasks pipeline is their strongest feature — it handles complexity analysis, subtask expansion, dependency mapping, and research-augmented generation using a separate research model. Tasks support arbitrary JSON metadata for storing external IDs, Jira tickets, sprint data. The recent Hamster integration lets teams turn PRDs into living briefs connected to codebase and agents.

### Where They Stop

**JSON as the data model.** Tasks live in `.taskmaster/tasks/tasks.json` — a flat file that creates merge conflicts on team branches and gets unwieldy fast. The PRD gets consumed and disappears into a task list. Nothing persists as structured, human-readable documents.

**No planning structure above tasks.** PRDs are unstructured text files. No spec document type, no ADR, no feature hierarchy. The workflow is: write text, parse to tasks, execute tasks. There is no structured middle layer.

**No worktrees, no agent orchestration, no UI.** Pure CLI and MCP. No kanban, no visual workflow, no parallel execution, no branch-scoped config.

**API key required.** Requires direct AI provider API keys — no BYOM via MCP sampling.

### What to Take From Them

**PRD → task parsing depth.** `ship_extract_issues` does essentially this but Taskmaster's implementation is more mature — complexity analysis, subtask expansion, dependency mapping, research augmentation. The Shipwright equivalent should handle all of these, not just produce a flat list.

**Complexity analysis as an explicit tool.** `ship_analyze_complexity` that surfaces which issues are underspecified or too large before agent work begins. Useful standalone and as a pre-session check.

**Tagged workstreams.** Organizing issues into parallel named contexts (feature-xyz, refactor-api, tech-debt), each parsed from its own PRD. Shipwright's feature hierarchy handles this structurally via git branches, but explicit tagging for cross-feature grouping is worth considering.

### Positioning Against Them

Taskmaster is a workflow scaffold for a single AI chat session. Shipwright is persistent infrastructure. A developer who outgrows Taskmaster — wants context to survive between sessions, wants visual workflow management, wants agent config to follow branches automatically — is exactly the Shipwright customer.

---

## Continue

**Site:** continue.dev  
**Distribution:** VS Code / JetBrains extension + CLI  
**Stack:** TypeScript  
**Model:** Open source + Hub (cloud config management)  
**Status:** Funded, recently pivoted toward AI-powered PR review

### What They Do

Continue is primarily an AI coding assistant embedded in your editor. Context is provided via `.continue/rules/` markdown files scoped at different levels of the project hierarchy. Hub configurations let teams share configs centrally — models, rules, MCP servers, all versioned and shareable with a link.

Their current focus is AI-powered PR review — each check is a markdown file in your repo at `.continue/checks/`, version-controlled, runs as a full AI agent on every PR, green or red with a suggested diff.

### Where They Stop

An editor assistant with good context management. Not a project management tool. No kanban, no issue tracker, no worktrees, no specs, no ADRs, no structured planning layer. The `.continue/rules/` system is the closest thing to Shipwright's skills — static markdown files that shape agent behavior — but without the structured document types around them.

### What to Take From Them

**Hierarchical rules scoping.** Rules files placed at different levels of the project hierarchy trigger only in that context. Shipwright's skills are currently flat — all global to the project. A scoping model where skills can be placed at the feature or spec level and only activate in that context is worth considering for V1.

**AI checks as committed markdown files.** Each check is a markdown file — version-controlled, reviewable, team-owned, runs automatically. This is Shipwright's prompts system applied to the CI layer. `.ship/agents/prompts/` files that run on merge events are a natural V1 extension.

### Positioning Against Them

Not a direct competitor — different primary surface (editor vs. standalone), different primary value (autocomplete vs. project memory). A developer could use Continue in their editor and Shipwright for project management simultaneously. The Shipwright MCP server feeds Continue the same structured context it feeds Claude Code.

---

## Emerging Threats

### Claude Code (Anthropic)

The most significant potential threat. If Anthropic ships native persistent project context, structured task management, and worktree orchestration inside Claude Code, the category shifts significantly.

**Defense:** Git-native documents work with every agent, not just Claude. The full workflow layer (note → spec → feature → issue) is orthogonal to writing code — Anthropic is not incentivized to build it. Multi-provider story (Gemini CLI, Codex, Cursor) is something Anthropic structurally cannot offer.

### GitHub Copilot Workspace

Moving toward rich project management with AI from the repository relationship every developer already has. Cloud-first with no local-first answer.

**Defense:** Local-first is permanent. Project state lives next to code in git. No account required for core functionality.

### Editor-native project management (Cursor, Windsurf)

A natural extension of where editors are heading. If Cursor ships a native kanban and persistent agent memory it captures developers who never leave the editor.

**Defense:** The MCP interface works with Cursor as well as Claude Code. Shipwright doesn't care what editor you use. The structured document layer is editor-agnostic by design.

---

## Summary

| | Shipwright | Vibe Kanban | Taskmaster | Continue |
|---|---|---|---|---|
| Local-first | ✓ | Partial | ✓ | ✓ |
| Git-native documents | ✓ | ✗ | Partial | Partial |
| Persistent project memory | ✓ | ✗ | Partial | ✗ |
| Visual workflow (kanban) | ✓ | ✓ | ✗ | ✗ |
| Spec → issue workflow | ✓ | ✗ | Partial | ✗ |
| ADRs / decisions | ✓ | ✗ | ✗ | ✗ |
| Parallel agent execution | V2 | ✓ | ✗ | ✗ |
| Branch-scoped agent config | ✓ | ✗ | ✗ | ✗ |
| MCP server management | ✓ | Partial | ✗ | ✗ |
| Multi-provider agents | ✓ | ✓ | ✓ | ✓ |
| No API key required | ✓ | ✓ | ✗ | Partial |
| No account required | ✓ | ✓ | ✓ | ✓ |
