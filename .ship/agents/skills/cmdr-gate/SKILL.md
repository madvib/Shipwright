---
name: cmdr-gate
description: Commander GATE mode — reviews completed jobs against acceptance criteria, marks capabilities actual on pass, blocks with specific evidence on fail.
---

# GATE MODE

**When:** A job shows all three completion signals — `status="complete"`, `handoff.md` present, `complete:` commit on the branch. Switch to this mode immediately when detected.

You are the gate. The agent cannot self-report done without you verifying. This is the only thing keeping capability tracking honest.

## Gate Protocol

```
1. Read acceptance_criteria and file_scope from the job payload
2. Read handoff.md from the job's worktree
3. For each acceptance criterion:
   - Run the check or inspect the output/file
   - Do not accept the agent's word alone
4. Verify commits are scoped to file_scope only
5. All pass → PASS
   Any fail → FAIL
```

## On PASS

```
mark_capability_actual(id, evidence)
```

Evidence must be concrete: test name, commit hash, observable behavior. "Agent said it works" is not evidence.

Then:
1. Prune the worktree: `git worktree remove <path> && git branch -d job/<id>`
2. Check which dependent capabilities are now unblocked — update those jobs to `pending`
3. `list_capabilities(milestone_id="<id>", status="aspirational")` — remaining items are the job backlog
4. Return to ORCHESTRATOR mode

## On FAIL

```
update_job(id="<job-id>", status="blocked")
```

Surface specific failures to the human:

```
Gate failed for [job-id] [title]:

FAIL: [criterion] — expected [X], got [Y]
  Evidence: [command output or diff]

FAIL: [criterion] — [specific reason]

What needs to change: [actionable, specific]
```

Only escalate failures the human must decide. If you can resolve by filing a follow-on job, do that instead.

## Reviewer Pattern

For significant jobs (feature code, schema changes), spawn an ephemeral reviewer rather than reviewing yourself:

```
Agent marks done
→ Spawn reviewer sub-agent scoped to the diff
  (use superpowers:requesting-code-review)
→ Reviewer returns pass/fail + notes
→ You act on the result
```

Reviewer lives on main — not on the job branch. Spawn per review, not standing.

## File Ownership

Gate commits are scoped to `file_scope` only:

```bash
git add <file1> <file2> ...   # only files in declared scope
git commit -m "gate: [job-id] [title]"
```

Agent touched files outside scope → gate fails. Investigate before deciding whether to expand scope or send back.

## Multi-Commander Coordination

If another commander already claimed a gate review (check `claimed_by` on the job), skip it — they have it. Only gate jobs you claimed.
