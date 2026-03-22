---
name: API Docs
description: Writing clear, complete API documentation with request/response schemas and examples
tags: [documentation, api, rest, openapi]
---

# API Documentation

## Endpoint Documentation Structure

Every API endpoint needs these sections, in this order:

1. Title and one-line description
2. HTTP method and path
3. Authentication requirements
4. Request parameters (path, query, body)
5. Response schema (success and error)
6. Code examples
7. Error reference

## Template

```markdown
## Create Order

Creates a new order for the authenticated user.

### Request

`POST /api/orders`

**Authentication:** Bearer token required

**Headers:**
| Header | Value | Required |
|--------|-------|----------|
| Authorization | Bearer {token} | Yes |
| Content-Type | application/json | Yes |

**Body:**
| Field | Type | Required | Description |
|-------|------|----------|-------------|
| product_id | string | Yes | Product UUID |
| quantity | integer | Yes | Must be >= 1 |
| shipping_address | object | Yes | See Address schema |
| notes | string | No | Order notes (max 500 chars) |

**Body Example:**
```json
{
  "product_id": "prod_abc123",
  "quantity": 2,
  "shipping_address": {
    "line1": "123 Main St",
    "city": "Portland",
    "state": "OR",
    "zip": "97201"
  }
}
```

### Response

**201 Created**
```json
{
  "order": {
    "id": "ord_xyz789",
    "status": "pending",
    "total": 5998,
    "created_at": "2024-01-15T10:30:00Z"
  }
}
```

**Error Responses:**

| Status | Code | Description |
|--------|------|-------------|
| 400 | invalid_quantity | Quantity must be a positive integer |
| 401 | unauthorized | Missing or invalid authentication token |
| 404 | product_not_found | Product ID does not exist |
| 422 | insufficient_stock | Requested quantity exceeds available stock |
```

## Schema Documentation

### Type Reference

| JSON Type | Description | Example |
|-----------|------------|---------|
| string | UTF-8 text | `"hello"` |
| integer | Whole number (no decimals) | `42` |
| number | Decimal number | `19.99` |
| boolean | True or false | `true` |
| array | Ordered list | `[1, 2, 3]` |
| object | Key-value pairs | `{"key": "value"}` |
| null | Absent value | `null` |

### Documenting Enums

List all valid values. Describe what each value means.

```markdown
**status** (string): Order status
| Value | Description |
|-------|-------------|
| pending | Order created, awaiting payment |
| paid | Payment confirmed, awaiting fulfillment |
| shipped | Order dispatched to carrier |
| delivered | Carrier confirmed delivery |
| cancelled | Order cancelled by user or system |
```

## Pagination

Document pagination parameters consistently across all list endpoints.

```markdown
### Query Parameters

| Param | Type | Default | Description |
|-------|------|---------|-------------|
| page | integer | 1 | Page number (1-indexed) |
| per_page | integer | 20 | Items per page (max 100) |
| sort | string | created_at | Sort field |
| order | string | desc | Sort order: asc or desc |

### Response Envelope

```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total": 142,
    "total_pages": 8
  }
}
```
```

## Error Format

Document a consistent error format used across all endpoints.

```markdown
### Error Response Format

All errors return a JSON object with `error` and `message` fields.

```json
{
  "error": "validation_failed",
  "message": "Quantity must be a positive integer",
  "details": [
    { "field": "quantity", "code": "invalid", "message": "must be >= 1" }
  ]
}
```

The `details` array is present only for validation errors (400 status).
```

## Writing Quality Checklist

- [ ] Every endpoint has method, path, auth, request, response, and errors documented
- [ ] Request body shows all fields with type, required/optional, and description
- [ ] Response shows the full JSON shape, not just one field
- [ ] Error responses list all possible status codes and their meaning
- [ ] Code examples are copy-pasteable (valid JSON, complete curl commands)
- [ ] Pagination documented for all list endpoints
- [ ] Authentication requirements stated explicitly (not assumed)
- [ ] Field constraints documented (min/max, regex patterns, allowed values)
