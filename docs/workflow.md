# Ship Workflow Guide

How to work with Ship's commander pattern — give it a goal, get a plan, track progress.

---

## The Three Building Blocks

### Target
The north star for a phase of work. Describes the desired end state in plain language:

> "Ship a working auth flow with GitHub OAuth and session management by end of March."

Targets don't change often. Every capability and job is evaluated against the active target.

### Capability
A verifiable slice of a target. Has:
- A title ("GitHub OAuth login flow")
- Acceptance criteria — a checklist you can actually verify
- A phase (which milestone it belongs to)
- A scope (which files/dirs an agent may touch)

Capabilities start **aspirational** and become **actual** once a gate passes with evidence.

### Job
The execution record for a unit of work. Tied to a capability, assigned to an agent (or human). Moves through: `pending → running → done` (or `blocked`).

Commander creates jobs, dispatches them, monitors progress, and gates results.

---

## How Commander Translates Intent

You don't write the capability map. You give commander a goal:

> "I want users to be able to sign in with GitHub"

Commander reads the active target, asks you 1–2 focused questions if needed, then builds the plan:

```
Capability 1: GitHub OAuth provider setup
  → Agent: better-auth
  → Scope: apps/web/src/auth/
  → Done when: OAuth flow completes in dev, tokens stored correctly

Capability 2: Login/logout UI
  → Agent: web-lane
  → Scope: apps/web/src/components/auth/
  → Done when: login button visible, session persists on refresh

Cap1 and Cap2 can run in parallel. Confirm and I'll dispatch.
```

You confirm. Commander dispatches agents into isolated worktrees, each preconfigured with the right profile. You get updates when things are blocked or done.

---

## The Three Modes

Commander is always in one mode:

### PLANNER
Active at session start or when you give a new goal. Commander reads the target, asks 1–2 questions, builds the capability map, gets your confirmation. Nothing dispatches until you say go.

### ORCHESTRATOR
Active when jobs are running. Commander monitors progress, surfaces blockers, routes work between agents. It does not do specialist work — it routes.

### GATE
Active when an agent marks a job done. Commander reviews the work against the capability's acceptance criteria. Pass: capability becomes actual, worktree pruned. Fail: job blocked, specific failures surfaced. You only see failures that need your decision.

---

## How Agents Are Dispatched

Each job runs in its own git worktree — an isolated copy of the repo on a dedicated branch. Commander:

1. Creates the worktree at `~/dev/ship-worktrees/<job-id>`
2. Runs `ship use <profile>` to compile the agent's full context into `CLAUDE.md`
3. Writes a `job-spec.md` with scope, acceptance criteria, and constraints
4. Gives you a one-line launch command to paste in a new terminal tab

The agent reads its compiled context automatically at session start, then reads `job-spec.md` and begins — no further input needed.

```
→ worktree: ~/dev/ship-worktrees/abc123
→ launch:

  cd ~/dev/ship-worktrees/abc123 && claude .
```

---

## Starting a Session

1. Open your project directory in Claude Code with Ship MCP active
2. Tell commander what you want to build — or ask what's in flight
3. Commander reads the active target and checks the job queue
4. Running jobs → ORCHESTRATOR, you get a status brief
5. No running jobs → tell commander your goal, PLANNER mode begins

You don't need to know what a capability is. Just say what you're trying to accomplish.

---

## Targets and ADRs as Documents of Intent

Targets describe outcome intent. ADRs (Architecture Decision Records) capture the reasoning behind significant choices. Neither is ARCHITECTURE.md — that's a code-level reference.

When commander makes a meaningful architecture decision, it writes an ADR. When a phase completes, the target's capabilities flip from aspirational to actual — the target becomes a record of what was built, not just what was planned.
