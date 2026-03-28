---
name: refactor
stable-id: refactor
description: Use in REFACTOR phase of TDD — improve code quality without changing behavior. All tests must pass before and after.
tags: [tdd, refactoring, workflow]
authors: [ship]
---

# Refactor Phase

All tests are green. Now make the code better without changing what it does.

## Rules

- Run ALL tests before you start — they must pass
- Run ALL tests after every change — they must still pass
- If a test fails after your change, REVERT immediately
- You may edit any file (tests and implementation)
- You may NOT add new behavior or new test cases

## What to improve

- Remove duplication
- Improve naming (variables, functions, files)
- Simplify complex conditionals
- Extract functions that do too much
- Remove dead code
- Fix formatting inconsistencies
- Reduce nesting depth

## What NOT to do

- Add features
- Add error handling for untested cases
- Change public APIs
- Add new dependencies
- Write new tests (that's Red's job)

## Process

1. Run tests — confirm green
2. Pick one improvement
3. Make the change
4. Run tests — confirm still green
5. Commit
6. Repeat or stop
