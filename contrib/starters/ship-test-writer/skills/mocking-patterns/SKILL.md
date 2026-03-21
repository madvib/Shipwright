---
name: Mocking Patterns
description: When and how to mock dependencies in tests across languages
tags: [testing, mocking, stubs, fakes, test-doubles]
---

# Mocking Patterns

## Test Double Taxonomy

| Type | Description | Use When |
|------|------------|----------|
| Stub | Returns predefined values | You need controlled inputs |
| Mock | Records calls, asserts interactions | You need to verify a side effect happened |
| Fake | Working implementation with shortcuts | You need realistic behavior without the real dependency |
| Spy | Wraps real object, records calls | You need real behavior + interaction verification |

### Decision Tree

```
External API call? --> Stub the response (or use HTTP recording)
Database? --> Fake (in-memory DB) for unit tests, real DB for integration
File system? --> Fake or t.TempDir / tmp directory
Email / SMS? --> Mock (verify it was called with correct args)
Clock / time? --> Stub (inject a fixed time)
Randomness? --> Stub (inject a fixed seed or value)
Business logic collaborator? --> Use the real object
```

## The Mock Boundary Rule

Mock at the boundary between your code and external systems. Never mock your own business logic.

```
YOUR CODE                    BOUNDARY                 EXTERNAL
  Service logic    -->    Repository interface    -->    Database
  Handler          -->    HTTP client interface   -->    Third-party API
  Scheduler        -->    Clock interface         -->    System clock
```

Mock the interface at the boundary, not the database driver or HTTP library internals.

## Dependency Injection for Testability

Structure code so dependencies are injected, not hardcoded.

### Constructor Injection

```typescript
// Testable — dependency is injected
class OrderService {
  constructor(private readonly paymentGateway: PaymentGateway) {}

  async placeOrder(order: Order): Promise<Result> {
    const charge = await this.paymentGateway.charge(order.total);
    // ...
  }
}

// In tests
const mockGateway: PaymentGateway = {
  charge: vi.fn().mockResolvedValue({ id: "ch_123", status: "success" }),
};
const service = new OrderService(mockGateway);
```

```python
# Testable — dependency is injected
class OrderService:
    def __init__(self, payment_gateway: PaymentGateway) -> None:
        self._gateway = payment_gateway

    def place_order(self, order: Order) -> Result:
        charge = self._gateway.charge(order.total)
        # ...

# In tests
gateway = Mock(spec=PaymentGateway)
gateway.charge.return_value = Charge(id="ch_123", status="success")
service = OrderService(payment_gateway=gateway)
```

```go
// Testable — interface defined by consumer
type PaymentGateway interface {
    Charge(ctx context.Context, amount int64) (*Charge, error)
}

type OrderService struct {
    gateway PaymentGateway
}

// In tests
type mockGateway struct {
    chargeResult *Charge
    chargeErr    error
}

func (m *mockGateway) Charge(_ context.Context, _ int64) (*Charge, error) {
    return m.chargeResult, m.chargeErr
}
```

## HTTP Mocking

### Request Interception (MSW, httptest, responses)

Intercept HTTP at the network level so your code uses its real HTTP client.

```typescript
// MSW (TypeScript)
import { http, HttpResponse } from "msw";
import { setupServer } from "msw/node";

const server = setupServer(
  http.get("https://api.example.com/users/:id", ({ params }) => {
    return HttpResponse.json({ id: params.id, name: "Alice" });
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());
```

```python
# responses (Python)
import responses

@responses.activate
def test_fetch_user():
    responses.add(
        responses.GET,
        "https://api.example.com/users/123",
        json={"id": "123", "name": "Alice"},
        status=200,
    )
    user = fetch_user("123")
    assert user.name == "Alice"
```

## Time Mocking

Never depend on the real clock in tests.

```typescript
// Vitest
vi.useFakeTimers();
vi.setSystemTime(new Date("2024-01-15T10:00:00Z"));
// ... test code that uses Date.now() or new Date()
vi.useRealTimers();
```

```python
# freezegun
from freezegun import freeze_time

@freeze_time("2024-01-15 10:00:00")
def test_token_expiration():
    token = create_token(ttl_hours=24)
    assert not token.is_expired()
```

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|-------------|---------|-----|
| Mocking everything | Tests pass when code is broken | Mock only external boundaries |
| Mocking the return value of the thing you are testing | Proves nothing | Test real behavior |
| Complex mock setup (20+ lines) | Test is harder to read than the code | Simplify the design or use a fake |
| Asserting mock call count | Brittle, breaks on harmless refactors | Assert outcomes, not interactions |
| Shared mocks across tests | One test's setup breaks another | Each test creates its own mocks |
| Not verifying mock expectations | Mock is configured but never asserted | Always assert or use strict mocks |

## When NOT to Mock

- Pure functions with no side effects
- Data transfer objects and value types
- In-process collaborators with fast execution
- Code you own and can instantiate cheaply

If you find yourself mocking half the codebase to test one function, the function has too many dependencies. Refactor the design.

## Checklist

- [ ] Mocks only at external boundaries (network, disk, clock, randomness)
- [ ] Dependencies injected, not hardcoded
- [ ] Each test creates its own mock state
- [ ] Mock setup is under 10 lines per test
- [ ] Assertions verify outcomes, not internal call patterns
- [ ] Fakes used instead of mocks when behavior matters
- [ ] No mocking of the unit under test
