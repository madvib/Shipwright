# Binary Gate

Ship uses a Cargo `unstable` feature flag to separate in-progress features from the release surface.

- `just build` — dev build with all features (`+unstable`). Default for development.
- `just build-release` — stable build, what users get.
- `just check-gates` — verify both compile. Run before merging.

**Agents must use `just build` when rebuilding.**
Rebuilding without the flag drops unstable MCP tools from every active session.

Check your build: `ship --version`. `+unstable` = dev. No suffix = stable.
