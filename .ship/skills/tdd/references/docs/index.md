---
title: TDD — Test-Driven Development Skill
description: Write failing tests first, then the minimum code to pass. Framework-aware with auto-detection.
---

# Test-Driven Development

TDD is a development discipline: write a failing test, make it pass with minimal code, refactor, repeat. Every behavior starts as a test.

## Why TDD

- **Catches regressions immediately.** The test exists before the code, so breakage surfaces the moment it happens.
- **Forces small steps.** Each test drives one piece of behavior. You build incrementally instead of designing the whole thing up front.
- **Documents intent.** Tests describe what the code should do, not how it's implemented. They survive refactors.
- **Enables confident refactoring.** Green tests mean the behavior hasn't changed.

## The Red-Green-Refactor Cycle

1. **Red** — Write a test for the next behavior. Run it. It must fail. If it passes, either the test is wrong or the behavior already exists.
2. **Green** — Write the minimum production code to make the test pass. Not the whole feature — just enough for this one test.
3. **Refactor** — Clean up duplication, improve naming, simplify. Tests must stay green.

## Variables

| Variable | Type | Scope | Default | Description |
|----------|------|-------|---------|-------------|
| `test_runner` | enum | project | `auto` | Test framework. `auto` detects from project files. |
| `commit_at_green` | bool | global | `true` | Auto-commit after each passing test. |

```bash
# Lock to a specific test runner for this project
ship vars set tdd test_runner vitest

# Disable auto-commit (manual commit workflow)
ship vars set tdd commit_at_green false
```

## Framework Detection

When `test_runner` is `auto`, the skill checks for:

| Priority | File | Runner |
|----------|------|--------|
| 1 | `Cargo.toml` | `cargo test` |
| 2 | `package.json` + vitest | `npx vitest run` |
| 3 | `package.json` + jest | `npx jest` |
| 4 | `pyproject.toml` + pytest | `pytest` |
| 5 | `go.mod` | `go test ./...` |
| 6 | `Makefile` + test target | `make test` |

## Running Specific Tests

| Framework | Single test | Single file | Verbose |
|-----------|-----------|-------------|---------|
| cargo | `cargo test test_name` | `cargo test --test file` | `-- --nocapture` |
| vitest | `npx vitest run -t "name"` | `npx vitest run file` | `--reporter=verbose` |
| jest | `npx jest -t "name"` | `npx jest file` | `--verbose` |
| pytest | `pytest -k "name"` | `pytest file.py` | `-v` |
| go | `go test -run Name ./...` | `go test ./pkg/...` | `-v` |

## When NOT to Use TDD

- **Config files** — No behavior to test
- **Generated code** — Test the generator, not the output
- **Throwaway prototypes** — Exploring, not building
- **Pure UI layout** — Visual verification > unit tests (use browse skill instead)

## Common Mistakes

**Writing too many tests before implementing.** TDD is one test at a time. Write a test, make it pass, then write the next test. Batching tests defeats the purpose.

**Testing implementation details.** Test behavior (what the code does), not structure (how it does it). If refactoring breaks your tests, they're too coupled to implementation.

**Making the test pass with a hack.** The minimum code should be the simplest correct implementation, not a hardcoded return value. If the test can pass with a constant, write a second test that forces real logic.

**Skipping the red step.** If you don't see it fail, you don't know it tests the right thing. A test that has never failed might never fail.
