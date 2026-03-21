---
name: Go Testing
description: Go testing patterns with table-driven tests, test helpers, and testify
tags: [go, testing, table-driven, testify]
---

# Go Testing

## Table-Driven Tests

The standard pattern for Go tests. One function covers many cases.

```go
func TestParsePort(t *testing.T) {
    tests := []struct {
        name    string
        input   string
        want    int
        wantErr bool
    }{
        {name: "valid port", input: "8080", want: 8080},
        {name: "min port", input: "1", want: 1},
        {name: "max port", input: "65535", want: 65535},
        {name: "zero", input: "0", wantErr: true},
        {name: "negative", input: "-1", wantErr: true},
        {name: "too large", input: "65536", wantErr: true},
        {name: "not a number", input: "abc", wantErr: true},
        {name: "empty string", input: "", wantErr: true},
    }

    for _, tt := range tests {
        t.Run(tt.name, func(t *testing.T) {
            got, err := ParsePort(tt.input)
            if tt.wantErr {
                if err == nil {
                    t.Fatalf("ParsePort(%q) = %d, want error", tt.input, got)
                }
                return
            }
            if err != nil {
                t.Fatalf("ParsePort(%q) unexpected error: %v", tt.input, err)
            }
            if got != tt.want {
                t.Errorf("ParsePort(%q) = %d, want %d", tt.input, got, tt.want)
            }
        })
    }
}
```

### Table Test Rules

- Every case has a descriptive `name` field
- Use `t.Run(tt.name, ...)` for subtests (enables `go test -run TestParsePort/valid_port`)
- Test both success and error paths
- Use `t.Fatalf` for precondition failures, `t.Errorf` for assertion failures

## Test Helpers

Mark helper functions with `t.Helper()` so failure messages report the caller's line.

```go
func assertEqualUser(t *testing.T, got, want *User) {
    t.Helper()
    if got.ID != want.ID {
        t.Errorf("user ID = %q, want %q", got.ID, want.ID)
    }
    if got.Email != want.Email {
        t.Errorf("user email = %q, want %q", got.Email, want.Email)
    }
}

func newTestDB(t *testing.T) *sql.DB {
    t.Helper()
    db, err := sql.Open("sqlite3", ":memory:")
    if err != nil {
        t.Fatalf("opening test db: %v", err)
    }
    t.Cleanup(func() { db.Close() })
    return db
}
```

## Test Fixtures

### t.TempDir

For tests that need filesystem access:

```go
func TestWriteConfig(t *testing.T) {
    dir := t.TempDir() // automatically cleaned up
    path := filepath.Join(dir, "config.toml")

    err := WriteConfig(path, &Config{Port: 8080})
    if err != nil {
        t.Fatalf("WriteConfig: %v", err)
    }

    got, err := ReadConfig(path)
    if err != nil {
        t.Fatalf("ReadConfig: %v", err)
    }
    if got.Port != 8080 {
        t.Errorf("Port = %d, want 8080", got.Port)
    }
}
```

### testdata Directory

Static fixtures go in `testdata/`. Go tooling ignores this directory during builds.

```
mypackage/
  parser.go
  parser_test.go
  testdata/
    valid_input.json
    malformed.json
    empty.json
```

```go
func TestParser(t *testing.T) {
    data, err := os.ReadFile("testdata/valid_input.json")
    if err != nil {
        t.Fatalf("reading fixture: %v", err)
    }
    // ...
}
```

## HTTP Handler Testing

Use `httptest` for testing HTTP handlers without starting a server.

```go
func TestHealthHandler(t *testing.T) {
    req := httptest.NewRequest("GET", "/health", nil)
    rec := httptest.NewRecorder()

    HealthHandler(rec, req)

    if rec.Code != http.StatusOK {
        t.Errorf("status = %d, want %d", rec.Code, http.StatusOK)
    }

    var body map[string]string
    if err := json.NewDecoder(rec.Body).Decode(&body); err != nil {
        t.Fatalf("decoding response: %v", err)
    }
    if body["status"] != "ok" {
        t.Errorf("body status = %q, want %q", body["status"], "ok")
    }
}
```

## Mocking with Interfaces

Define a minimal interface, implement a test double.

```go
type Mailer interface {
    Send(ctx context.Context, to, subject, body string) error
}

type mockMailer struct {
    calls []mailCall
}

type mailCall struct{ to, subject, body string }

func (m *mockMailer) Send(_ context.Context, to, subject, body string) error {
    m.calls = append(m.calls, mailCall{to, subject, body})
    return nil
}
```

## Build Tags for Integration Tests

Separate slow/integration tests with build tags.

```go
//go:build integration

package store_test

func TestPostgresStore(t *testing.T) {
    // requires running Postgres
}
```

Run with: `go test -tags=integration ./...`

## Test Commands

```bash
go test ./...                          # all tests
go test -v ./pkg/auth/                 # verbose, one package
go test -run TestParsePort ./...       # one test by name
go test -race ./...                    # enable race detector
go test -count=1 ./...                 # disable test caching
go test -coverprofile=cover.out ./...  # coverage report
go tool cover -html=cover.out          # view coverage in browser
```

## Checklist

- [ ] Table-driven tests for functions with multiple input cases
- [ ] All test cases have descriptive names
- [ ] Helpers marked with `t.Helper()`
- [ ] `t.Cleanup` used for resource teardown (not defer in helpers)
- [ ] No test interdependencies
- [ ] `t.Parallel()` on tests that can run concurrently
- [ ] Integration tests behind build tags
- [ ] `-race` flag used in CI
