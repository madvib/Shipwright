---
name: Cargo Workspace
description: Cargo workspace organization, dependency management, and multi-crate patterns
tags: [rust, cargo, workspace, dependencies, crates]
---

# Cargo Workspace

## Workspace Structure

A workspace is a set of crates sharing a `Cargo.lock` and output directory. Use workspaces when you have 2+ crates that evolve together.

```
Cargo.toml              # workspace root
crates/
  core/                  # library crate — pure logic, no I/O
    Cargo.toml
    src/lib.rs
  cli/                   # binary crate — thin transport over core
    Cargo.toml
    src/main.rs
  server/                # binary crate — HTTP transport over core
    Cargo.toml
    src/main.rs
```

### Root Cargo.toml

```toml
[workspace]
members = ["crates/*"]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
thiserror = "2"
```

## Dependency Management

### Workspace Dependencies

Declare shared dependencies at the workspace level, reference them from member crates.

```toml
# In crate Cargo.toml
[dependencies]
serde = { workspace = true }
tokio = { workspace = true }
```

Benefits: one place to update versions, no version drift between crates.

### Dependency Decision Tree

```
Need the dependency? -->
  Is it already in workspace.dependencies? --> Use { workspace = true }
  Does another crate already use a similar dep? --> Consolidate to one
  Is it a new dep? -->
    Does it pull in 10+ transitive deps? --> Justify in PR or find a lighter alternative
    Is it maintained (commits in last 6 months)? --> Add to workspace.dependencies
    Unmaintained? --> Fork, vendor, or find an alternative
```

### Feature Flags

Enable features explicitly. Never use `features = ["full"]` unless you actually need every feature.

```toml
# Bad — pulls in everything
tokio = { version = "1", features = ["full"] }

# Good — only what you use
tokio = { version = "1", features = ["rt-multi-thread", "macros", "net"] }
```

## Crate Boundaries

### Library vs Binary Separation

Keep business logic in library crates. Binary crates are thin wrappers.

```
core/src/lib.rs    — Config, Parser, Engine (all logic)
cli/src/main.rs    — Argument parsing, calls core functions, formats output
server/src/main.rs — HTTP routing, calls core functions, serializes responses
```

This makes the logic testable without spinning up a server or parsing CLI args.

### Inter-Crate Dependencies

Depend on sibling crates by path:

```toml
[dependencies]
core = { path = "../core" }
```

Dependency direction rules:
- Binary crates depend on library crates (never the reverse)
- Lower-level crates never depend on higher-level crates
- If two crates need shared types, extract a `types` or `common` crate

## Build and Test Commands

```bash
# Build everything
cargo build --workspace

# Test everything
cargo test --workspace

# Test one crate
cargo test -p core

# Lint everything
cargo clippy --workspace -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Build docs
cargo doc --workspace --no-deps
```

## Release Workflow

### Version Synchronization

Use `workspace.package.version` and inherit it:

```toml
# In member Cargo.toml
[package]
name = "my-crate"
version.workspace = true
edition.workspace = true
```

### Publishing Order

Publish leaf crates first (no local dependencies), then work up the dependency tree.

```bash
cargo publish -p types
cargo publish -p core    # depends on types
cargo publish -p cli     # depends on core
```

## Anti-Patterns

| Anti-Pattern | Problem | Fix |
|-------------|---------|-----|
| Putting all code in one crate | Slow builds, no encapsulation | Split into workspace |
| Binary crate with business logic | Cannot test without running binary | Extract to library crate |
| Circular dependencies | Will not compile | Restructure or extract shared crate |
| Version drift across crates | Mysterious conflicts | Use workspace.dependencies |
| `path` deps without workspace | Missing `Cargo.lock` sharing | Use workspace members |
| `default-features = false` everywhere | Breaks transitive features | Only disable when you know the impact |

## Checklist

- [ ] Workspace root has `resolver = "2"`
- [ ] Shared dependencies declared in `[workspace.dependencies]`
- [ ] Business logic in library crates, not binaries
- [ ] No circular dependencies between crates
- [ ] `cargo clippy --workspace -- -D warnings` passes
- [ ] `cargo test --workspace` passes
- [ ] Each crate has a clear single responsibility
