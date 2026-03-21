---
name: Review Checklist
description: Structured code review process covering correctness, architecture, and maintainability
tags: [review, code-quality, checklist, pull-request]
---

# Review Checklist

## Review Process

Read the full diff before writing any comments. Understand the intent of the change before evaluating the implementation.

### Step 1: Understand the Change

- Read the PR description and linked issue
- Identify the scope: is this a new feature, bug fix, refactor, or config change?
- Check if the change matches what was described

### Step 2: Correctness

| Check | What to Look For |
|-------|-----------------|
| Logic errors | Off-by-one, wrong comparison operator, missing negation |
| Edge cases | Empty inputs, null values, boundary values, concurrent access |
| Return values | Are all paths returning the correct type and value? |
| Error handling | Are errors caught, propagated, or logged appropriately? |
| Type safety | Any type coercions, unsafe casts, or `any` types? |
| State mutations | Does this mutate shared state? Is it thread-safe? |

### Step 3: Architecture

- Does the change follow the project's architecture patterns?
- Is logic in the right layer (domain vs transport vs persistence)?
- Are new dependencies justified and minimal?
- Does the file organization follow existing conventions?

### Step 4: Test Coverage

```
Changed behavior? --> Must have tests
  Bug fix? --> Must have regression test (test that fails without the fix)
  New feature? --> Happy path + at least one error path
  Refactor? --> Existing tests must still pass (no new tests needed unless behavior changed)
  Config change? --> No tests needed
```

### Step 5: Readability

- Are variable and function names clear and consistent with the codebase?
- Is the code self-documenting, or does it need comments for non-obvious logic?
- Are complex operations broken into named steps?

## Severity Definitions

### Critical

The change will cause a bug, security vulnerability, data corruption, or crash in production. Must fix before merge.

Examples:
- SQL injection via unsanitized input
- Missing null check that will throw at runtime
- Race condition in concurrent code
- Missing authentication on a protected endpoint

### Warning

The change works but has a meaningful problem that should be addressed. Acceptable to merge with a follow-up if time-constrained.

Examples:
- Missing error handling for a network call
- N+1 query that will be slow at scale
- Missing test for a significant code path
- Inconsistent naming with rest of codebase

### Nit

Style or minor improvements. Never block a merge for nits.

Examples:
- Slightly better variable name
- Comment that could be clearer
- Import ordering

## Review Comment Format

```
[CRITICAL] file.ts:42 — Missing auth check on delete endpoint.
Risk: Any authenticated user can delete any record.
Fix: Add ownership check before deletion.

[WARNING] service.ts:85 — No error handling for API call.
Risk: Unhandled rejection will crash the process in Node.
Suggestion: Wrap in try/catch, return error result.

[NIT] utils.ts:12 — `data` could be named `userRecords` for clarity.
```

## Common Mistakes Reviewers Make

| Mistake | Why It Is Wrong | What to Do Instead |
|---------|----------------|-------------------|
| Reviewing existing code, not the diff | Scope creep, unfair to author | Review only what changed |
| Blocking on style preferences | Wastes time, creates friction | Use automated formatters |
| Suggesting rewrites of working code | Author knows the context better | Ask why, do not prescribe |
| Skipping test review | Tests are the specification | Review tests with same rigor as code |
| Approving without reading | Rubber stamp defeats the purpose | Request more time if needed |

## Final Verdict

After completing the checklist:

- **PASS**: No critical issues. Warnings are minor and tracked.
- **FAIL**: One or more critical issues. List them with specific line references and fix suggestions.
- **PASS WITH COMMENTS**: No blockers, but warnings that the author should address before or after merge.
