# E2E Workspace

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
cargo run --manifest-path ../../Cargo.toml -p cli -- feature list
cargo run --manifest-path ../../Cargo.toml -p cli -- release list
cargo run --manifest-path ../../Cargo.toml -p cli -- workspace list
cargo run --manifest-path ../../Cargo.toml -p cli -- config status list
cargo run --manifest-path ../../Cargo.toml -p cli -- mode list
cargo run --manifest-path ../../Cargo.toml -p cli -- event list --since 0 --limit 20
cargo run --manifest-path ../../Cargo.toml -p cli -- mcp list
```

UI check (from repository root):

```bash
pnpm --dir crates/ui tauri dev
```

Then open this folder in the UI as the active project.

## Scope Matrix

See `test-matrix.md` for the requirement-to-implementation matrix used during alpha validation.

By default this workflow keeps project/workflow docs local, while `ship.toml`, MCP config, permissions, and rules remain tracked.

## E2E Checks

Run the end-to-end check suite from repo root:

```bash
./examples/e2e/checks/project-features.sh
```

Set `KEEP_TMP=1` to preserve the generated workspace for post-run inspection.

This suite validates:
- CLI release/feature/workspace workflows
- Agent config and MCP runtime registration
- Event stream coverage for workspace + session lifecycle

## Compiler Fixtures

Fixture projects for multi-provider config compilation live under:

- `../projects/webapp-nextjs/`
- `../projects/rust-cli/`

Run the compiler-matrix e2e tests from repo root:

```bash
SHIP_BIN=./target/debug/ship cargo test -p examples-e2e --test compiler_matrix
```
