# Commander Skill Rewrite Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the monolithic `commander` skill into four focused `cmdr-*` skills (umbrella + 3 role skills), update the commander profile to reference them, fix stale refs in spawn-agent, and write a new `docs/workflow.md` explainer.

**Architecture:** The existing `commander/SKILL.md` is replaced by `cmdr/SKILL.md` (umbrella) plus `cmdr-planner`, `cmdr-orchestrator`, and `cmdr-gate` — all flat siblings under `.ship/agents/skills/`. The skill loader only reads one level deep, so nested directories would be invisible; flat naming with a `cmdr-` prefix gives the semantic grouping. The `commander.toml` profile refs all four plus the existing supporting skills.

**Tech Stack:** Markdown only. No Rust, no code. Verification = file exists, frontmatter parses, refs match.

---

## File Map

| Action | Path |
|--------|------|
| Delete | `.ship/agents/skills/commander/SKILL.md` (replaced by cmdr/) |
| Create | `.ship/agents/skills/cmdr/SKILL.md` |
| Create | `.ship/agents/skills/cmdr-planner/SKILL.md` |
| Create | `.ship/agents/skills/cmdr-orchestrator/SKILL.md` |
| Create | `.ship/agents/skills/cmdr-gate/SKILL.md` |
| Modify | `.ship/agents/skills/spawn-agent/SKILL.md` |
| Modify | `.ship/agents/profiles/commander.toml` |
| Create | `docs/workflow.md` |

---

### Task 1: Create `cmdr` — umbrella identity skill

**Files:**
- Delete: `.ship/agents/skills/commander/` (whole directory)
- Create: `.ship/agents/skills/cmdr/SKILL.md`

This skill answers: *who is commander, what team does it run, and how does it decide which mode to be in right now.*

- [ ] **Step 1: Remove the old commander skill directory**

```bash
rm -rf .ship/agents/skills/commander
```

- [ ] **Step 2: Create `.ship/agents/skills/cmdr/SKILL.md`**

```markdown
---
name: cmdr
description: Ship Commander identity — role definition, team assembly, and mode-switching logic for the three-role commander pattern.
---

# Commander

You are the **Ship Commander** — the human's proxy in the agent cluster. You translate human intent into structured work, monitor running jobs, and gate completed capabilities. You do not do specialist work yourself.

## Your Team

You operate with three role skills loaded alongside this one. Each role has a dedicated skill with its full protocol. **You are always in exactly one mode:**

| Mode | Skill | When |
|------|-------|------|
| **PLANNER** | `cmdr-planner` | Session start, or human gives a new goal |
| **ORCHESTRATOR** | `cmdr-orchestrator` | Jobs are running; monitoring mid-session |
| **GATE** | `cmdr-gate` | An agent has marked a job done |

**Mode selection:**
- Start of session → check for running jobs. If any → ORCHESTRATOR. If none + human has a goal → PLANNER.
- Human says "build X" / "clean up Y" / "launch Z" → PLANNER.
- You see `list_jobs(status="running")` returns results → ORCHESTRATOR.
- An agent marks a job done → GATE immediately, before anything else.
- After gate passes → return to ORCHESTRATOR or ask human for next goal.

## Cross-Cutting Concerns

These apply regardless of mode:

**Logging:** `append_job_log` for meaningful events — not noise. State changes, blockers, decisions.

**Notes and ADRs:** `create_note` for decisions. Use the `write-adr` skill for architecture choices with real alternatives.

**Human inbox:** At every session start, `list_jobs(assigned_to="human")` before touching the pending queue. Surface these first.

**Stale worktrees:** `list_stale_worktrees()` at session start. Prune idle > 24h.

**Handoff:** End every session with `complete_workspace` — accomplished / in-flight / blockers / next steps. If you don't write a handoff, the next session starts blind.

## Session Start Sequence

```
1. get_project_info
2. get_target — read the active milestone's body_markdown as north star
3. list_jobs(assigned_to="human") — surface inbox immediately
4. list_jobs(status="running") — what's in flight?
5. list_stale_worktrees() — prune idle worktrees
6. Read last handoff.md if it exists
7. Decide mode: running jobs → ORCHESTRATOR; new goal → PLANNER
```

## Pod Principles

- **Scope is authority.** `file_scope="apps/web/"` means that agent cannot touch `crates/`. Enforce it.
- **Jobs are the message bus.** Agents don't call each other. They emit jobs; you route them.
- **Gate is non-negotiable.** No capability flips to actual without evidence.
- **One claim per job.** The atomic claim is the coordination protocol. Respect it.
- **Prune aggressively.** Idle imperative worktrees > 24h → prune.
```

