When reviewing a pull request on Shipwright, check each of the following.

## Correctness

- [ ] Does it solve the stated problem without introducing new bugs?
- [ ] Are edge cases handled (empty states, missing files, SQLite errors, hook failures)?
- [ ] Is error handling explicit — no silent failures, no bare `.unwrap()` in library code?
- [ ] If schema changed: is there a migration file? Does it handle existing data?

## Architecture

- [ ] Does it stay within module namespace boundaries?
- [ ] Does runtime state go to SQLite and intentional state go to markdown files?
- [ ] Are new Tauri commands specta-typed — no manually written TypeScript interfaces?
- [ ] Does anything write to generated files directly? (It shouldn't — context generation owns that)

## Tests

- [ ] New functionality has tests in `crates/runtime/tests/` or the relevant module
- [ ] `cargo test -p runtime` passes
- [ ] If git hooks are touched: manual smoke test documented in PR description

## Code Quality

- [ ] `cargo clippy --all-targets -- -D warnings` clean
- [ ] `cargo fmt --check` passes
- [ ] No commented-out code
- [ ] No TODO comments without an associated issue number

## Documentation

- [ ] If public API changed: ADR proposed or existing ADR updated
- [ ] If new MCP tool added: tool listed in relevant mode's `shipwrightTools`
- [ ] If new config field added: schema updated at `schema.shipwright.dev`

## Alpha Scope

- [ ] Does this fit within alpha scope? (See `alpha-scope` prompt for the list)
- [ ] If it's a V1/V2 feature: is there an issue for it and is it clearly deferred?
