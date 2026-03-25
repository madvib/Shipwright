---
name: mission-control
description: Use when coordinating multiple agents across worktrees without Ship's MCP server. File-based job tracking, three-mode operation (plan, dispatch, gate), and structured handoffs. Works with any AI provider that supports Ship.
tags: [coordination, orchestration, workflow]
authors: [ship]
---

# Mission Control

Coordinate a team of specialist agents using git worktrees and markdown files. No MCP server required — all state lives in `.ship-session/`.

## Three Modes

You are always in exactly one mode:

| Mode | When |
|------|------|
| **PLAN** | Human gives a goal. Decompose, scope, confirm. |
| **DISPATCH** | Plan confirmed. Create worktrees, write specs, launch agents. |
| **GATE** | Agent signals done. Review diff against acceptance criteria. |

## Session State

All coordination state lives in `.ship-session/` at the project root. Create it on first use.

```
.ship-session/
  plan.md            # current work plan (capabilities, scope, dependencies)
  jobs/
    <slug>.md        # one file per dispatched job (spec + status in frontmatter)
  log.md             # append-only decisions and state changes
  handoff.md         # end-of-session summary for next session pickup
```

## PLAN Mode

When the human gives a goal:

1. **Read context.** Scan the codebase for relevant files. Read `.ship-session/handoff.md` if it exists from a prior session.
2. **Ask 0-2 questions.** Only if genuine ambiguity. Do not ask what you can read.
3. **Decompose into jobs.** Each job is a concrete deliverable with:
   - Title (what ships)
   - Agent (which specialist)
   - File scope (what they may touch)
   - Acceptance criteria (how you verify it's done)
   - Dependencies (what must be true first)
4. **Check for scope conflicts.** No two parallel jobs may share files.
5. **Present the plan and wait for confirmation.** Never dispatch before the human says go.

Write the plan to `.ship-session/plan.md`.

## DISPATCH Mode

After human confirms:

For each ready (unblocked) job:

```bash
# 1. Write the job spec
mkdir -p .ship-session/jobs
cat > .ship-session/jobs/<slug>.md << 'SPEC'
---
status: dispatched
agent: <agent-id>
scope: <file paths>
dispatched: <ISO timestamp>
---

# <Job Title>

## Goal
<what to build>

## Scope
<files/directories the agent may touch>

## Acceptance Criteria
- [ ] <concrete, verifiable>
- [ ] <concrete, verifiable>

## Constraints
- Do NOT touch <off-limits files>
- <other constraints>

## Dependencies
<what must already be true>
SPEC

# 2. Create worktree and launch
bash .ship/skills/mission-control/dispatch.sh --slug <slug> --agent <agent-id> --spec .ship-session/jobs/<slug>.md
```

Log each dispatch to `.ship-session/log.md`:
```
## <timestamp> — Dispatched <slug>
Agent: <agent-id>, Scope: <files>, Blocked by: <none|other-slug>
```

## GATE Mode

When an agent signals done (handoff.md exists in their worktree, or they tell you):

1. **Read the job spec** from `.ship-session/jobs/<slug>.md`
2. **Read the agent's handoff** from their worktree
3. **Check each acceptance criterion:**
   - Run the test, inspect the file, verify the behavior
   - Do not accept "I did it" — verify independently
4. **Check scope:** `git diff --name-only main..job/<slug>` — any files outside declared scope = fail
5. **Verdict:**

**PASS:**
```bash
# Update job status
# (edit frontmatter of .ship-session/jobs/<slug>.md to status: passed)

# Merge the work
git checkout <base-branch>
git merge job/<slug> --no-ff -m "merge: <slug> — <title>"

# Clean up
git worktree remove <worktree-path>
git branch -d job/<slug>
```

Log: `## <timestamp> — Gate PASSED: <slug>`

**FAIL:**
```
Update .ship-session/jobs/<slug>.md status to: blocked

Log specifically what failed:
## <timestamp> — Gate FAILED: <slug>
- FAIL: <criterion> — expected X, got Y
- What needs to change: <specific, actionable>
```

Surface failures to the human. If fixable by filing a follow-on job, do that.

## Cross-Agent Routing

Agents don't talk to each other. They talk to you.

When an agent needs work outside their scope:
1. They note it in their handoff or tell you directly
2. You create a new job spec in `.ship-session/jobs/`
3. You dispatch it to the right specialist
4. You update the blocked job's spec with the dependency

## Session Handoff

At the end of every session, write `.ship-session/handoff.md`:

```markdown
# Handoff — <date>

## Accomplished
- <what shipped, with evidence>

## In Flight
- <slug>: <status, what's left>

## Blocked
- <slug>: <what's blocking, who needs to act>

## Next Session
- <what to do first>
```

The next session (yours or someone else's) reads this first.

## Risk Tiers

| Tier | Who reviews | Jobs like |
|------|------------|-----------|
| `auto` | You review inline | Tests, docs, analysis, config |
| `review` | Spawn a separate reviewer agent | Feature code, schema changes, API work |
| `human` | Human must approve | Credentials, deploys, billing, architecture breaks |

`human` tier jobs are never dispatched. Write the spec, mark `status: waiting-on-human`, surface it.
