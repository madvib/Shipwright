---
name: cmdr-gate
stable-id: cmdr-gate
description: Review a completed job branch against acceptance criteria. Returns pass/fail with evidence. Run as subagent or inline.
tags: [gate, review, quality]
authors: [ship]
---

# Gate Review

Review a completed job branch against its acceptance criteria. Return a clear pass/fail verdict with evidence.

## Protocol

```
1. Read the job spec (acceptance criteria + file scope)
2. Read handoff.md from the worktree
3. For each acceptance criterion:
   - Run the check or inspect the output
   - Do not accept the agent's word alone
4. Verify commits are scoped to file_scope only
5. Run automated checks (below)
6. All pass → PASS with evidence
   Any fail → FAIL with specific evidence
```

## Verdict Format

### PASS

```
GATE PASS: [job title]
Branch: job/[slug]

Evidence:
  ✓ [criterion 1] — [test name / commit hash / observable behavior]
  ✓ [criterion 2] — [evidence]
  ...

Automated checks: all passed
Ready to merge.
```

### FAIL

```
GATE FAIL: [job title]
Branch: job/[slug]

  ✓ [criterion 1] — passed
  ✗ [criterion 2] — expected [X], got [Y]
    Evidence: [command output or diff]
  ✗ [criterion 3] — [specific reason]

What needs to change: [actionable, specific]
```

## Automated Checks

Run all four. Any failure blocks the gate.

### 1. Build verification

```bash
just build
```

Must exit 0.

### 2. Test verification

```bash
cargo test -p <relevant-crate>
```

Must exit 0.

### 3. Scope violation check

```bash
git diff main...HEAD --name-only
```

Every changed file must be within the declared `file_scope`. Files outside scope → FAIL.

### 4. Silent fallback scan (Rust only)

Scan non-test code for silent error suppression:

```bash
grep -rn --include="*.rs" \
  -e 'unwrap_or_else(|_|' \
  -e 'unwrap_or_default()' \
  -e 'unwrap_or(' \
  -e '\.unwrap()' \
  src/ \
  | grep -v '#\[cfg(test)\]' \
  | grep -v 'mod tests'
```

Each match needs a justifying comment (e.g., `// infallible: static data`). Unjustified matches → FAIL.

### 5. Abandoned problem detection

```bash
git diff main...HEAD -- '*.rs' '*.ts' '*.tsx' \
  | grep '^+' \
  | grep -iE 'TODO.*error|TODO.*broken|FIXME.*error|FIXME.*broken|not my fault|leaving.?alone'
```

A noticed problem not filed as a follow-on job is a FAIL. The fix: file the job, then re-gate.

### 6. MCP config verification

```bash
cat <worktree>/.mcp.json
```

Must exist and contain `ship mcp serve`. An agent without MCP cannot have logged progress properly.

## Land Step (on PASS)

Gate owns the merge. Commander never touches git.

```bash
# 1. Stage only files within file_scope
git -C <worktree-path> add <file1> <file2> ...

# 2. Commit with message derived from spec Goal
git -C <worktree-path> commit -m "<type>: <goal summary>"

# 3. Merge into target branch (default: v0.2.0)
git checkout <target-branch>
git merge --no-ff job/<slug> -m "merge: job/<slug>"

# 4. Clean up worktree and branch
git worktree remove <worktree-path>
git branch -d job/<slug>
```

Report back to commander: `LANDED: <commit-hash>`

If any step fails, report `LAND FAILED: <reason>` — do not leave partial state.
Commander does not retry the land; human decides next action.

## Signal commander on completion

After every verdict (PASS+land or FAIL), notify commander so it can act without human relay:

1. Call `mcp__ship__mesh_broadcast` with:
   ```json
   { "topic": "gate.result", "verdict": "LANDED|GATE FAIL", "slug": "<slug>", "detail": "<one line>" }
   ```
2. If the job spec includes a `commander_workspace_id` field, also call `mcp__ship__emit_studio_event` with:
   ```json
   {
     "event_type": "studio.gate.result",
     "target_workspace_id": "<commander_workspace_id>",
     "payload": { "verdict": "LANDED|GATE FAIL", "slug": "<slug>", "detail": "<one line>" }
   }
   ```
   This writes to the commander workspace's inbox and triggers the MCP resource notification that wakes the commander session.

If shipd is not running, skip step 1. Step 2 is best-effort — never block on it.