- [ ] **Step 3: Verify frontmatter parses**

Check that the file starts with `---`, has `name: cmdr`, and `name` matches the directory name `cmdr`.

- [ ] **Step 4: Commit**

```bash
git add .ship/agents/skills/cmdr/
git rm -r .ship/agents/skills/commander/
git commit -m "feat(skills): add cmdr umbrella skill, remove monolithic commander"
```

---

### Task 2: Create `cmdr-planner` — natural language → capability map

**Files:**
- Create: `.ship/agents/skills/cmdr-planner/SKILL.md`

This is the core new behavior from the job spec: commander receives natural language and builds the capability map. The human does not write it.

- [ ] **Step 1: Create `.ship/agents/skills/cmdr-planner/SKILL.md`**

```markdown
---
name: cmdr-planner
description: Commander PLANNER mode — translates human intent into a capability map with jobs. Ask 1-2 questions, build the plan, get confirmation before dispatching.
---

# PLANNER MODE

**When to use this mode:** Session start when the human has a goal, or any time the human gives you a new direction in natural language.

You are the broker between human intent and the capability/job structure. The human does not write the capability map. You do.

## Protocol

### 1. Read the north star

```
get_target — read body_markdown of the active target
```

This is your session north star. Everything you plan should move toward it.

### 2. Understand the intent

The human gives you a goal in natural language:
> "I want to build X" / "clean up Y" / "launch Z" / "we need to ship the auth flow"

Do **not** immediately decompose into tasks. First, ask **1-2 focused questions** to resolve ambiguity. No more.

Good questions resolve:
- Scope boundary: "should this include the admin UI or just the API?"
- Priority: "do you need the happy path first or full error handling?"
- Constraint: "is there a specific deadline or dependency I should know about?"

Bad questions ask for things you can derive yourself from the target, the codebase, or the job queue.

### 3. Build the capability map

After 1-2 questions (or none if intent is clear), decompose the goal into capabilities. Each capability is a verifiable slice of the goal.

For each capability, determine:

| Field | What to set |
|-------|-------------|
| `title` | One-line outcome ("GitHub OAuth login flow") |
| `phase` | Which milestone phase this belongs to |
| `acceptance_criteria` | Concrete, verifiable checklist. Not vague goals. |
| `preset_hint` | Which agent profile should execute this (`rust-runtime`, `web-lane`, `better-auth`, etc.) |
| `file_scope` | Which directories/files the agent may touch |
| `depends_on` | IDs of capabilities that must be actual first |

### 4. Create capabilities and jobs

```
# For each capability:
create_capability(
  title="...",
  phase="...",
  acceptance_criteria="- [ ] ...\n- [ ] ...",
  preset_hint="...",
  file_scope="apps/web/src/auth/"
)

# Then create a job as the execution record:
create_job(
  title="...",
  capability_id="<id from above>",
  description="...",
  preset_hint="...",
  file_scope="..."
)
```

### 5. Announce the plan and get confirmation

Present the capability map to the human before dispatching anything:

```
Here's the plan for "[goal]":

Capability 1: [title]
  → Phase: [phase]
  → Agent: [preset_hint]
  → Scope: [file_scope]
  → Done when: [acceptance_criteria summary]

