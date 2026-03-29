---
name: green
stable-id: green
description: Use in GREEN phase of TDD — write the minimum implementation code to make the failing test pass. Do not refactor.
tags: [tdd, implementation, workflow]
authors: [ship]
---

# Green Phase

There is a failing test. Your only job is to make it pass.

## Rules

- Read the failing test FIRST — understand exactly what it expects
- Write the MINIMUM code to make it pass
- You may NOT modify test files
- Do not add error handling for cases the test doesn't cover
- Do not add features the test doesn't require
- Do not refactor existing code
- Run the test after every change

## Process

1. Read the failing test
2. Understand what it asserts
3. Write the smallest change that makes it pass
4. Run it
5. If it passes, stop. Hand off to Refactor.
6. If it fails, adjust and try again.

## What "minimum" means

If the test expects a function to return 42, write `return 42`. Don't write a general-purpose calculator. The next Red test will force you to generalize. That's the point.
