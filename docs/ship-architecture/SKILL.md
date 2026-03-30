---
name: ship-architecture
stable-id: ship-architecture
description: Use when understanding Ship's three-layer architecture — compiler, runtime, transport. How the compilation pipeline works, how state is managed, how transport layers connect.
tags: [ship, architecture, concepts]
authors: [ship]
---

# Ship Architecture

Ship is structured in three layers: compiler (pure transformation), runtime (state management), and transport (CLI, MCP, web). Domain logic lives in the compiler and runtime crates. Transport layers are thin dispatchers.

For full details see `references/docs/`.
