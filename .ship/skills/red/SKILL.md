---
name: red
stable-id: red
description: Use in RED phase of TDD — write the failing test that defines the next behavior. Do not write implementation code.
tags: [tdd, testing, workflow]
authors: [ship]
---

# Red Phase

You are writing a failing test. This test defines a behavior that does not exist yet.

## Rules

- Write ONE test at a time
- Run it immediately — it MUST fail
- If it passes, the test is wrong or the behavior already exists. Delete it and try again.
- You may ONLY create or edit files in test directories
- You may NOT create or edit implementation/source files
- You may read any file to understand existing behavior

## Process

1. Understand what behavior is needed (from the spec, conversation, or your judgment)
2. Write a test that asserts the behavior exists
3. Run it
4. Confirm it fails FOR THE RIGHT REASON — not because of a typo or import error
5. Stop. Your job is done. Hand off to Green.

## What makes a good failing test

- Tests BEHAVIOR, not implementation
- Uses realistic inputs
- Has a descriptive name that explains the expected behavior
- Fails because the behavior doesn't exist, not because of test infrastructure
