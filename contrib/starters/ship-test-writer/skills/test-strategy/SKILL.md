---
name: Test Strategy
description: Deciding what to test, test categorization, and coverage strategy
tags: [testing, strategy, coverage, tdd]
---

# Test Strategy

## What to Test

Not everything needs a test. Focus effort where it matters most.

### Test Priority Matrix

| Code Type | Priority | Test Type | Examples |
|-----------|----------|-----------|---------|
| Business logic | Highest | Unit tests | Calculations, validations, state machines |
| API endpoints | High | Integration tests | Request/response contracts, auth flows |
| Data access | High | Integration tests | Queries, migrations, constraints |
| Error handling | High | Unit tests | Exception paths, fallback behavior |
| UI interactions | Medium | Component tests | Form submission, navigation, conditional rendering |
| Configuration | Low | Smoke tests | App boots, config loads without error |
| Generated code | Skip | None | Protobuf stubs, ORM migrations, CSS modules |

### Decision Tree

```
Is it generated code? --> Skip
Is it a type definition only? --> Skip
Does it contain logic (conditionals, loops, calculations)? --> Test it
Does it interact with external systems? --> Integration test
Is it a pure function? --> Unit test (high priority)
Is it glue code (just wiring things together)? --> Test through integration
```

## Test Categories

### Unit Tests

Test a single function or class in isolation. Fast, deterministic, no I/O.

```
Scope: one function or method
Dependencies: none (or stubs/fakes for collaborators)
Speed: < 10ms per test
Determinism: must pass 100% of the time
```

### Integration Tests

Test multiple components working together. May use real databases or file systems.

```
Scope: function + its real dependencies
Dependencies: real database, real file system, stubbed external APIs
Speed: < 1s per test
Determinism: deterministic if external state is controlled
```

### End-to-End Tests

Test the full system from user perspective.

```
Scope: entire application stack
Dependencies: all real (or containerized)
Speed: seconds per test
Determinism: may be flaky — minimize these
```

## Coverage Strategy

### The 80/20 Rule

Aim for 80% line coverage through meaningful tests. Do not chase 100% — the last 20% often requires testing trivial code or implementation details.

### What to Prioritize

| High Value | Low Value |
|-----------|----------|
| Complex conditionals | Simple getters/setters |
| Error handling paths | Logging statements |
| Business rules | Framework boilerplate |
| State transitions | Type conversions |
| Boundary values | Happy-path-only pass-through |

### Coverage Gaps to Watch

These patterns indicate missing tests:

```
Untested catch/except blocks --> Add error injection tests
Untested else branches --> Add negative case tests
Untested default cases --> Add edge case tests
Untested error returns --> Add failure path tests
Complex boolean expressions --> Test each combination
```

## Test Naming

Test names are specifications. Someone reading only the test names should understand the module's behavior.

### Naming Pattern

`<unit>_<behavior>_<condition>`

```
calculate_total_returns_zero_for_empty_cart
parse_date_throws_on_invalid_format
login_returns_token_when_credentials_valid
login_locks_account_after_five_failed_attempts
```

### Bad Names

```
test_1                    # meaningless
test_calculate            # which behavior?
test_error                # which error?
it_works                  # what works?
```

## Writing Tests for Existing Code

When adding tests to untested code:

1. Read the source and identify all code paths (conditionals, loops, early returns)
2. List the behaviors as test names before writing any test body
3. Write the happy path test first
4. Write error path tests (invalid input, missing data, network failure)
5. Write edge case tests (empty, null, maximum values, concurrent access)
6. Run with coverage to find paths you missed

## Test Maintenance

### When to Update Tests

| Change | Test Action |
|--------|------------|
| Bug fix | Add regression test first, then fix |
| New feature | Add tests for new behavior |
| Refactor (same behavior) | Tests should pass without changes |
| API contract change | Update integration tests |
| Deleted feature | Delete corresponding tests |

### Signs of Bad Tests

- Test fails intermittently (flaky) -- fix the non-determinism
- Test name does not match assertion -- rename or split
- Test has 10+ assertions -- split into focused tests
- Test sets up state it does not use -- remove dead setup
- Test mocks everything -- you are testing the mocks, not the code

## Checklist

- [ ] Business logic has unit tests
- [ ] API endpoints have integration tests
- [ ] Error paths are tested (not just happy path)
- [ ] Edge cases covered (empty, null, boundary values)
- [ ] Test names read as behavior specifications
- [ ] No test depends on another test's state
- [ ] Mocks used only for external boundaries
- [ ] Tests run and pass before submission
