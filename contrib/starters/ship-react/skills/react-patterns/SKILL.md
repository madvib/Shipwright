---
name: React Patterns
description: Modern React component patterns, hooks, and state management
tags: [react, hooks, components, patterns]
---

# React Patterns

## Component Structure

Every component file follows this order:

1. Imports (React, third-party, local)
2. Type definitions (Props, internal types)
3. Component function (named export)
4. Helper functions (below the component, not inline)

```tsx
import { useState, useCallback } from "react";
import type { User } from "@/types";

interface UserCardProps {
  user: User;
  onSelect: (id: string) => void;
}

export function UserCard({ user, onSelect }: UserCardProps) {
  const handleClick = useCallback(() => {
    onSelect(user.id);
  }, [user.id, onSelect]);

  return (
    <button onClick={handleClick}>
      {user.name}
    </button>
  );
}
```

## Hook Patterns

### Custom Hook Naming

All custom hooks start with `use`. Return objects for hooks with multiple values, tuples for simple state+setter patterns.

```tsx
// Object return for complex hooks
function useAuth() {
  return { user, login, logout, isLoading };
}

// Tuple return for simple state hooks
function useToggle(initial = false): [boolean, () => void] {
  const [value, setValue] = useState(initial);
  const toggle = useCallback(() => setValue((v) => !v), []);
  return [value, toggle];
}
```

### Data Fetching Hooks

Wrap all async operations in hooks that return `{ data, error, isLoading }`. Never call fetch inside a component body.

```tsx
function useUser(id: string) {
  const [state, setState] = useState<{
    data: User | null;
    error: Error | null;
    isLoading: boolean;
  }>({ data: null, error: null, isLoading: true });

  useEffect(() => {
    let cancelled = false;
    fetchUser(id)
      .then((data) => { if (!cancelled) setState({ data, error: null, isLoading: false }); })
      .catch((error) => { if (!cancelled) setState({ data: null, error, isLoading: false }); });
    return () => { cancelled = true; };
  }, [id]);

  return state;
}
```

## Composition Over Configuration

### Compound Components

Use compound components for complex UI with shared state. Avoid mega-props objects.

```tsx
// Prefer this:
<Select>
  <Select.Trigger>Choose...</Select.Trigger>
  <Select.Options>
    <Select.Option value="a">Option A</Select.Option>
  </Select.Options>
</Select>

// Over this:
<Select
  placeholder="Choose..."
  options={[{ label: "Option A", value: "a" }]}
  renderOption={(o) => <span>{o.label}</span>}
/>
```

### Render Props vs Hooks

Prefer hooks for logic reuse. Use render props only when the consumer needs to control the rendering of children that depend on the shared state.

## State Management Decision Tree

```
Need state? -->
  Used by one component? --> useState
  Shared by parent + children? --> Props / composition
  Shared by siblings? --> Lift state to parent
  Shared across distant components? --> Context
  Global, complex, or async? --> External store (zustand, jotai)
  Server state? --> TanStack Query / SWR
```

## Anti-Patterns to Avoid

| Anti-Pattern | Problem | Fix |
|-------------|---------|-----|
| `useEffect` for derived state | Extra render, stale values | Compute during render |
| Props spreading `{...props}` | Hides interface, passes junk | Destructure explicitly |
| Index as key in dynamic lists | Breaks reconciliation | Use stable unique IDs |
| Inline object/array literals in JSX | New reference every render | useMemo or extract |
| Nested ternaries in JSX | Unreadable | Extract to variable or early return |
| `any` type on props | Defeats TypeScript | Define explicit interface |

## Performance Checklist

- [ ] Memoize expensive computations with `useMemo`
- [ ] Memoize callbacks passed to children with `useCallback`
- [ ] Use `React.memo` only when profiling shows unnecessary re-renders
- [ ] Avoid creating new objects/arrays in render — extract to constants or memoize
- [ ] Use `key` prop correctly in lists (stable IDs, never index for dynamic lists)
- [ ] Lazy-load routes and heavy components with `React.lazy` + `Suspense`
