---
title: "Coding Standards"
description: "Commit conventions, file limits, TDD, type safety, and architecture rules."
sidebar:
  label: "Coding Standards"
  order: 3
---
## Commit Conventions

Every commit message starts with a type prefix:

| Type | Use for |
|------|---------|
| `feat` | New feature |
| `fix` | Bug fix |
| `refactor` | Code restructuring (no behavior change) |
| `test` | Adding or updating tests |
| `docs` | Documentation changes |
| `chore` | Build, CI, dependency updates |

Keep subjects imperative and concise. Stage explicit files -- avoid `git add -A` or `git add .` to prevent committing sensitive files or build artifacts. No AI attribution or co-author noise in commit messages.

## Test-Driven Development

Write the failing test before the implementation. This is not optional.

- Cover happy paths and meaningful failure paths.
- Add or update tests for every bug fix and behavior change.
- One test at a time. Never batch-write tests.
- After 5-8 tests, checkpoint the spec.

Rust tests: `just test-rust`. Web tests: `just test-web`.

## File Length

Maximum 300 lines per file. If a module needs more, split it. Applies to Rust, TypeScript, and documentation files.

## Architecture Rules

**One way to do things.** If a solution exists, use it. Do not build a parallel system. One auth system (Better Auth), one parser (WASM compiler), one migration tool (drizzle-kit).

**No backward compatibility without downstream consumers.** Make hard breaks. Only allow temporary compat for data-safety migrations with explicit scope, removal criteria, and a test that fails once the exception expires.

**Transport thin, domain in runtime.** CLI and MCP are dispatchers. Coordinated state logic belongs in the runtime crate, not in transport layers.

**Compiler is pure.** No filesystem, no network, no database. `ProjectLibrary` in, `CompileOutput` out.

**Events are append-only.** Never update or delete events.

**Idempotent by default.** `ship use` can run repeatedly. The compiler overwrites artifacts. The runtime uses upsert patterns.

## Type Safety

Types come from exactly two places:

1. **`@ship/ui`** -- generated from the Rust compiler via Specta. Source of truth for agent, compiler, and config types. Import directly where needed.
2. **`#/db/schema`** -- Drizzle D1 schema. Source of truth for server-side data (packages, profiles, workflows).

Rules:

- No local `types.ts` barrel files that re-export or wrap generated types.
- API response types derive from schema types, never standalone interfaces.
- Use TanStack Start `createServerFn` for end-to-end type safety.
- UI-only state (dialog open, scroll position, active tab) is the only legitimate local type definition.
- No hardcoded data arrays in components. Data comes from props, hooks, or API.

Any type that duplicates, shadows, or extends a generated type creates drift. Do not create translation layers.

## Error Handling

Error messages must be actionable and specific. No silent fallbacks. If something fails, the user should know what happened and what to do about it.

## Code Organization

- Rust domain logic in `crates/core/runtime/` (state) or `crates/core/compiler/` (transformation).
- CLI and MCP stay thin -- map inputs, format outputs.
- React component state and API contracts must be explicit and stable.
- `@ship/primitives` for shared UI (shadcn). `@ship/ui` for generated types. Import directly, no local wrappers.

## Review Checklist

Before merging, review for:

- Regressions in existing behavior
- Architecture drift (logic in wrong layer, new parallel systems)
- Missing tests for new or changed behavior
- Silent error handling or swallowed failures
- Files exceeding 300 lines
