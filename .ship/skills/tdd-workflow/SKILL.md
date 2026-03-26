---
name: tdd-workflow
stable-id: tdd-workflow
description: Use when building a feature or fixing a bug through test-first development. Interactive loop — write tests WITH the user to build a spec, then dispatch implementation. Not just "make tests pass" but collaborative test design.
tags: [testing, workflow, tdd, spec]
authors: [ship]
---

# TDD Workflow

Build specs through tests. The tests ARE the spec — each one captures a decision.

## The Loop

This is not "write tests then implement." This is an interactive conversation where tests crystallize requirements.

### Phase 1: Understand (1-2 minutes)

Read the goal. Ask ONE question if ambiguous. Then propose the first test:

```
Based on [goal], here's the first behavior to pin down:

  test: "returns 404 when agent ID doesn't exist"
  why:  establishes the error contract before the happy path

Want to start here, or is there a more fundamental behavior?
```

### Phase 2: Test Together (the core loop)

For each test:

1. **Propose the test** — name, assertion, why this behavior matters
2. **User confirms, refines, or redirects** — they might say "actually test X first"
3. **Write the test** — run it, confirm it fails for the right reason
4. **Discuss what it implies** — "this test means we need a lookup function. Should it hit the DB directly or go through a cache?"
5. **Write the next test** based on what we learned

Each test is a micro-decision. The conversation surfaces edge cases:

```
Test 3 passes. That gives us: create, read, not-found.

Next I'd test: "updating an agent preserves fields not included in the patch"
This pins down PATCH vs PUT semantics. Which do we want?
```

### Phase 3: Spec Checkpoint

After 5-8 tests, pause and summarize:

```markdown
## Spec (from tests)

Behaviors pinned:
- [x] Create agent with required fields (name, id, providers)
- [x] Read agent by ID returns full config
- [x] Missing ID returns 404 with actionable message
- [x] Update is PATCH (preserves unset fields)
- [x] Delete is idempotent (deleting missing ID succeeds)

Open questions:
- Should list return all agents or paginate?
- Do we need an "active" flag?

Next tests to write:
- Validation (bad ID format, duplicate name)
- Permission checks (can agent X modify agent Y?)
```

Write this to `.ship-session/spec-from-tests.md`.

### Phase 4: Implement

Once the user says "spec is good, implement it":

Option A — implement inline (small features):
```
Run each test, write minimum code to pass, refactor at green.
```

Option B — dispatch to a worktree (larger features):
```bash
bash scripts/dispatch.sh \
  --slug impl-agent-crud \
  --agent rust-runtime \
  --spec .ship-session/spec-from-tests.md
```

The implementation agent gets the tests + spec. Their job: make all tests green.

### Phase 5: Verify

After implementation:
- Run the full test suite
- Check coverage — any test from Phase 2 that isn't green is a failure
- The tests are the acceptance criteria. Nothing else needed.

## Anti-patterns

- **"Let me write all the tests first"** — No. One at a time. Each test is a conversation.
- **"The test is obvious"** — Say it anyway. Obvious tests catch the assumptions that feel safe.
- **"Let me implement first and backfill tests"** — This skill doesn't do that. Tests first or use a different agent.
- **"Test the implementation"** — Test the BEHAVIOR. The implementation can change; the behavior contract is what matters.

## When to use

- New feature (build the spec through tests)
- Bug fix (reproduce the bug as a test, THEN fix)
- Refactor (write characterization tests for current behavior, THEN change)

## When NOT to use

- Config changes (no behavior to test)
- UI polish (visual, not behavioral)
- Prototyping (throw it away anyway)
