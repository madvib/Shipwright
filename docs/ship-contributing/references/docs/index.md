---
group: Contributing
title: Contributing
description: How to contribute to Ship -- repo layout, dev environment, and workflow.
order: 1
---

# Contributing

A Rust + TypeScript monorepo. The Rust workspace contains the compiler, runtime, CLI, and MCP server. The TypeScript side has the web app (Studio), docs site, and shared npm packages.

## Repository Layout

```
apps/
  ship-studio-cli/       CLI binary (Rust, clap)
  mcp/                   MCP server (Rust, rmcp) -- stdio and HTTP transports
  web/                   Ship Studio (TanStack Start + Cloudflare Workers)
  docs/                  Documentation site (Astro + Starlight)
crates/core/
  compiler/              Types, resolution, output generation. Compiles to native + WASM.
  runtime/               State management -- workspaces, sessions, events, DB, skill vars
  cli-framework/         Shared CLI metadata and scaffolding
  mcp-framework/         Shared MCP app lifecycle scaffolding
crates/
  xtask/                 Build automation tasks
packages/
  compiler/              @ship/compiler -- WASM npm package (built from crates/core/compiler)
  primitives/            @ship/primitives -- shared UI components (shadcn)
  ui/                    @ship/ui -- generated types from Rust via Specta
  assets/                Shared static assets
contrib/
  ship-dev.sh            Dev container launcher (Podman)
  Containerfile          Dev container image definition
schemas/
  vars.schema.json       JSON Schema for skill variables
```

## Getting Started

### Option 1: Dev Container (recommended)

The dev container includes all dependencies pre-installed: Rust toolchain, Node via fnm, pnpm, wasm-pack, just, and standard dev tools. Uses Podman (Docker-compatible).

```bash
cp contrib/ship-dev.sh ~/bin/ship-dev
chmod +x ~/bin/ship-dev
ship-dev start
```

First run builds the image, installs JS deps, and builds the ship binary. Subsequent sessions attach instantly with `ship-dev shell`.

The container mounts the repo at `/workspaces/ship` and exposes ports for Studio (3000), docs (4321), and the MCP server (51741).

### Option 2: Local Setup

Prerequisites: Rust (rustup + stable), Node.js (via fnm or nvm), pnpm, wasm-pack, just, git.

```bash
git clone https://github.com/madvib/ship.git
cd ship
pnpm install
cargo build --features unstable -p ship-studio-cli -p mcp
```

## Running Tests

Ship uses a Justfile for common tasks. Run `just` with no arguments to list all commands.

```bash
just test              # All tests (Rust + web)
just test-rust         # Rust workspace (unstable features enabled)
just test-rust-stable  # Stable surface only (what CI gates on)
just test-runtime      # Runtime crate only (fastest Rust feedback)
just test-compiler     # Compiler crate only
just test-web          # Web app (vitest)
just typecheck         # TypeScript type check
```

## Building

```bash
just build             # Dev build (unstable features)
just build-release     # Release binary (stable, what users get)
just install           # Install CLI to ~/.cargo/bin
just wasm              # Rebuild WASM compiler
```

## Workflow

1. Create a branch for your change.
2. Write a failing test first.
3. Implement the minimum change to pass the test.
4. Run `just test` and `just lint` before committing.
5. Follow commit conventions (see [Coding Standards](standards.md)).
6. Open a PR against `main`.