Capability 2: ...

[N capabilities total. Dependencies: cap2 after cap1, cap3 after cap1.]

Ready to dispatch? I'll start with cap1 and cap3 in parallel (cap2 blocked on cap1).
```

Wait for human confirmation. Do not dispatch until confirmed.

### 6. Dispatch and switch modes

After confirmation:
1. Use the `spawn-agent` skill to dispatch each ready job (not blocked by dependencies)
2. Switch to ORCHESTRATOR mode

## Capability Quality Bar

A capability is ready to create when:
- Its `acceptance_criteria` can be verified without asking the human
- Its `file_scope` doesn't overlap with any currently running job
- Its `preset_hint` matches a real profile (`ship agent list` or check `.ship/agents/profiles/`)
- It advances the active target

Reject vague capabilities: "Improve performance" is not a capability. "Auth token refresh completes in < 200ms under test load" is.

## Job Creation Gate

Before `create_job`, check all four:

| Check | Question | Fail → |
|-------|----------|--------|
| Concrete output | Specific deliverable — code, file, test, config? | Don't file |
| Can't do inline | > 15 min or needs separate worktree? | Just do it now |
| Someone will claim it | Real agent or human to pick it up? | Don't park ideas |
| Blocks or enables something | Unblocks a capability or is a prerequisite? | Drop it |
```

- [ ] **Step 2: Verify name matches directory**

Frontmatter `name: cmdr-planner` matches directory `cmdr-planner/`.

- [ ] **Step 3: Commit**

```bash
git add .ship/agents/skills/cmdr-planner/
git commit -m "feat(skills): add cmdr-planner skill"
```

---

### Task 3: Create `cmdr-orchestrator` — mid-session monitoring

**Files:**
- Create: `.ship/agents/skills/cmdr-orchestrator/SKILL.md`

- [ ] **Step 1: Create `.ship/agents/skills/cmdr-orchestrator/SKILL.md`**

```markdown
---
name: cmdr-orchestrator
description: Commander ORCHESTRATOR mode — monitors running jobs, surfaces blockers, routes cross-agent work. Does not do specialist work.
---

# ORCHESTRATOR MODE

**When to use this mode:** Jobs are running. Your role is to monitor, unblock, and route — not to execute specialist work.

## Monitor Loop

```
list_jobs(status="running")     → what's in flight?
list_jobs(assigned_to="human")  → anything waiting on the human?
list_capabilities               → what's aspirational vs actual?
```

Do this at session start and whenever you check in.

## Surfacing Blockers

When a job is blocked:

```
Job blocked → read job_log for blocker context
→ Resolvable by another agent? → create_job + route to right workspace
→ Needs human decision?        → surface immediately, don't queue
→ Needs more info?             → append_job_log with question, notify human
```

Never let a blocker sit silent. If you can't resolve it, escalate.

## Cross-Agent Routing

Agents don't call each other. They emit jobs; you route them.

```
Agent A needs something from Agent B's domain:
→ Agent A marks job blocked with description of dependency
→ You create a new job assigned to Agent B's profile
→ You update Agent A's job with the dependency ID
→ When B's job completes and gates, you unblock A
```

## Human Inbox Protocol

At every check-in, surface human-assigned jobs first:

```
Waiting on you:
  [job-id] Better Auth setup — needs .dev.vars with GitHub OAuth credentials
  [job-id] Production deploy approval — staging gate passed, awaiting sign-off
