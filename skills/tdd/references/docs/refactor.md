---
title: Refactor Phase
description: Improving code quality without changing behavior.
section: phases
order: 3
---

# Refactor Phase

All tests are green. Now make the code better without changing what it does.

## Rules

- Run ALL tests before starting — they must pass
- Run ALL tests after every change — they must still pass
- If a test fails after your change, revert immediately
- You may edit any file (tests and implementation)
- Do NOT add new behavior, new test cases, or new dependencies

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
- Write new tests (that's Red's job)

## Process

1. Run tests — confirm green
2. Pick one improvement
3. Make the change
4. Run tests — confirm still green
5. Commit
6. Repeat or stop

## Phase lock

To constrain an agent to Refactor only:

```
ship vars set tdd phase refactor
```
