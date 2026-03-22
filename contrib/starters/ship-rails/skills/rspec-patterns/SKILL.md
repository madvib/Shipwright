---
name: RSpec Patterns
description: RSpec testing patterns for Rails applications with FactoryBot and system specs
tags: [rails, rspec, testing, factorybot]
---

# RSpec Patterns

## Test Organization

```
spec/
  models/          — Unit tests for models
  requests/        — Controller/API integration tests
  services/        — Service object tests
  jobs/            — Background job tests
  system/          — Full browser tests (Capybara)
  support/         — Shared helpers and config
  factories/       — FactoryBot definitions
```

Use request specs over controller specs. Controller specs are deprecated in modern Rails.

## Spec Structure

Every spec follows the Given-When-Then pattern using `describe`, `context`, and `it`.

```ruby
RSpec.describe Orders::CreateService do
  describe ".call" do
    context "with valid params" do
      it "creates the order and returns success" do
        user = create(:user)
        product = create(:product, stock: 10)

        result = described_class.call(
          { product_id: product.id, quantity: 2 },
          current_user: user
        )

        expect(result).to be_success
        expect(result.order).to be_persisted
        expect(result.order.quantity).to eq(2)
      end

      it "sends a confirmation email" do
        user = create(:user)
        product = create(:product, stock: 10)

        expect {
          described_class.call(
            { product_id: product.id, quantity: 1 },
            current_user: user
          )
        }.to have_enqueued_mail(OrderMailer, :confirmation)
      end
    end

    context "with insufficient stock" do
      it "returns failure without creating an order" do
        user = create(:user)
        product = create(:product, stock: 0)

        result = described_class.call(
          { product_id: product.id, quantity: 1 },
          current_user: user
        )

        expect(result).not_to be_success
        expect(Order.count).to eq(0)
      end
    end
  end
end
```

## FactoryBot

### Factory Definitions

```ruby
FactoryBot.define do
  factory :user do
    sequence(:email) { |n| "user#{n}@example.com" }
    name { "Test User" }
    role { "member" }

    trait :admin do
      role { "admin" }
    end

    trait :deactivated do
      deactivated_at { 1.day.ago }
    end
  end
end
```

### Factory Usage Rules

| Method | Use When |
|--------|----------|
| `build(:user)` | Need an instance but do not need it saved to DB |
| `create(:user)` | Need a persisted record (associations, queries) |
| `build_stubbed(:user)` | Need an instance with an ID but no DB at all |
| `attributes_for(:user)` | Need a hash of attributes (request spec params) |

Prefer `build` over `create` when possible. Database operations slow tests.

### Traits Over Nesting

```ruby
# Good — composable traits
create(:user, :admin, :deactivated)

# Bad — deeply nested factories
create(:deactivated_admin_user)
```

## Request Specs

Test the full HTTP request cycle. These replace controller specs.

```ruby
RSpec.describe "POST /api/orders", type: :request do
  let(:user) { create(:user) }
  let(:product) { create(:product, price: 29_99) }
  let(:headers) { auth_headers_for(user) }

  context "with valid params" do
    it "returns 201 and the order JSON" do
      post "/api/orders",
        params: { order: { product_id: product.id, quantity: 1 } },
        headers: headers,
        as: :json

      expect(response).to have_http_status(:created)
      expect(json_body["order"]["product_id"]).to eq(product.id)
    end
  end

  context "without authentication" do
    it "returns 401" do
      post "/api/orders",
        params: { order: { product_id: product.id } },
        as: :json

      expect(response).to have_http_status(:unauthorized)
    end
  end
end
```

## Shared Examples

Extract common behavior into shared examples when 3+ specs share the same assertions.

```ruby
RSpec.shared_examples "a soft-deletable model" do
  describe "#soft_delete!" do
    it "sets deleted_at without destroying the record" do
      record = create(described_class.model_name.singular)
      record.soft_delete!

      expect(record.deleted_at).to be_present
      expect(record).to be_persisted
    end
  end
end

RSpec.describe User do
  it_behaves_like "a soft-deletable model"
end
```

## Matchers Cheat Sheet

```ruby
# Equality
expect(result).to eq(expected)
expect(result).to be(exact_same_object)

# Truthiness
expect(value).to be_truthy
expect(value).to be_nil

# Collections
expect(array).to include(item)
expect(array).to contain_exactly(a, b, c)  # order independent
expect(array).to match_array([c, b, a])

# Changes
expect { action }.to change(User, :count).by(1)
expect { action }.to change { user.reload.name }.from("old").to("new")

# Errors
expect { action }.to raise_error(ActiveRecord::RecordNotFound)

# Jobs / Mailers
expect { action }.to have_enqueued_job(SomeJob).with(args)
expect { action }.to have_enqueued_mail(SomeMailer, :method)
```

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|-------------|---------|-----|
| `let!` everywhere | Creates records even when not needed | Use `let` (lazy) by default |
| Testing private methods | Breaks on refactor | Test through public interface |
| `subject` with complex setup | Obscures what is being tested | Explicit `described_class.call(...)` |
| `before` blocks doing too much | Shared state confusion | Move setup into each test |
| Mocking the object under test | Proves nothing | Mock collaborators only |

## Checklist

- [ ] Request specs for all API endpoints
- [ ] Model specs for validations, scopes, and instance methods
- [ ] Service specs for happy path and error paths
- [ ] Factories use traits over nested factories
- [ ] `build` preferred over `create` where possible
- [ ] No test interdependencies (each test sets up its own state)
- [ ] Shared examples extracted when 3+ specs share behavior
- [ ] Job specs verify idempotency