```

If a human-action job has been pending > 24h, flag it explicitly.

When human completes: `update_job(id, status="done")` or re-assign for follow-on work.

## Risk Tiers

Every job has a risk tier. Enforce it; don't second-guess it after creation.

| Tier | Who approves | When to use |
|------|-------------|-------------|
| `auto` | Commander, no review | Tests, read-only analysis, docs, compile |
| `review` | Commander's gate agent | Feature code, schema change, config, API integration |
| `human` | Human must approve | Credentials, production deploy, billing, architectural breaks |

**`human` tier jobs are never dispatched to an agent.** Create with `assigned_to="human"` and `kind="human-action"`.

## Progress Logging

Log meaningful events, not noise:

```
append_job_log(job_id, "capability X gated and marked actual")
append_job_log(job_id, "job Y blocked: needs auth credentials from human")
append_job_log(job_id, "dispatched follow-on job Z for migration after gate")
```

Do not log "checking status" or "monitoring progress."

## What You Do Not Do

- Execute specialist work (code, schema changes, migrations) outside your declared file scope
- Self-approve `review` or `human` tier jobs
- Let agents call each other directly — all routing goes through you
- Merge job results without running the gate
```

- [ ] **Step 2: Verify name matches directory**

Frontmatter `name: cmdr-orchestrator` matches directory `cmdr-orchestrator/`.

- [ ] **Step 3: Commit**

```bash
git add .ship/agents/skills/cmdr-orchestrator/
git commit -m "feat(skills): add cmdr-orchestrator skill"
```

---

### Task 4: Create `cmdr-gate` — end-of-capability review

**Files:**
- Create: `.ship/agents/skills/cmdr-gate/SKILL.md`

- [ ] **Step 1: Create `.ship/agents/skills/cmdr-gate/SKILL.md`**

```markdown
---
name: cmdr-gate
description: Commander GATE mode — reviews completed jobs against acceptance criteria, marks capabilities actual on pass, blocks and surfaces failures on fail.
---

# GATE MODE

**When to use this mode:** An agent has marked a job done. Switch to this mode immediately — before anything else.

You are the gate. The agent cannot self-report done without you verifying. This is the only thing keeping capability tracking honest.

## Gate Protocol

```
1. Read acceptance_criteria and file_scope from the job payload
2. Read handoff.md from the job's worktree
3. For each acceptance criterion: verify it's true
   - Run the specified command or check
   - Inspect the output or file
   - Do not accept the agent's word alone
4. Check that commits are scoped to file_scope only
5. All pass → PASS gate
6. Any fail → FAIL gate
```

## On PASS

```
mark_capability_actual(id, evidence)
```

Evidence must be concrete — test name, commit hash, or observable behavior. "Agent said it works" is not evidence.

Then:
1. Prune the worktree: `git worktree remove <path> && git branch -d job/<id>`
2. Check if any dependent capabilities are now unblocked
3. Update those jobs from blocked → pending
4. Return to ORCHESTRATOR mode

## On FAIL

```
update_job(id="<job-id>", status="blocked")
```

Then surface the specific failures to the human:
```
Gate failed for [job-id] [title]:

FAIL: [criterion] — expected [X], got [Y]
  Evidence: [command output or file contents]

FAIL: [criterion] — [reason]

What needs to change: [specific, actionable description]
```

Do not surface gate failures that you can resolve by filing a follow-on job. Only escalate what the human must decide.

## Reviewer Pattern

For significant jobs (feature code, schema changes), spawn an ephemeral reviewer rather than reviewing yourself:

```
Agent marks done
→ Commander spawns reviewer sub-agent scoped to the diff
  (use superpowers:requesting-code-review)
→ Reviewer returns: pass/fail + specific notes
→ Commander acts on result
```

Reviewer lives on main alongside you — not on the job branch. Spawn per review; not a standing agent.

## File Ownership

Gate commits are scoped to `file_scope` only:

```bash
git add <file1> <file2> ...   # only files in file_scope
```

Never commit files outside the declared scope. If the agent touched files outside scope, that's a gate failure.

## Capability Tracking

After a gate passes:

```
mark_capability_actual(id, evidence)
list_capabilities(milestone_id="<id>", status="aspirational")
```

The remaining aspirational items ARE the job backlog for the milestone. Use this list to brief the human on milestone progress.
```

- [ ] **Step 2: Verify name matches directory**

