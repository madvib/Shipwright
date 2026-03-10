---
name: ship-workflow
description: >
  Guide for working with Ship — the project intelligence layer. Use this skill whenever
  you're starting a work session, planning features, making architectural decisions, logging
  progress, or wrapping up a session. Covers all three stages: Planning (features, decisions,
  notes), Workspace activation (context compilation, mode selection), and Sessions (start,
  log, end with feedback into docs). Always use this when the user says "start a session",
  "let's plan", "log what we did", or "wrap up" — and proactively consult it at the beginning
  of any interaction with a Ship-managed project.
metadata:
  display_name: Ship Workflow
  source: builtin
---

# Ship Workflow

Ship is a project intelligence layer — it keeps your agent, your team, and your codebase in
sync around what's being built and why. This skill guides you through the three-stage workflow.

---

## Stage 1 — Planning

Use planning tools when the work is about *what* to build, *why*, or *how*. You're shaping
intent, not executing code.

### Starting a planning session

1. Read `ship://project_info` first. Always. It gives you active workspaces, recent features,
   open decisions, and log entries. Don't assume you know the state — read it.
2. Orient the user: summarize what's in flight and ask what they want to focus on.

### Features — the primary planning artifact

Features are living documents, not tickets. They evolve from rough intent into documentation
as the work gets done. Every feature should have two sections:

```markdown
## Intent
What this feature is, why it exists, how it should behave from the user's perspective.
This is the planning north star — keep it stable even as implementation details change.

## Documentation
How it actually works once built. Code structure, usage, configuration, edge cases.
This section starts empty and fills in during and after the session.
```

- Use `create_feature` to capture a new feature idea. Don't wait until it's fully defined.
- Use `update_feature` to refine — iterating on intent is expected and encouraged.
- Read `ship://features/{id}` before updating a feature so you don't overwrite sections.

### Decisions — log them while they're fresh

When you commit to a technical approach, trade-off, or design choice:

- Use `create_adr` with a clear title and the reasoning. Future contributors (including you)
  will need to understand *why*, not just *what*.
- Don't log every micro-decision — only ones that would surprise someone reading the code later.

### Quick capture

Use `create_note` for thoughts that aren't ready to be features or decisions: ideas,
observations, questions, things to revisit. Notes are cheap — capture first, refine later.

---

## Stage 2 — Workspace Activation

A workspace is the execution context for a feature or patch. Activating
one compiles the right CLAUDE.md and .mcp.json for your AI provider.

### Activating a workspace

1. Read `ship://workspaces` to see what's available.
2. Call `activate_workspace` with the branch name. This recompiles provider context — the
   agent immediately has the right tools and instructions for that workspace's mode and scope.
3. Optionally call `set_mode` if you want to shift the tool surface (e.g., enable extended
   planning tools for release/spec coordination).

### Workspace types

- **service**: the project-management workspace (`ship` branch). Always present. Activating it
  unlocks the full PM surface — specs, releases, sessions history — without needing a
  mode. Use this for planning, triage, release prep, and cross-workspace coordination.
- **feature**: tied to a feature document, inherits feature's mode/provider config
- **patch**: urgent, minimal scope

**The service workspace is the home base.** If the user wants to coordinate planning, prep a release,
or work across multiple features, activate it first:

```
activate_workspace(branch="ship")
```

This compiles a bird's-eye CLAUDE.md (all features, active sessions, upcoming release)
and expands the available tools to the full PM surface automatically.

### Modes shape the tool surface

By default (non-service workspace, no mode), only core workflow tools are visible. Two ways
to expand the surface:

1. **Activate the service workspace** (`ship`) — auto-unlocks PM tools (specs, releases, notes)
2. **Set a mode** with `active_tools` configured — fine-grained control for any workspace type

```
active_tools: []          # unlocks everything
active_tools: ["create_spec", "update_spec", "list_releases"]   # scoped planning tools
```

If the user wants to work on specs or releases from a feature workspace, read
`ship://modes` and activate an appropriate mode, or suggest switching to the service workspace.

---

## Stage 3 — Session Loop

Sessions are focused work blocks. They record what was attempted, what changed, and feed
that back into the feature documentation.

### Starting a session

```
start_session(goal="<clear one-line goal>")
```

Always pass a goal. "implement workspace activate" is better than nothing. The goal becomes
part of the session record and helps the end-of-session doc pass.

### During a session — log_progress

Use `log_progress` at natural checkpoints — not obsessively, but when something significant
happens:

- A decision was made mid-session
- A blocker was hit (and what you tried)
- A significant piece of work completed
- The approach changed from the original goal

Progress notes appear in the project log and surface in `ship://project_info`. They're the
breadcrumbs that make session summaries meaningful.

### Ending a session — the feedback loop

This is the most important part. When calling `end_session`:

1. **Provide a summary**: what actually happened, not what was planned. Be honest about
   partial progress, pivots, or blockers.

2. **List updated_feature_ids**: every feature that was touched — even if just the intent
   was clarified. Ship will bump their timestamps so they surface as recently active.

3. **Review docs before closing**: read `ship://features/{id}` for each updated feature and check
   the `## Documentation` section. If it's empty or stale compared to what was built this
   session, update it with `update_feature`. Do this before calling end_session so the
   feature record is accurate.
   - Propose the documentation update to the user before writing.
   - Keep it factual: what the code does, not what you intended.

4. **For non-feature workspaces** (patch): just provide a summary.
   No feature doc update required, but a progress note before ending is still useful.

### End-session checklist

```
[ ] get_feature for each updated feature — is ## Documentation current?
[ ] update_feature if docs are stale (with user approval)
[ ] end_session with: summary, updated_feature_ids
[ ] Confirm session ended and features show updated timestamps
```

---

## System of Record Rules

- Ship entities are the source of truth for project state. Don't track work in ad-hoc chat.
- After each state-changing tool call, verify the result. Read back what you wrote.
- Keep feature intent stable — only change it if the actual goal changed, not just the
  implementation approach.
- Log decisions as ADRs, not as ad-hoc comments in features or notes. They're easier to find.
- Don't close features or sessions based on memory — read state first.

## Anti-Patterns

- Starting work without `get_project_info` — you'll miss active context
- Skipping `start_session` — progress won't be tracked and `log_progress` will fail
- Calling `end_session` without updating feature docs — documentation debt accumulates fast
- Using `create_note` when a feature or decision is more appropriate — notes are ephemeral
- Logging every micro-step with `log_progress` — signal drowns in noise
