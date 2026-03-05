# CLI/MCP Offerings and Install Paths

## What We Ship Today

### 1) `ship` (CLI binary)

- Crate: `crates/cli`
- Binary name: `ship`
- Purpose: primary command-line interface for Ship workflows.

### 2) `ship-mcp` (MCP server binary)

- Crate: `crates/mcp`
- Binary name: `ship-mcp`
- Purpose: standalone MCP stdio server for agent/tool integrations.

### 3) Shipwright Desktop App (Tauri)

- Crate: `crates/ui/src-tauri`
- Binary name in this crate: `ship`
- Behavior:
  - no CLI args -> launches GUI
  - CLI args present -> routes to shared CLI handlers
  - `mcp serve` path -> calls MCP server entrypoint

Important: PATH installs via `cargo install --path crates/cli` install the **CLI crate binary**, not the desktop app bundle binary.

## Rebuild + Update PATH Binary

From repo root:

```bash
# Build local debug artifacts
cargo build -p cli -p mcp

# Install/replace PATH binaries in ~/.cargo/bin
cargo install --path crates/cli --force --locked
cargo install --path crates/mcp --force --locked
```

Verify:

```bash
which ship
ship --version
ship version

which ship-mcp
ship-mcp  # starts MCP stdio server
```

If `~/.cargo/bin` is not on PATH:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

## Dependency Model (Current Truth)

### Runtime vs Framework

- `core/runtime` is still the canonical AgentOS engine (state, config, events, persistence, context/export logic).
- `core/cli-framework` is a transport/framework layer that:
  - owns lifecycle hooks and built-in core commands (`init`, `doctor`, `version`)
  - owns several core primitive handlers (`skill`, `mode`, `event`, `providers`, `mcp`)
  - depends on `core/runtime`.
- `core/mcp-framework` is currently lifecycle/bootstrap focused (metadata, preflight/serve/postflight, banners) and depends on `core/runtime`.

### App crates

- `crates/cli` composes `cli-framework` and still handles Ship-specific domain commands (`issue`, `feature`, `release`, `spec`, `adr`, `note`, plus dev commands).
- `crates/mcp` composes `mcp-framework` but still implements Ship tool handlers that call `runtime` and Ship modules directly.

Bottom line: CLI/MCP are partially abstracted by framework crates, but both still rely on `core/runtime` (directly and/or through framework).
