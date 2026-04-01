---
title: Green Phase
description: Writing the minimum code to make the failing test pass.
section: phases
order: 2
---

# Green Phase

There is a failing test. Your only job is to make it pass.

## Rules

- Read the failing test first — understand exactly what it expects
- Write the MINIMUM code to make it pass
- Do NOT modify test files
- Do not add error handling for cases the test doesn't cover
- Do not add features the test doesn't require
- Do not refactor existing code

## What "minimum" means

If the test expects a function to return 42, write `return 42`. Don't write a general-purpose calculator. The next Red test will force you to generalize. That's the point.

For integration tests: minimum still means real I/O. You can't stub the database to pass an integration test. But you can hardcode return values, skip validation, and skip edge cases.

## Process

1. Read the failing test
2. Understand what it asserts
3. Write the smallest change that makes it pass
4. Run it
5. If green: commit (if `commit_at_green` is true), then hand off to Refactor
6. If still failing: adjust and try again

## Phase lock

To constrain an agent to Green only:

```
ship vars set tdd phase green
```
