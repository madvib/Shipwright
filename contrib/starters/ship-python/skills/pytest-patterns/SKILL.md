---
name: Pytest Patterns
description: Effective pytest usage including fixtures, parametrize, mocking, and test organization
tags: [python, pytest, testing, fixtures]
---

# Pytest Patterns

## Test File Layout

Tests mirror the source tree. `src/auth/service.py` is tested by `tests/auth/test_service.py`.

```
src/
  auth/
    service.py
    models.py
tests/
  auth/
    test_service.py
    test_models.py
  conftest.py
```

## Test Function Structure

Every test follows Arrange-Act-Assert. One assertion concept per test.

```python
def test_user_login_returns_token_on_valid_credentials():
    # Arrange
    service = AuthService(user_repo=FakeUserRepo())
    service.register("alice", "password123")

    # Act
    result = service.login("alice", "password123")

    # Assert
    assert result.token is not None
    assert result.user.username == "alice"
```

## Naming Convention

Test names describe the behavior, not the method: `test_<unit>_<behavior>_<condition>`.

```python
# Good — describes behavior
def test_cart_total_includes_tax_when_region_is_us():
def test_login_raises_error_on_invalid_password():
def test_parser_returns_empty_list_for_blank_input():

# Bad — describes implementation
def test_calculate():
def test_login_method():
def test_parse_function():
```

## Fixtures

Use fixtures for shared setup. Keep them in `conftest.py` at the appropriate directory level.

```python
import pytest
from myapp.db import Database

@pytest.fixture
def db():
    """Create an in-memory test database with schema."""
    database = Database(":memory:")
    database.run_migrations()
    yield database
    database.close()

@pytest.fixture
def user_service(db):
    """Service with a fresh database."""
    return UserService(db=db)
```

### Fixture Scope

| Scope | Lifetime | Use For |
|-------|----------|---------|
| `function` (default) | Each test | Most fixtures |
| `class` | Per test class | Shared state within a class |
| `module` | Per test file | Expensive setup (DB, server) |
| `session` | Entire test run | Global setup (Docker containers) |

Use the narrowest scope that works. Wider scope means shared state, which means ordering bugs.

## Parametrize

Use `@pytest.mark.parametrize` for the same logic with different inputs.

```python
@pytest.mark.parametrize("input_val,expected", [
    ("hello", "HELLO"),
    ("", ""),
    ("Hello World", "HELLO WORLD"),
    ("123abc", "123ABC"),
])
def test_uppercase_converts_correctly(input_val: str, expected: str):
    assert uppercase(input_val) == expected
```

For error cases, parametrize with `pytest.raises`:

```python
@pytest.mark.parametrize("invalid_input", [None, 42, [], {}])
def test_uppercase_rejects_non_string(invalid_input):
    with pytest.raises(TypeError):
        uppercase(invalid_input)
```

## Mocking

Mock at the boundary, not deep in the implementation.

```python
from unittest.mock import patch, MagicMock

def test_order_service_sends_confirmation_email(user_service):
    with patch("myapp.email.send") as mock_send:
        user_service.place_order(order_id="abc")
        mock_send.assert_called_once_with(
            to="user@test.com",
            subject="Order Confirmation",
        )
```

### When to Mock

| Mock | Do Not Mock |
|------|-------------|
| External APIs, email, payment | Pure functions |
| System clock (`datetime.now`) | Data classes |
| File system (for unit tests) | Your own business logic |
| Random/UUID generation | Database in integration tests |

## Exception Testing

```python
def test_withdraw_raises_on_insufficient_funds():
    account = Account(balance=100)
    with pytest.raises(InsufficientFunds, match="balance 100.*withdraw 200"):
        account.withdraw(200)
```

Always assert the exception message or type. Bare `pytest.raises(Exception)` catches everything and proves nothing.

## Marks and Organization

```python
@pytest.mark.slow
def test_full_migration_suite():
    ...

@pytest.mark.integration
def test_api_round_trip():
    ...
```

Run subsets: `pytest -m "not slow"`, `pytest -m integration`.

## Checklist

- [ ] One assertion concept per test
- [ ] Test name describes the behavior being verified
- [ ] Fixtures use the narrowest scope that works
- [ ] Mocks only at boundaries (network, filesystem, clock)
- [ ] Exception tests assert specific type and message
- [ ] Parametrize used for multiple inputs to the same logic
- [ ] No test interdependencies (tests pass in any order)
- [ ] `conftest.py` fixtures are documented with docstrings
