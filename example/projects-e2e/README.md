# Projects E2E Workspace

Use this folder to validate project workflows without polluting the main repo's tracked `.ship/` state.

## Bootstrap

Reset to a clean, reproducible local workspace:

```bash
./reset.sh
```

Then, from this directory:

```bash
cargo run --manifest-path ../../Cargo.toml -p cli -- demo .
```

If you have `ship` installed globally, you can use:

```bash
ship demo .
```

## Quick Checks

From this directory:

```bash
cargo run --manifest-path ../../Cargo.toml -p cli -- issue list
cargo run --manifest-path ../../Cargo.toml -p cli -- config status list
cargo run --manifest-path ../../Cargo.toml -p cli -- mode list
cargo run --manifest-path ../../Cargo.toml -p cli -- event list --since 0 --limit 20
cargo run --manifest-path ../../Cargo.toml -p cli -- mcp
```

UI check (from repository root):

```bash
pnpm --dir crates/ui tauri dev
```

Then open this folder in the UI as the active project.

## Scope Matrix

See `feature-matrix.md` for the requirement-to-implementation matrix used during alpha validation.

By default this workflow treats release/feature/spec/ADR artifacts as committed project memory, while issue execution data remains local unless explicitly included.

## E2E Checks

Run the end-to-end check suite from repo root:

```bash
./example/projects-e2e/e2e/project-features.sh
```

This suite validates:
- CLI release/feature/spec/issue workflows
- MCP release+feature tool parity (via `cargo test -p mcp`)
- Filesystem ingest into the event stream
