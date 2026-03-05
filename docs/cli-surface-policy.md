# CLI Surface Policy

## Purpose

Keep the `ship` CLI product surface intentional as the codebase scales.

## Command Tiers

### 1. Stable User Surface

Commands intended for normal product use:

- workflow/entity management
- workspace flows
- provider/mode/config operations

These should remain discoverable in `ship --help`.

### 2. Advanced Surface

Power-user commands that are still supported but may require deeper context.

- export/import helpers
- diagnostics

These may be visible, but should be documented with clear caveats.

### 3. Dev-Only Surface

Maintenance and migration tooling that should not be part of the default product UX.

- `ship dev migrate`
- one-off repair/reindex commands

Rules:

- Keep these under `ship dev ...` and hide by default in help.
- Do not preserve old command aliases when there are no downstream consumers.
- Avoid adding new dev-only commands at top-level.

## Compatibility Policy

- Backward compatibility is not a default requirement for CLI/MCP/runtime/config surfaces.
- If no downstream consumer depends on a surface, remove old paths in the same change.
- Only data-safety exceptions are allowed (for migrations preventing data loss/corruption).

## Enforcement Guidance

- Every new command must declare its tier during review.
- If a command is one-off, put it in `dev` or a separate maintenance binary.
- Prefer runtime/service-layer APIs; keep CLI handlers thin orchestration only.
