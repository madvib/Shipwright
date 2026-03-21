---
name: tdd
description: Test-driven development protocol — write the failing test first, then the minimal code to pass it. Use for any feature, fix, or behavior change.
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

## Rust

```bash
cargo test <test_name>        # run one test
cargo test                    # run all
cargo test -- --nocapture     # see println output
```

## TypeScript / Vitest

```bash
pnpm test <test_file>         # run one file
pnpm test                     # run all
pnpm test --reporter=verbose  # see each test name
```
