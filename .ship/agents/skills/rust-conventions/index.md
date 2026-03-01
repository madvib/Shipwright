## Codebase Structure

Shipwright is a Rust monorepo with the following crates:

- `crates/runtime` — core substrate: SQLite repositories, document model, MCP server, event bus, module store
- `crates/sdk` — ShipwrightModule trait definition
- `crates/cli` — CLI binary
- `crates/ui` — Tauri desktop application
- `crates/modules/issues` — issues module
- `crates/modules/specs` — specs module
- `crates/modules/adrs` — ADRs module
- `crates/modules/notes` — notes module
- `crates/modules/git` — git hooks, worktree lifecycle, context generation

## Before Committing

Always run before committing:

```
cargo test -p runtime
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

## Database

- All SQLite access goes through typed repositories in `crates/runtime/src/db/`
- No raw SQL outside repository files
- Use `sqlx::migrate!()` for all schema changes — never modify schema directly
- SQLx compile-time query verification is required — `DATABASE_URL` must be set in `.env`
- Schema changes require a new migration file in `crates/runtime/migrations/`

## Error Handling

- Use `thiserror` for library errors, `anyhow` for binary errors
- Never `.unwrap()` in library code — propagate with `?`
- Tauri commands return `Result<T, String>` — use `.map_err(|e| e.to_string())`

## Async

- Tauri async runtime throughout — use `tauri::async_runtime::spawn` for background tasks
- SQLx async queries — always `.await`
- No `std::thread::sleep` — use `tokio::time::sleep`

## File Conventions

- Document files use TOML frontmatter delimited by `+++`
- Config files are JSONC with `$schema` field
- Status is derived from directory path, never stored in frontmatter
- IDs follow `<type>-NNN` format: `issue-001`, `spec-023`, `adr-001`
