# Shipwright — Vision (Canonical)

**Status:** Active  
**Last Updated:** 2026-02-24

---

## Naming and Format Conventions

These names are authoritative across product, docs, and code:

- Company and product name: `Shipwright`
- CLI binary name: `ship`
- Project data directory: `.ship/`
- Global data directory: `~/.ship/`
- Document format: Markdown (`.md`) with TOML frontmatter (`+++`)
- Config format: TOML (`config.toml`)

These conventions are intentionally stable and should not be changed casually.

---

## North Star

**Shipwright is the operating system for software projects.**

Not a project management tool and not an AI model. Shipwright is the durable project substrate shared by humans and agents.

When the code changes, project state changes with it. When an agent starts, it reads the same project memory a human sees. When decisions are made, they are recorded as first-class project artifacts.

---

## Core Product Thesis

- Files are the universal interface that every tool and agent can use.
- Structured project memory beats chat-only context.
- The fastest teams have one source of truth for work, decisions, and context.

Shipwright exists to make software delivery more continuous, less lossy, and more reliable under heavy AI usage.

---

## PMF Wedge: Agent Configuration + Modes

The highest-probability product-market-fit candidate is the combination of:

- Unified agent configuration in one place (global + project)
- Provider pass-through support (`claude`, `gemini`, `codex`)
- MCP registry management with safe export semantics
- Modes as explicit capability boundaries

This directly solves a painful operational problem teams already have today: fragmented AI tool configuration and inconsistent capability control across workflows.

For alpha, this wedge is prioritized over broader platform expansion.

---

## Architecture Direction

Shipwright architecture is layered and plugin-oriented:

1. Runtime layer
- File-backed document primitives
- Config store and merge rules
- Mode and MCP capability control
- Event/log substrate

2. First-party modules
- Issues, specs, ADRs, and future first-party domain modules
- Built on shared runtime primitives

3. Product surfaces
- `ship` CLI
- Tauri desktop app
- MCP server and tool surface

4. Third-party extension model (later)
- SDK for external plugin authors after runtime/module contracts stabilize

### Alpha reality and anti-duplication rule

Parallel implementation has created friction. For alpha:

- The **authoritative execution path** is current `logic + cli + ui/src-tauri + ui` behavior.
- `runtime/modules/sdk` are treated as **scaffolding**, not competing production paths.
- We keep the skeleton for trajectory, but avoid duplicate business logic until the runtime migration is intentional and test-backed.

---

## Plugin Model (Phased)

### Alpha
- Keep plugin-oriented structure visible.
- Use first-party modules and stable primitives.
- No third-party SDK commitments yet.

### V1
- Harden runtime/module contracts through real first-party usage.
- Expand first-party module capabilities and internal extension points.

### V3+
- Introduce a third-party SDK once contracts are stable and proven.
- SDK scope follows production needs, not speculative API design.

---

## Alpha Product Priorities

1. AI generation via pass-through CLI (`claude`, `gemini`, `codex`)
2. Unified global + project agent config layer
3. Modes as first-class capability control
4. MCP server registry with clean export semantics
5. High-confidence primitives through robust tests

---

## Quality Bar (Non-Negotiable)

Fast iteration is only useful with durable correctness.

- Test core config merge semantics (global/project)
- Test mode invariants and active capability filtering
- Test export round-trips and user-config preservation
- Test provider pass-through command behavior and failure modes

If a feature increases speed but weakens primitives, we fix the primitives first.

---

## Out of Scope for Alpha

- Third-party plugin marketplace
- Public SDK for external extension authors
- MCP sampling as the primary generation path
- Multi-cloud orchestration and enterprise packaging

---

## Canonical Alpha Spec

Alpha implementation detail lives in:

- `.ship/specs/alpha-ai-config-and-modes.md`

Older deep-dive docs are retained as archived references only.
