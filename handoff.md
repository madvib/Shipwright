# Handoff: Artifact-to-Event Registration + KernelRouter Split-Brain Fix

Branch: `job/event-registration`

## What was done

### 1. `artifacts` field on Skill types
Both the runtime (`crates/core/runtime/src/agents/skill.rs`) and compiler (`crates/core/compiler/src/types/skill.rs`) `Skill` structs now have `artifacts: Vec<String>`. The loader (`apps/ship-studio-cli/src/loader.rs`) and dep-skills parser (`apps/ship-studio-cli/src/dep_skills.rs`) both parse the YAML inline array format (`artifacts: [html, adr]`).

### 2. Artifact → event mapping
`crates/core/runtime/src/events/artifact_events.rs` — new module:
- `events_for_artifact(artifact_type)` → platform event suffix(es)
- `skill_event_subscriptions(skills)` → `ship.{suffix}` + `{skill.id}.` per skill, deduplicated
- `skill_custom_namespaces(skills)` → only `{skill.id}.` namespaces

The same mapping is duplicated in `crates/core/compiler/src/compile/skills.rs` — compiler can't depend on runtime; this is intentional.

### 3. Dynamic actor subscriptions
`apps/mcp/src/server/mod.rs` — `spawn_agent_actor()` now loads project skills via `runtime::list_skills()` and builds subscriptions dynamically instead of hardcoding them.

`apps/mcp/src/studio_server.rs` — `spawn_studio_actor()` subscribes to all skill custom namespaces.

### 4. Split-brain KernelRouter fix
`apps/mcp/src/http.rs` — `build_studio_app()` now serves **two** MCP endpoints in the same process:
- `/mcp` → `StudioServer` (web UI)
- `/agent` → `ShipServer` (agent connections)

Both use `init_kernel_router` (idempotent via `OnceLock`) so they share the same in-process router. Agents connect to `/agent` and their events route directly to Studio's mailbox.

### 5. HTTP transport when Studio is active
`crates/core/compiler/src/types/agent_profile.rs` — added `ProfileApps { studio: bool }` to `AgentProfile`.

`crates/core/compiler/src/resolve.rs` — `ResolvedConfig` gains `studio_mcp_url: Option<String>`. `resolve_library()` sets it when the active agent profile has `apps.studio = true`.

`crates/core/compiler/src/compile/mcp.rs` — `build_mcp_servers()` accepts `studio_url: Option<&str>` and emits `{"url": "http://localhost:PORT/agent"}` instead of stdio when set. Same for `gemini.rs` and `cursor.rs`.

### 6. `event_subscriptions` in CompileOutput
`crates/core/compiler/src/compile/mod.rs` — `CompileOutput.event_subscriptions: Vec<String>` added. Populated from `skills::resolve_event_subscriptions()`.

### 7. Remove REST `/studio/event` endpoint
`apps/web/src/features/studio/useLocalMcp.ts` — removed `postStudioEvent`.
`apps/web/src/features/studio/session/useSessionHandlers.ts` — now calls `mcp.callTool('emit_studio_event', ...)` directly.

## Test status
- `cargo test -p compiler`: 419 passed, 0 failed
- `cargo test -p runtime`: 522 passed, 2 pre-existing failures (worktree `.ship` resolution, unrelated)
- `just check-gates`: Both gates compile

## Known pre-existing failures (not introduced here)
- `project::tests::resolve_project_ship_dir_prefers_main_ship_over_worktree_copy`
- `tests::test_get_project_dir_prefers_main_ship_when_worktree_has_local_copy`

Both fail on `main` before this branch's changes.

## What's not done (out of scope / future work)
- Integration tests for cross-actor event delivery
- WASM rebuild — `CompileOutput.event_subscriptions` field added; `wasm-pack build` needed before using from JS
- `studio_port` field in `ProjectLibrary` TOML schema documentation
