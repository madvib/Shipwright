# Shipwright — Vision

**The structure layer your AI agents don't have.**

---

## The Problem

AI agents are extraordinarily capable within a session. Between sessions, they remember nothing. Every conversation starts from scratch — re-explaining the project, re-establishing context, re-discovering what was decided last week and why.

The tools aren't the problem. The missing layer is persistent, structured project memory that agents can read without being told where to look. And a way to configure that memory precisely for the work being done right now.

That's Shipwright.

---

## The Insight

Every tool that matters to software development can read and write files. Git runs on files. AI agents read files. Developers live in files. The file system is the one API that has never broken backward compatibility.

Shipwright's data model is a folder of markdown files with TOML frontmatter — git-friendly, grep-able, readable by any agent without an API call and any human without a browser tab. Runtime state that agents and orchestration churn through lives in SQLite — transactional, fast, local.

The stuff humans write lives in git. The stuff agents churn through lives in SQLite. Both are local. Neither requires a cloud.

---

## How It Works

```
shipwright init
```

Creates `.ship/` in your project. From that moment your specs, issues, ADRs, and notes are plain files next to your code. A git hook fires on every branch checkout. Shipwright reads the branch, finds the relevant spec, and writes `CLAUDE.md` and `.mcp.json` automatically — scoped to exactly the work on this branch. Your agent wakes up with the right context, the right tools, and no re-explaining required.

The core loop:

```
Note → Spec → Feature branch → Issues → Agent session (worktree) → Merge → Archive
```

Every step is auditable. Every decision is recorded. Every agent session starts with full context and ends with a summary.

---

## The Agent Config Layer

Developers using Claude Code, Gemini CLI, or Codex today manage MCP servers through scattered config files across their machine. There's no concept of "these servers are relevant for auth work" versus "these for frontend." It's manual, fragile, and gets worse with every new tool.

Shipwright fixes this at the feature branch level. When you create a feature branch, Shipwright generates the MCP config for that work — the servers, skills, context files, model, and prompts that make sense for this spec. Switch branches, the config switches with you. The agent is always correctly equipped.

For project-wide configuration that belongs on every branch, Shipwright manages your global AI CLI configs — writing to `~/.claude/`, `~/.gemini/`, `~/.codex/` only when you explicitly set global defaults. Everything else is project-scoped and gitignored.

---

## The Module System

Shipwright's core is a runtime. Everything visible is a module — Issues, Specs, ADRs, Notes, Git, Agents. Modules register document types, MCP tools, CLI commands, and UI contributions. First-party modules are compiled in. Third-party extensions come later, once the API has scar tissue from real use.

Premium modules extend the runtime for teams: GitHub Sync, Agent Runner, Team Sync, Documentation Generation. One binary, entitlement-gated, no reinstall on upgrade.

---

## Modes

Modes shape the Shipwright UI and MCP tool surface for the kind of work you're doing — Planning when drafting specs, Execution when working issues, Review when filing ADRs. They're manually switched and global to your Shipwright session.

Branch config is different. It's derived automatically from the spec and the branch. Modes are about human intent. Branch config is about agent environment. Both exist. Neither replaces the other.

---

## Workflows Are Configurable

Not every team needs the full loop. Shipwright's workflow is configurable:

- Solo developer: Note → Spec → Issues → Agent. No features, no worktrees.
- Small team: Spec → Feature branch → Issues → Worktree agents → Merge.
- Large team: Full loop with approval gates, audit logging, enterprise plugins.

The default workflow is sensible. Teams replace what doesn't fit.

---

## Monetization

**Free forever:** Core tool, local, no account required. The free tier is a complete product, not a trial.

**Premium modules:** GitHub Sync, Agent Runner, Team Sync, Docs Generator. Compiled in, entitlement-gated. Upgrade = immediate unlock, no reinstall.

**Marketplace:** MCP server discovery, community workflows, skills library. Server authors pay for verification and featured placement.

**Enterprise:** SSO, audit logs, compliance document types, custom modules.

---

## Roadmap

**Alpha — The core loop works.**
Init, Notes, Specs, Issues, Kanban, MCP server, git hooks, branch-scoped CLAUDE.md and MCP config generation, SQLite runtime state. No account. No internet. Good enough to use every day.

**V1 — The agent config layer.**
External MCP management (Claude Code, Gemini CLI, Codex). Feature-branch config UI — select servers, skills, context, model from a library, no magic strings. Global AI CLI config management. Premium modules + auth. MCP marketplace beta.

**V2 — Shipwright runs the agents.**
Native agent runner, worktree orchestration, session summaries, parallel agent coordination, cloud execution (optional).

**V3 — The whole team.**
Figma sync, CI/CD integration, customer feedback pipeline, real-time collaboration, mobile session monitor.

**V4 — Enterprise.**
SSO, audit logs, approval workflows, compliance types, admin controls.

---

## What Shipwright Is Not

- Not a code editor. Agents do the coding.
- Not a Notion replacement. General docs are not Shipwright's domain.
- Not an AI model. Models are brought by the user.
- Not SaaS-first. Local-first is permanent.
- Not enterprise-first. The free tier has to be genuinely great.

---

## Naming and Format Conventions

**Does this make the full development workflow — across every stakeholder, every tool, every AI agent — more continuous, more persistent, and less lossy?**

If yes, it belongs in Shipwright.
