---
name: cmdr-gate
stable-id: cmdr-gate
description: Commander GATE mode — reviews completed jobs against acceptance criteria, marks capabilities actual on pass, blocks with specific evidence on fail.
---

# GATE MODE

**When:** A job shows all three completion signals — `status="complete"`, `handoff.md` present, `complete:` commit on the branch. Switch to this mode immediately when detected.

**Commander does NOT do gate review directly for `review`-tier jobs.** For feature code, schema changes, config, or API integrations — spawn a Gate agent (separate session scoped to the diff). Commander acts on the result. Only `auto`-tier jobs (docs, analysis, compile) may be reviewed inline.

The gate agent cannot self-report done without verifying. This is the only thing keeping capability tracking honest.

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

## Gate Agent Pattern

For `review`-tier jobs, spawn a Gate agent rather than reviewing yourself:

```
Agent marks done
→ Commander spawns Gate agent scoped to the job branch diff
  (read job spec + acceptance_criteria, run checks, return pass/fail)
→ Gate agent returns verdict + evidence
→ Commander acts on result: merge on pass, block + surface specific failures on fail
```

Gate agent lives on main — not on the job branch. Spawn per review, ephemeral. It has read access to the worktree; commander does not.

## File Ownership

Gate commits are scoped to `file_scope` only:

```bash
git add <file1> <file2> ...   # only files in declared scope
git commit -m "gate: [job-id] [title]"
```

Agent touched files outside scope → gate fails. Investigate before deciding whether to expand scope or send back.

## Rust Silent Fallback Check

For any job touching Rust source files, scan non-test code for silent error suppression patterns.

```bash
# Run from the job worktree root
# Exclude test modules and #[cfg(test)] blocks
grep -rn --include="*.rs" \
  -e 'unwrap_or_else(|_|' \
  -e 'unwrap_or_default()' \
  -e 'unwrap_or(' \
  -e '\.unwrap()' \
  src/ \
  | grep -v '#\[cfg(test)\]' \
  | grep -v 'mod tests'
```

**PASS:** Zero matches in non-test code, OR each match is accompanied by a comment explaining why the value is guaranteed (e.g., `// infallible: static data`).

**FAIL:** Any silent substitution or panic-on-unwrap in non-test code with no justification comment. List each offending line as evidence.

`unwrap_or_default()` and `unwrap_or(` substitute silently — they hide errors from callers. `unwrap()` panics — it is a visible crash, but it is still a gate fail unless the comment justifies it.

## "Leaving It Alone" Detection

Scan all changed files (diff against main) for comments or log strings that signal a noticed-but-ignored problem.

```bash
git diff main...HEAD -- '*.rs' '*.ts' '*.tsx' '*.md' \
  | grep '^+' \
  | grep -iE \
    'not my fault|leaving.?alone|out of scope|TODO.*error|TODO.*broken|FIXME.*error|FIXME.*broken|FIXME.*skip'
```

**PASS:** Zero matches.

**FAIL:** Any match. A noticed problem that is not filed as a follow-on job is a gate FAIL. The fix: file a job for the problem, then re-gate. "Not my fault" is never evidence of correctness.

## .mcp.json Verification

An agent that ran without MCP cannot have properly logged progress or read directives from the runtime.

```bash
# Check file exists in worktree root
cat <worktree>/.mcp.json
```

**PASS:** File exists and contains a `ship` server entry with `ship mcp serve` in its args array. Example shape:

```json
{
  "mcpServers": {
    "ship": {
      "command": "ship",
      "args": ["mcp", "serve"]
    }
  }
}
```

**FAIL:** File absent, malformed JSON, no `ship` key, or `ship mcp serve` not present in args. Surface the actual file contents (or absence) as evidence.

## Test/Implementation Order Check (Feature Jobs)

For feature jobs, tests must be committed before implementation. This verifies TDD discipline.

```bash
# List commits on the branch in order (oldest first)
git log --oneline --reverse main..HEAD
```

Classify each commit:
- **test commit** — subject starts with `test:`, or diff adds only `*_test.rs` / `*.test.ts` / `*.spec.ts` files
- **impl commit** — adds non-test source files

**PASS:** The first test commit precedes the first impl commit in the log, OR the job is not a feature job (patch, docs, chore, refactor).

**FLAG (not auto-fail):** Tests and implementation land in the same commit. Surface this explicitly:

```
NOTE: test/impl separation — tests and implementation committed together in [hash].
TDD discipline requires tests first. Not blocking this gate, but flag for the next job.
```

**FAIL:** Implementation commit appears before any test commit with no explanation in the handoff.

## Multi-Commander Coordination

If another commander already claimed a gate review (check `claimed_by` on the job), skip it — they have it. Only gate jobs you claimed.
