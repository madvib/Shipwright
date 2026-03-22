---
name: Python Typing
description: Modern Python type annotation patterns and mypy/pyright compliance
tags: [python, typing, mypy, pyright, type-safety]
---

# Python Typing

## Annotation Rules

Every function signature must be fully annotated. No exceptions for "simple" functions.

```python
# Correct
def get_user(user_id: int) -> User | None:
    ...

# Wrong — missing return type
def get_user(user_id: int):
    ...

# Wrong — missing parameter type
def get_user(user_id) -> User | None:
    ...
```

## Common Type Patterns

### Union Types (Python 3.10+)

Use the pipe operator for unions. For older Python, use `Union` from typing.

```python
# Python 3.10+
def parse(value: str | int) -> Result:
    ...

# Python 3.9 and below
from typing import Union
def parse(value: Union[str, int]) -> Result:
    ...
```

### Optional vs Union with None

`Optional[X]` is equivalent to `X | None`. Prefer the explicit union form for clarity.

```python
# Preferred
def find(name: str) -> User | None:
    ...

# Acceptable but less clear
from typing import Optional
def find(name: str) -> Optional[User]:
    ...
```

### Collections

Use built-in generics (Python 3.9+) over typing module versions.

```python
# Python 3.9+ — use built-in
def process(items: list[str]) -> dict[str, int]:
    ...

# Python 3.8 — use typing
from typing import List, Dict
def process(items: List[str]) -> Dict[str, int]:
    ...
```

### Callable Types

```python
from collections.abc import Callable

# Function that takes a string and returns bool
Predicate = Callable[[str], bool]

# Function with no args returning None
Callback = Callable[[], None]

# Function with arbitrary args
Handler = Callable[..., None]
```

## TypedDict for Structured Dicts

When you must use dicts (API responses, JSON), use TypedDict instead of `dict[str, Any]`.

```python
from typing import TypedDict

class UserResponse(TypedDict):
    id: int
    name: str
    email: str
    is_active: bool

def fetch_user(uid: int) -> UserResponse:
    ...
```

## Protocol Classes (Structural Typing)

Use Protocol for interfaces instead of ABC when you want structural (duck) typing.

```python
from typing import Protocol

class Renderable(Protocol):
    def render(self) -> str: ...

# Any class with a render() -> str method satisfies Renderable
# No inheritance required
def display(item: Renderable) -> None:
    print(item.render())
```

## Generic Types

```python
from typing import TypeVar, Generic

T = TypeVar("T")

class Stack(Generic[T]):
    def __init__(self) -> None:
        self._items: list[T] = []

    def push(self, item: T) -> None:
        self._items.append(item)

    def pop(self) -> T:
        return self._items.pop()
```

## Type Narrowing

Use `isinstance`, `is None` checks, or `TypeGuard` to narrow types.

```python
from typing import TypeGuard

def is_string_list(val: list[object]) -> TypeGuard[list[str]]:
    return all(isinstance(x, str) for x in val)

def process(data: str | list[str]) -> str:
    if isinstance(data, str):
        return data  # type checker knows this is str
    return ", ".join(data)  # type checker knows this is list[str]
```

## Checklist

- [ ] All function signatures fully annotated (params + return)
- [ ] No `Any` types unless interfacing with untyped third-party code
- [ ] Prefer `X | None` over `Optional[X]`
- [ ] Use built-in generics (`list`, `dict`, `set`, `tuple`) on Python 3.9+
- [ ] Structured data uses dataclass, Pydantic model, or TypedDict
- [ ] Run type checker (`mypy --strict` or `pyright`) in CI
- [ ] TypeVar names match the constraint: `T` for unconstrained, `UserT` for bounded