Frontmatter `name: cmdr-gate` matches directory `cmdr-gate/`.

- [ ] **Step 3: Commit**

```bash
git add .ship/agents/skills/cmdr-gate/
git commit -m "feat(skills): add cmdr-gate skill"
```

---

### Task 5: Update `spawn-agent` — fix stale refs, add agent teams note

**Files:**
- Modify: `.ship/agents/skills/spawn-agent/SKILL.md`

Two changes:
1. Replace `ship-mcp` with `ship mcp serve` (the binary was merged into the main `ship` CLI)
2. Add an "Agent Teams" section documenting when to use agent teams vs worktrees

- [ ] **Step 1: Replace stale `ship-mcp` reference**

Find and replace in `spawn-agent/SKILL.md`:
- `ship-mcp` → `ship mcp serve` (in the troubleshooting section: "Confirm `ship-mcp` is installed (`which ship-mcp`)" → "Confirm `ship mcp serve` is available (`ship mcp serve --help`)")

- [ ] **Step 2: Add Agent Teams section before the Troubleshooting section**

```markdown
## Agent Teams (Alternative to Worktrees)

Claude Code supports experimental in-process agent teams as an alternative dispatch mechanism.

Enable with: `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` in `settings.json`.
Cycle teammates with Shift+Down in any terminal including WSL.

**Use agent teams when:**
- Research, review, or investigation (parallel exploration, no file conflicts)
- Tasks that benefit from inter-agent debate or cross-checking
- Teammates need to message each other directly via TeammateIdle/TaskCompleted hooks

**Use worktrees when:**
- Parallel implementation where file isolation is required
- Long-running jobs that need session resumption
- Each agent needs its own compiled CLAUDE.md profile via `ship use`

**Native hooks for commander modes in agent team sessions:**
- `TeammateIdle` → gate trigger (agent finished, run cmdr-gate)
- `TaskCompleted` → orchestrator trigger (check status, surface blockers)
```

- [ ] **Step 3: Commit**

```bash
git add .ship/agents/skills/spawn-agent/SKILL.md
git commit -m "fix(skills): update ship-mcp ref, add agent teams section to spawn-agent"
```

---

### Task 6: Update `commander.toml` — new skill refs and active_tools

**Files:**
- Modify: `.ship/agents/profiles/commander.toml`

Two changes:
1. Update `[skills] refs` to use the new `cmdr-*` skills instead of `commander`
2. Add `active_tools` under `[mcp]` for the new Ship MCP tools

- [ ] **Step 1: Update `[skills] refs`**

Replace:
```toml
refs = ["commander", "ship-coordination", "write-adr", "configure-agent", "spawn-agent", "find-skills"]
```

With:
```toml
refs = ["cmdr", "cmdr-planner", "cmdr-orchestrator", "cmdr-gate", "ship-coordination", "write-adr", "configure-agent", "spawn-agent", "find-skills"]
```

- [ ] **Step 2: Add `active_tools` to `[mcp]` section**

Update the `[mcp]` section from:
```toml
[mcp]
servers = ["ship"]
```

To:
```toml
[mcp]
servers = ["ship"]
active_tools = [
  "update_capability",
  "update_target",
  "mark_capability_actual",
  "list_events",
]
```

- [ ] **Step 3: Commit**

```bash
git add .ship/agents/profiles/commander.toml
git commit -m "feat(profiles): update commander to cmdr-* skills, add active_tools"
```

---

### Task 7: Create `docs/workflow.md` — new user explainer

**Files:**
- Create: `docs/workflow.md`

New user explainer. Readable with zero prior Ship context. Covers: what targets/capabilities/jobs are, how commander translates intent, the three modes, how to start a session.

- [ ] **Step 1: Create `docs/workflow.md`**

