---
name: Go Idioms
description: Idiomatic Go patterns for error handling, interfaces, concurrency, and package design
tags: [go, idioms, patterns, concurrency]
---

# Go Idioms

## Error Handling

Errors are values. Handle them explicitly at every call site.

```go
result, err := doSomething()
if err != nil {
    return fmt.Errorf("doing something: %w", err)
}
```

### Error Wrapping

Use `%w` to wrap errors for `errors.Is` / `errors.As` compatibility. Use `%v` only when you intentionally want to break the error chain.

```go
// Wraps — callers can unwrap and match
return fmt.Errorf("loading config from %s: %w", path, err)

// Does NOT wrap — creates new error, hides cause
return fmt.Errorf("loading config: %v", err)
```

### Sentinel Errors vs Error Types

| Pattern | When to Use |
|---------|------------|
| `var ErrNotFound = errors.New("not found")` | Simple condition checks |
| Custom error type | Need to carry structured data (field values, codes) |

```go
// Sentinel
var ErrNotFound = errors.New("resource not found")

// Checking
if errors.Is(err, ErrNotFound) { ... }

// Custom type
type ValidationError struct {
    Field   string
    Message string
}

func (e *ValidationError) Error() string {
    return fmt.Sprintf("validation failed on %s: %s", e.Field, e.Message)
}

// Checking
var ve *ValidationError
if errors.As(err, &ve) {
    log.Printf("field %s: %s", ve.Field, ve.Message)
}
```

## Interfaces

Interfaces are defined where they are used, not where they are implemented.

```go
// In the consumer package (handler/)
type UserStore interface {
    GetUser(ctx context.Context, id string) (*User, error)
}

// handler depends on this interface, not on the concrete store
func NewHandler(store UserStore) *Handler { ... }
```

### Interface Size

| Methods | Name Pattern | Example |
|---------|-------------|---------|
| 1 | `-er` suffix | `Reader`, `Writer`, `Closer` |
| 2-3 | Descriptive compound | `ReadWriter`, `UserStore` |
| 4+ | Probably too large | Split into smaller interfaces |

## Concurrency Patterns

### Goroutine Lifecycle

Never fire-and-forget goroutines. Always have a mechanism to wait or cancel.

```go
func process(ctx context.Context, items []Item) error {
    g, ctx := errgroup.WithContext(ctx)

    for _, item := range items {
        item := item // capture loop variable (Go < 1.22)
        g.Go(func() error {
            return processItem(ctx, item)
        })
    }

    return g.Wait()
}
```

### Channel Patterns

```go
// Fan-out: multiple goroutines reading from one channel
jobs := make(chan Job)
for i := 0; i < numWorkers; i++ {
    go worker(jobs)
}

// Fan-in: multiple producers writing to one channel
results := make(chan Result)
// ... goroutines write to results
// Consumer reads from results
```

### Mutex vs Channel

| Use Mutex | Use Channel |
|-----------|-------------|
| Protecting shared state (map, counter) | Passing ownership of data |
| Short critical sections | Coordinating goroutine lifecycle |
| Simple read/write guards | Pipeline stages |

## Package Design

### Naming

```
user/       — good (singular noun)
users/      — avoid (plural)
util/       — never (meaningless)
helpers/    — never (meaningless)
common/     — avoid (put things where they belong)
```

### Package Boundaries

A package should have a clear, single responsibility. If you describe it with "and", split it.

```
// Good
auth/       — authentication
store/      — data persistence
handler/    — HTTP handlers

// Bad
auth/       — authentication AND authorization AND session management
```

## Struct Design

### Constructor Functions

Use `New` prefix. Return the concrete type, not an interface.

```go
func NewServer(addr string, opts ...Option) *Server {
    s := &Server{addr: addr}
    for _, opt := range opts {
        opt(s)
    }
    return s
}
```

### Functional Options

Use for optional configuration. Avoids config struct explosion.

```go
type Option func(*Server)

func WithTimeout(d time.Duration) Option {
    return func(s *Server) { s.timeout = d }
}

func WithLogger(l *slog.Logger) Option {
    return func(s *Server) { s.logger = l }
}
```

## Checklist

- [ ] Every error checked and either handled or wrapped with context
- [ ] Interfaces defined at consumption site, not implementation site
- [ ] No goroutine leaks (all goroutines have exit paths)
- [ ] Package names are short, singular, lowercase
- [ ] Exported identifiers have doc comments
- [ ] No `init()` functions without justification
- [ ] `context.Context` passed as first parameter for I/O functions
