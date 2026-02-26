ship_dir := ".ship"
cli     := "./target/release/ship"
mcp     := "./target/release/ship-mcp"

# List available recipes
default:
    @just --list
# ── Install ────────────────────────────────────────────────────────────────────

# Build and install `ship` + `ship-mcp` to ~/.cargo/bin (adds to PATH)
install: build
    cargo install --path crates/cli --locked
    cargo install --path crates/mcp --locked

# Alias: same as install
reinstall: install

# ── Build ──────────────────────────────────────────────────────────────────────

# Build cli + mcp (skips Tauri — needs glib-2.0 not available in WSL)
build:
    cargo build --release -p cli -p mcp

# Build in debug mode (faster iteration)
build-dev:
    cargo build -p cli -p mcp

# Build everything including Tauri UI (requires native libs)
build-all:
    cargo build --release

# ── Test ───────────────────────────────────────────────────────────────────────

# Run all tests
test:
    cargo test -p logic -p cli -p mcp

# Run logic tests only (fastest)
test-logic:
    cargo test -p logic

# Run tests with output visible
test-verbose:
    cargo test -p logic -- --nocapture

# ── Migration ──────────────────────────────────────────────────────────────────

# Migrate existing YAML issues → TOML in-place
migrate:
    SHIP_DIR={{ship_dir}} {{cli}} migrate

# ── Dev workflow ───────────────────────────────────────────────────────────────

# Build then run migration (typical dev loop after a format change)
build-migrate: build migrate

# Watch for changes and rebuild (requires cargo-watch)
watch:
    cargo watch -x "build -p cli -p mcp"

# ── Tauri ──────────────────────────────────────────────────────────────────────

# Run Tauri dev server (UI hot-reload)
tauri-dev:
    cd crates/ui && npm run tauri dev

# ── Ship CLI shortcuts ─────────────────────────────────────────────────────────

# List all issues
issues:
    SHIP_DIR={{ship_dir}} {{cli}} issue list

# List issues by status
issues-backlog:
    SHIP_DIR={{ship_dir}} {{cli}} issue list --status backlog

issues-in-progress:
    SHIP_DIR={{ship_dir}} {{cli}} issue list --status in-progress

# Scan for ghost issues (TODO/FIXME/HACK/BUG)
ghost:
    SHIP_DIR={{ship_dir}} {{cli}} ghost scan

# Run projects-module e2e checks
e2e-projects:
    ./example/projects-e2e/e2e/project-features.sh

# Reset local example workspace with fresh generated .ship data
e2e-reset:
    ./example/projects-e2e/reset.sh

# ── MCP ────────────────────────────────────────────────────────────────────────

# Start the MCP server manually (usually Claude Code does this)
mcp-start:
    {{mcp}}

# ── Housekeeping ───────────────────────────────────────────────────────────────

# Run clippy
lint:
    cargo clippy -p logic -p cli -p mcp -- -D warnings

# Format all Rust code
fmt:
    cargo fmt

# Check formatting without modifying
fmt-check:
    cargo fmt --check

# Remove build artifacts
clean:
    cargo clean
