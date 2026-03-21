---
name: Rust Error Handling
description: Idiomatic Rust error handling with thiserror, anyhow, and the Result pattern
tags: [rust, errors, thiserror, anyhow, result]
---

# Rust Error Handling

## The Two-Crate Rule

| Context | Crate | Why |
|---------|-------|-----|
| Library (consumed by other crates) | `thiserror` | Callers need to match on specific error variants |
| Binary / application | `anyhow` | Top-level code just needs to report errors |

Never use `anyhow` in library code. Never define `thiserror` enums in `main.rs`.

## Defining Library Errors with thiserror

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid header at byte {offset}: {reason}")]
    InvalidHeader { offset: usize, reason: String },

    #[error("unsupported version {0}")]
    UnsupportedVersion(u32),

    #[error("I/O error reading input")]
    Io(#[from] std::io::Error),
}
```

### Design Rules

- Each variant describes one failure mode. No catch-all `Other(String)` variants.
- Use `#[from]` to auto-convert underlying errors when the mapping is 1:1.
- Error messages must be lowercase, no trailing period (Rust convention).
- Include enough context to diagnose: offsets, keys, expected vs actual values.

## Using anyhow in Binaries

```rust
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let config = load_config()
        .context("failed to load config from ~/.config/app/config.toml")?;

    let db = connect_db(&config.database_url)
        .context("failed to connect to database")?;

    run_server(db)?;
    Ok(())
}
```

### Context Rules

Every `?` in application code should have `.context()` or `.with_context()` attached. The context string answers "what were we trying to do when this failed?"

```rust
// Bad — raw error with no context
let file = File::open(path)?;

// Good — explains the operation
let file = File::open(path)
    .with_context(|| format!("failed to open config file {}", path.display()))?;
```

## The ? Operator

`?` does three things: unwrap Ok, convert error type (via `From`), return early on Err.

```rust
fn read_config(path: &Path) -> Result<Config, ParseError> {
    let content = std::fs::read_to_string(path)?;  // io::Error -> ParseError via #[from]
    let config = parse_toml(&content)?;              // ParseError passes through
    Ok(config)
}
```

## When to Use unwrap / expect

| Method | Use When |
|--------|----------|
| `unwrap()` | Never in production code without a comment proving safety |
| `expect("reason")` | Static guarantees make failure impossible (e.g., compiled regex) |
| `?` | Default for all fallible operations |

```rust
// Acceptable — regex is a compile-time constant
let re = Regex::new(r"^\d{4}-\d{2}-\d{2}$").expect("date regex is valid");

// Never acceptable — runtime data can be anything
let value = map.get("key").unwrap();

// Correct
let value = map.get("key").ok_or_else(|| anyhow!("missing required key 'key'"))?;
```

## Custom Result Type Aliases

Define a crate-level result alias to reduce boilerplate:

```rust
pub type Result<T> = std::result::Result<T, crate::Error>;
```

Then functions return `Result<Config>` instead of `Result<Config, crate::Error>`.

## Error Conversion Patterns

### Manual From Implementation

When `#[from]` is not appropriate (need to add context during conversion):

```rust
impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::InvalidFormat {
            line: err.line(),
            column: err.column(),
            message: err.to_string(),
        }
    }
}
```

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|-------------|---------|-----|
| `Box<dyn Error>` in libraries | Callers cannot match variants | Use thiserror enum |
| `.unwrap()` on user input | Panics in production | Use `?` with context |
| `Other(String)` catch-all variant | Callers cannot handle specific errors | Add explicit variants |
| Ignoring errors with `let _ =` | Hides failures | Log or propagate |
| Error messages starting with capital | Breaks error chain formatting | Use lowercase |

## Checklist

- [ ] Library crates use `thiserror`, binaries use `anyhow`
- [ ] Every `?` in application code has `.context()` attached
- [ ] No `unwrap()` without a safety comment
- [ ] Error variants are specific, not catch-all
- [ ] Error messages are lowercase, no trailing period
- [ ] `#[derive(Debug)]` on all error types
