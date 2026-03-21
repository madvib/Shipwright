---
name: Component Testing
description: Testing strategies for React components using Testing Library and Vitest
tags: [react, testing, vitest, testing-library]
---

# Component Testing

## Testing Philosophy

Test behavior, not implementation. Your tests should answer: "does this component do what the user expects?" Never test internal state, hook call counts, or CSS class names.

## Test File Structure

Every test file follows this layout:

```tsx
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi } from "vitest";
import { MyComponent } from "./MyComponent";

describe("MyComponent", () => {
  it("renders the initial state", () => {
    render(<MyComponent label="Click me" />);
    expect(screen.getByRole("button", { name: "Click me" })).toBeInTheDocument();
  });

  it("calls onSubmit with form data when submitted", async () => {
    const user = userEvent.setup();
    const handleSubmit = vi.fn();
    render(<MyComponent onSubmit={handleSubmit} />);

    await user.type(screen.getByLabelText("Email"), "test@example.com");
    await user.click(screen.getByRole("button", { name: "Submit" }));

    expect(handleSubmit).toHaveBeenCalledWith({ email: "test@example.com" });
  });
});
```

## Query Priority

Use queries in this order. Accessibility-first ensures your tests match how users interact with the page.

| Priority | Query | Use When |
|----------|-------|----------|
| 1 | `getByRole` | Buttons, links, headings, form controls |
| 2 | `getByLabelText` | Form fields with labels |
| 3 | `getByPlaceholderText` | Inputs with placeholder (fallback) |
| 4 | `getByText` | Non-interactive text content |
| 5 | `getByDisplayValue` | Current value of form controls |
| 6 | `getByTestId` | Last resort — no semantic alternative |

Never start with `getByTestId`. If you need it, the component may have an accessibility problem.

## User Interaction

Always use `userEvent` over `fireEvent`. It simulates real browser behavior (focus, keydown, keyup, input, click).

```tsx
const user = userEvent.setup();

// Typing
await user.type(input, "hello");

// Clicking
await user.click(button);

// Selecting options
await user.selectOptions(select, "value");

// Clearing and typing
await user.clear(input);
await user.type(input, "new value");

// Keyboard
await user.keyboard("{Enter}");
```

## Async Testing

For components with async behavior (data fetching, timers, transitions):

```tsx
it("shows loading then data", async () => {
  render(<UserProfile id="123" />);

  // Loading state
  expect(screen.getByText("Loading...")).toBeInTheDocument();

  // Wait for data
  expect(await screen.findByText("John Doe")).toBeInTheDocument();
  expect(screen.queryByText("Loading...")).not.toBeInTheDocument();
});
```

Use `findBy*` (returns promise) instead of `waitFor` + `getBy*` when waiting for an element to appear.

## Mocking

### API Mocking with MSW

Prefer MSW (Mock Service Worker) over mocking fetch/axios directly. It intercepts at the network level and works with any fetch implementation.

```tsx
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

const server = setupServer(
  http.get("/api/user/:id", ({ params }) => {
    return HttpResponse.json({ id: params.id, name: "Test User" });
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

### Module Mocking

Mock only what you must. Over-mocking makes tests pass when the code is broken.

```tsx
vi.mock("@/lib/analytics", () => ({
  track: vi.fn(),
}));
```

## Common Test Cases Checklist

- [ ] Renders correctly with required props
- [ ] Renders correctly with optional props omitted
- [ ] User interactions trigger expected callbacks
- [ ] Loading states display correctly
- [ ] Error states display with actionable messages
- [ ] Empty states display when data is empty
- [ ] Conditional rendering shows/hides elements correctly
- [ ] Form validation rejects invalid input
- [ ] Accessibility: focusable, labeled, keyboard-navigable

## Anti-Patterns

| Anti-Pattern | Why It Fails | Fix |
|-------------|-------------|-----|
| Testing implementation details | Breaks on refactor, passes on bugs | Test user-visible behavior |
| Snapshot tests for logic | Pass blindly with `--update` | Assert specific content |
| `container.querySelector` | Bypasses accessibility | Use `getByRole` queries |
| Testing library internals | Not your code | Mock boundaries only |
| No error case tests | Misses real failures | Test error and empty states |
