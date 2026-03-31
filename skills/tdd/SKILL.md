---
name: tdd
stable-id: tdd
description: Use when implementing any feature, fix, or behavior change. Enforces the TDD protocol — write the failing test first, then the minimal code to pass it. Includes interactive spec-building workflow for collaborative test design.
tags: [testing, workflow, engineering, tdd, spec]
authors: [ship]
artifacts: [markdown]
---

{% if phase == "red" %}
# Red Phase

You are writing a failing test. This test defines a behavior that does not exist yet.

**Rules:**
- Write ONE test at a time
- Run it — it MUST fail
- If it passes, the test is wrong or the behavior already exists. Delete it and try again.
- Edit test files only. Do NOT touch implementation files.

{% if test_style == "integration" %}
Write an **integration test** — crosses real boundaries (DB, filesystem, network). No mocks for the system under test. Fail because the behavior doesn't exist, not because of missing infrastructure.
{% elif test_style == "e2e" %}
Write an **end-to-end test** — full user flow through real endpoints. HTTP, browser, or CLI. No internal shortcuts.
{% else %}
Write a **unit test** — isolated, no I/O, fast. Mock external dependencies. One function per test.
{% endif %}

**Process:** Understand the behavior → write the test → run it → confirm it fails for the right reason → stop.

Check `.ship-session/spec-from-tests.md` for the planned test list if it exists.

{% elif phase == "green" %}
# Green Phase

There is a failing test. Your only job is to make it pass.

**Rules:**
- Read the failing test first — understand exactly what it expects
- Write the MINIMUM code to make it pass
- Do NOT modify test files
- Do not add features, error handling, or edge cases the test doesn't require
- Do not refactor

{% if test_style == "integration" %}
Minimum still means real I/O — you can't stub the database to pass an integration test. But hardcode return values, skip validation, skip edge cases. The next Red test forces each of those.
{% elif test_style == "e2e" %}
Wire only the happy path the test exercises. Leave error handling, middleware, and edge cases for later Red tests.
{% else %}
If the test expects `return 42`, write `return 42`. The next Red test forces generalization. That's the point.
{% endif %}

**Process:** Read test → write minimum code → run it → if green{% if commit_at_green %}, commit{% endif %} and hand off to Refactor → if not, adjust and repeat.

{% elif phase == "refactor" %}
# Refactor Phase

All tests are green. Make the code better without changing what it does.

**Rules:**
- Run ALL tests before starting — they must pass
- Run ALL tests after every change — they must still pass
- If a test fails after your change, revert immediately
- Do NOT add behavior, new tests, or new dependencies

**What to improve:** duplication, naming, complex conditionals, oversized functions, dead code, formatting, nesting depth.

{% if test_style == "e2e" %}
E2e tests only see the public surface — restructure internals freely as long as the endpoint contract holds.
{% elif test_style == "integration" %}
Integration tests pin the public API. Renaming internals is safe. Changing the shape of what the test calls is not.
{% endif %}

**Process:** Confirm green → pick one improvement → change → confirm still green → commit → repeat or stop.

{% else %}
# TDD

Write the test first. Watch it fail. Write the minimum code to pass. Refactor.

If you didn't watch it fail, you don't know if it tests the right thing.

## The Loop

```
1. Write a failing {{ test_style }} test  →  RED
2. Confirm it fails for the right reason
3. Write minimum code to pass            →  GREEN
4. Confirm it passes
5. Refactor if needed                    →  REFACTOR
{% if commit_at_green %}6. Commit
{% endif %}7. Repeat
```

**No production code before a failing test.** No exceptions.

{% if test_style == "integration" %}
## Test Style: integration

Integration tests — cross real boundaries (DB, filesystem, network). No mocks for the system under test. Slower, but they catch wiring bugs unit tests miss.
{% elif test_style == "e2e" %}
## Test Style: e2e

End-to-end tests — full user flows through real endpoints. Nothing is stubbed. Highest confidence, slowest feedback loop.
{% else %}
## Test Style: unit

Unit tests — isolated, no I/O, fast. Mock external dependencies. One function or module per test.
{% endif %}

## Phase Rules

**Red:** ONE test at a time. Edit test files only. Confirm the right failure before moving on.

**Green:** Read the failing test first. Write the minimum — if the test expects `42`, return `42`. The next test forces generalization. Do not refactor.{% if commit_at_green %} Commit at green.{% endif %}

**Refactor:** Run all tests before and after every change. Revert immediately if anything breaks. No new behavior.

## Spec Artifact

After 5–8 tests, write `.ship-session/spec-from-tests.md`:

```markdown
## Spec (from tests)

Behaviors confirmed:
- [x] <behavior> — <test name>

Open questions:
- <decision not yet pinned>

Next tests:
- <next behavior to write>
```

This is the deliverable of the spec phase. Implementation agents use it as their brief.

## Interactive Spec-Building

For collaborative test design — proposing tests one at a time, discussing implications, checkpointing specs — see `references/docs/interactive-workflow.md`.

{% endif %}

## Running Tests

{% if test_runner == "cargo" %}
`cargo test` — run full suite. `cargo test <name>` — run one test. `cargo test -- --nocapture` — with output.
{% elif test_runner == "vitest" %}
`npx vitest run` — full suite. `npx vitest run <file>` — one file. `npx vitest run --reporter=verbose` — verbose.
{% elif test_runner == "jest" %}
`npx jest` — full suite. `npx jest <file>` — one file. `npx jest --verbose` — verbose.
{% elif test_runner == "pytest" %}
`pytest` — full suite. `pytest <file>::<name>` — one test. `pytest -v` — verbose.
{% elif test_runner == "go" %}
`go test ./...` — full suite. `go test -run <name> ./...` — one test. `go test -v ./...` — verbose.
{% elif test_runner == "make" %}
`make test` — full suite. `make test VERBOSE=1` — verbose.
{% else %}
Detect from project files:

| File | Run one test | Full suite |
|------|-------------|------------|
| `Cargo.toml` | `cargo test <name>` | `cargo test` |
| `package.json` + vitest | `npx vitest run <file>` | `npx vitest run` |
| `package.json` + jest | `npx jest <file>` | `npx jest` |
| `pyproject.toml` | `pytest <file>::<name>` | `pytest` |
| `go.mod` | `go test -run <name> ./...` | `go test ./...` |
| `Makefile` | `make test` | `make test` |
{% endif %}
