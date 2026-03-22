---
name: Rails Conventions
description: Rails conventions, architecture patterns, and ActiveRecord best practices
tags: [rails, ruby, activerecord, conventions]
---

# Rails Conventions

## Application Structure

```
app/
  controllers/     — HTTP request handling only
  models/          — Business logic, validations, associations
  services/        — Complex operations spanning multiple models
  jobs/            — Background processing (ActiveJob)
  mailers/         — Email composition
  serializers/     — API response formatting
  queries/         — Complex database queries
  validators/      — Custom validation logic
```

## Controller Rules

Controllers receive requests and return responses. No business logic.

```ruby
class OrdersController < ApplicationController
  def create
    result = Orders::CreateService.call(order_params, current_user:)

    if result.success?
      redirect_to result.order, notice: "Order placed"
    else
      @order = result.order
      render :new, status: :unprocessable_entity
    end
  end

  private

  def order_params
    params.require(:order).permit(:product_id, :quantity, :shipping_address)
  end
end
```

### Controller Checklist

- [ ] Strong parameters for every action accepting input
- [ ] No direct model queries beyond simple `find` / `find_by`
- [ ] No business logic (conditions, calculations, side effects)
- [ ] Responds with appropriate HTTP status codes
- [ ] Uses `before_action` for authentication/authorization

## Model Patterns

### Validation Order

```ruby
class User < ApplicationRecord
  # 1. Constants
  ROLES = %w[admin member guest].freeze

  # 2. Associations
  has_many :posts, dependent: :destroy
  belongs_to :organization

  # 3. Validations
  validates :email, presence: true, uniqueness: { case_sensitive: false }
  validates :role, inclusion: { in: ROLES }

  # 4. Scopes
  scope :active, -> { where(deactivated_at: nil) }
  scope :admins, -> { where(role: "admin") }

  # 5. Callbacks (use sparingly)
  before_validation :normalize_email

  # 6. Instance methods
  def deactivate!
    update!(deactivated_at: Time.current)
  end

  private

  def normalize_email
    self.email = email&.downcase&.strip
  end
end
```

### When to Use Callbacks vs Services

| Use Callback | Use Service |
|-------------|-------------|
| Normalizing data before save | Multi-model operations |
| Setting default values | External API calls |
| Simple derived fields | Sending notifications |
| Maintaining data consistency within one model | Complex conditional logic |

## Service Objects

For operations that span multiple models or have complex orchestration.

```ruby
module Orders
  class CreateService
    def self.call(params, current_user:)
      new(params, current_user:).call
    end

    def initialize(params, current_user:)
      @params = params
      @current_user = current_user
    end

    def call
      order = Order.new(@params.merge(user: @current_user))

      ActiveRecord::Base.transaction do
        order.save!
        InventoryService.reserve(order)
        OrderMailer.confirmation(order).deliver_later
      end

      Result.new(success: true, order:)
    rescue ActiveRecord::RecordInvalid
      Result.new(success: false, order:)
    end
  end
end
```

## Database and Migrations

### Migration Rules

- Migrations must be reversible
- Add indexes for foreign keys and frequently queried columns
- Use `null: false` and database-level defaults where appropriate
- Never modify data in schema migrations; use separate data migrations

```ruby
class AddOrganizationToUsers < ActiveRecord::Migration[7.1]
  def change
    add_reference :users, :organization, null: false, foreign_key: true, index: true
  end
end
```

### Query Optimization

| Problem | Solution |
|---------|----------|
| N+1 queries | `includes`, `preload`, or `eager_load` |
| Loading unused columns | `select` to pick only needed columns |
| Counting with `length` | Use `count` or `size` (SQL COUNT vs cached) |
| Full table scans | Add database indexes |
| Complex queries in views | Use query objects or scopes |

```ruby
# Bad — N+1
users.each { |u| u.posts.count }

# Good — eager load
users = User.includes(:posts)
users.each { |u| u.posts.size }
```

## Background Jobs

```ruby
class SendInvoiceJob < ApplicationJob
  queue_as :default
  retry_on Net::OpenTimeout, wait: :polynomially_longer, attempts: 5

  def perform(order_id)
    order = Order.find(order_id)
    InvoiceService.generate_and_send(order)
  end
end
```

### Job Rules

- Pass IDs, not objects (objects may change between enqueue and execution)
- Set explicit retry policies
- Make jobs idempotent (safe to run twice)
- Use appropriate queues (`default`, `mailers`, `critical`)

## Checklist

- [ ] Controllers have no business logic
- [ ] Strong parameters on every action accepting input
- [ ] Models follow the standard section order
- [ ] Service objects for multi-model operations
- [ ] Migrations are reversible
- [ ] N+1 queries eliminated with eager loading
- [ ] Background jobs are idempotent
- [ ] Foreign keys have database indexes
