---
name: tdd
stable-id: tdd
description: Use when implementing any feature, fix, or behavior change. Enforces the TDD protocol — write the failing test first, then the minimal code to pass it.
tags: [testing, workflow, engineering]
authors: [ship]
---

# TDD

Write the test first. Watch it fail. Write the minimum code to pass. Refactor.

If you didn't watch it fail, you don't know if it tests the right thing.

## The Loop

```
1. Write a failing test for the specific behavior
2. Run it — confirm it fails for the right reason
3. Write the minimum production code to make it pass
4. Run it — confirm it passes
5. Refactor if needed — tests must still pass
6. Commit
7. Repeat
```

**No production code before a failing test.** No exceptions.

## What "Minimum Code" Means

The smallest change that makes the test pass. Not the full implementation — just enough for this test. The next test drives the next piece.

## Commit Cadence

Commit at green. Each passing test is a stable checkpoint. Small commits — one behavior at a time.

## When to Use

- New features
- Bug fixes (write a test that reproduces the bug first)
- Behavior changes

Skip for: config files, generated code, throwaway prototypes.

## Running Tests

Detect the project's test framework automatically:

```bash
bash scripts/detect-test-runner.sh <project-root>
```

Detection priority:
1. `Cargo.toml` → `cargo test`
2. `package.json` with `vitest` → `npx vitest run`
3. `package.json` with `jest` → `npx jest`
4. `pyproject.toml` with `pytest` → `pytest`
5. `go.mod` → `go test ./...`
6. `Makefile` with `test:` target → `make test`
7. None detected → ask the user

Run a single test by appending the test name or file to the detected command. For framework-specific flags:

| Framework | Run one test | Verbose output |
|-----------|-------------|----------------|
| cargo | `cargo test <name>` | `cargo test -- --nocapture` |
| vitest | `npx vitest run <file>` | `npx vitest run --reporter=verbose` |
| jest | `npx jest <file>` | `npx jest --verbose` |
| pytest | `pytest <file>` | `pytest -v` |
| go | `go test -run <name> ./...` | `go test -v ./...` |
| make | `make test` | `make test VERBOSE=1` |