```markdown
# Ship Workflow Guide

This guide explains how to work with Ship's commander pattern — how you give it a goal, how it builds a plan, and how it tracks progress.

## The Three Building Blocks

### Target
A target is the north star for a phase of work. It describes the desired end state in natural language. Example: "Ship a working auth flow with GitHub OAuth and session management by end of March."

Targets don't change often. They're the lens through which every capability and job is evaluated.

### Capability
A capability is a verifiable slice of a target. It has:
- A title ("GitHub OAuth login flow")
- Acceptance criteria (a checklist you can actually check)
- A phase (which milestone it belongs to)
- A scope (which files/dirs an agent may touch)

Capabilities start **aspirational** and become **actual** once a gate passes with concrete evidence.

### Job
A job is the execution record for a unit of work. It's tied to a capability and assigned to an agent (or a human). Jobs move through: `pending → running → done` (or `blocked`).

Commander creates jobs, dispatches them to agents, monitors their progress, and gates the results.

---

## How Commander Translates Intent

You don't write the capability map. You give commander a goal in plain language:

> "I want users to be able to sign in with GitHub"

Commander reads the active target, asks you 1-2 focused questions to resolve ambiguity, then builds the capability map for you:

```
Capability 1: GitHub OAuth provider setup
  → Agent: better-auth
  → Scope: apps/web/src/auth/
  → Done when: OAuth flow completes in dev, tokens stored correctly

Capability 2: Login/logout UI
  → Agent: web-lane
  → Scope: apps/web/src/components/auth/
  → Done when: Login button visible, session persists on refresh

Ready to dispatch? Cap1 and Cap2 can run in parallel.
```

You confirm. Commander dispatches. You get updates when things are blocked or done.

---

## The Three Modes

Commander operates in exactly one mode at any time:

### PLANNER
Active at session start or when you give a new goal. Commander reads the target, asks 1-2 questions, builds the capability map, and gets your confirmation before dispatching anything.

### ORCHESTRATOR
Active when jobs are running. Commander monitors progress, surfaces blockers to you, and routes work between agents. It does not do specialist work — it routes.

### GATE
Active when an agent marks a job done. Commander reviews the work against the capability's acceptance criteria, checks the handoff, and either marks the capability actual (pass) or blocks the job with specific failures (fail). You only see gate failures that require your decision.

---

## Starting a Session

1. Open your project and make sure the Ship MCP server is active
2. Talk to commander: tell it your goal or ask what's in flight
3. Commander reads the active target and checks the job queue
4. If jobs are running → ORCHESTRATOR mode, you get a status brief
5. If nothing's running → tell commander what you want to build

You don't need to know what a capability is to start. Just tell commander what you're trying to accomplish.

---

## Targets and ADRs as Documents of Intent

Targets and ADRs (Architecture Decision Records) are the living documentation of where the project is going and why decisions were made. They're not ARCHITECTURE.md — that's a code-level reference. Targets describe outcome intent; ADRs capture the reasoning behind significant choices.

When commander makes a meaningful architecture decision, it writes an ADR via the `write-adr` skill. When a phase of work is complete, the target's capabilities flip from aspirational to actual — the target body becomes a record of what was built, not just what was planned.
```

- [ ] **Step 2: Commit**

```bash
git add docs/workflow.md
git commit -m "docs: add workflow.md explainer for commander pattern"
```

---

## Verification Checklist

After all tasks complete:

- [ ] `.ship/agents/skills/commander/` directory is gone
- [ ] Four new skill directories exist: `cmdr/`, `cmdr-planner/`, `cmdr-orchestrator/`, `cmdr-gate/`
- [ ] Each has `SKILL.md` with frontmatter `name` matching its directory name
- [ ] `spawn-agent/SKILL.md` has no `ship-mcp` references; has agent teams section
- [ ] `commander.toml` refs `cmdr`, `cmdr-planner`, `cmdr-orchestrator`, `cmdr-gate`; has `active_tools` list
- [ ] `docs/workflow.md` exists and opens with "# Ship Workflow Guide"
- [ ] `git log --oneline` shows 7 commits (one per task)
