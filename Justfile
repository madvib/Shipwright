# ── Ship Development ───────────────────────────────────────────────────────────

default:
    @just --list

# ── Dev ────────────────────────────────────────────────────────────────────────

# Start Studio dev server (Vite + local D1)
dev:
    pnpm --filter web dev

# Watch and rebuild Rust on changes (unstable — includes all dev tools)
watch:
    cargo watch -x "build --features unstable -p ship-studio-cli -p mcp"

# ── Build ──────────────────────────────────────────────────────────────────────

# Build CLI + MCP (unstable dev build — default for development)
build:
    cargo build --features unstable -p ship-studio-cli -p mcp

# Build release binary (stable — no unstable features, what users get)
build-release:
    cargo build --release -p ship-studio-cli -p mcp

# Install stable binary to ~/.cargo/bin
install: build-release
    cargo install --path apps/ship-studio-cli

# Rebuild WASM compiler (requires build-essential)
wasm:
    wasm-pack build crates/core/compiler --target web --out-dir ../../../packages/compiler --out-name compiler -- --features wasm

# ── Test ───────────────────────────────────────────────────────────────────────

# Run all tests (Rust + web)
test: test-rust test-web

# Rust workspace tests (unstable)
test-rust:
    cargo test --workspace --features unstable

# Rust tests — stable surface only (CI release gate)
test-rust-stable:
    cargo test --workspace

# Runtime crate only (fastest Rust)
test-runtime:
    cargo test -p runtime

# Compiler crate only
test-compiler:
    cargo test -p compiler

# Web app tests (vitest)
test-web:
    pnpm --filter web test

# TypeScript type check
typecheck:
    pnpm --filter web exec tsc --noEmit

# ── Database ───────────────────────────────────────────────────────────────────

# Apply D1 migrations locally
db-migrate:
    cd apps/web && npx wrangler d1 migrations apply ship --local

# Regenerate Cloudflare worker types from wrangler.jsonc
cf-types:
    cd apps/web && npx wrangler types

# ── Lint & Format ──────────────────────────────────────────────────────────────

# Rust clippy (unstable)
lint:
    cargo clippy --workspace --features unstable -- -D warnings

# Format Rust
fmt:
    cargo fmt

# Check Rust formatting
fmt-check:
    cargo fmt --check

# ── Docs ──────────────────────────────────────────────────────────────────────

# Build docs site
docs:
    cd apps/docs && npx tsx scripts/collect-skill-docs.ts && pnpm build

# Dev docs site
docs-dev:
    cd apps/docs && pnpm dev --port 4321 --host

# ── Deploy ─────────────────────────────────────────────────────────────────────

# Deploy Studio to Cloudflare Workers
deploy:
    pnpm --filter web deploy

# Apply D1 migrations to remote (production)
db-migrate-remote:
    cd apps/web && npx wrangler d1 migrations apply ship --remote

# ── Registry ───────────────────────────────────────────────────────────────────

# Seed the local registry with @unofficial packages
seed:
    @echo "Seeding local registry — requires dev server running on :3000"
    curl -s -X POST http://localhost:3000/api/registry/seed \
      -H "Content-Type: application/json" \
      -H "X-Seed-Secret: ${SEED_SECRET}" \
      -H "Cookie: ${AUTH_COOKIE}" | jq .

# Publish a package to the local registry
publish repo:
    curl -s -X POST http://localhost:3000/api/registry/publish \
      -H "Content-Type: application/json" \
      -H "Cookie: ${AUTH_COOKIE}" \
      -d '{"repo_url": "{{repo}}"}' | jq .

# ── Ship CLI ──────────────────────────────────────────────────────────────────

# Compile agent config for current project
compile:
    ship compile

# Activate an agent profile
use profile="default":
    ship use {{profile}}

# View targets, capabilities, jobs in TUI
view:
    ship view

# ── Housekeeping ───────────────────────────────────────────────────────────────

# Remove build artifacts
clean:
    cargo clean
    rm -rf apps/web/dist apps/web/.wrangler/deploy

# Nuke local D1 and re-migrate
db-reset:
    rm -rf apps/web/.wrangler/state/v3/d1
    just db-migrate

# Rebuild everything from scratch
rebuild: clean build wasm
    pnpm install
    just db-migrate

# Verify both stable and unstable compile (CI check)
check-gates:
    cargo build -p ship-studio-cli -p mcp
    cargo build --features unstable -p ship-studio-cli -p mcp
    @echo "Both gates compile ✓"
