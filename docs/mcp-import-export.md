# MCP Import / Export

This document defines the MCP configuration sync contract between Ship and provider-native config files.

## Canonical Source Of Truth

- Ship canonical MCP registry: `.ship/agents/mcp.toml`
- Provider config files are treated as integration surfaces, not source-of-truth.

## Commands

### Export Ship MCP registry to provider config

```bash
ship mcp export --target claude
ship mcp export --target gemini
ship mcp export --target codex
```

Alias:

```bash
ship config export --target <provider>
```

### Import provider MCP registry into Ship

```bash
ship mcp import claude
ship mcp import gemini
ship mcp import codex
```

For full provider surface import (MCP + permissions):

```bash
ship providers import <provider>
```

## Import Path Resolution

For each provider, Ship checks:

1. project config path (`$WORKSPACE_ROOT/<provider path>`)
2. global config path (`$HOME/<provider path>`)

Rules:

- If project config exists, Ship imports only project config.
- If project config is missing, Ship falls back to global config.
- If both are missing, no-op.

## Import Validation / Guardrails

Imported entries are filtered before they are written to `.ship/agents/mcp.toml`:

- Reserved server ID `ship` is never imported.
- Empty IDs are skipped.
- Invalid stdio servers (missing `command`) are skipped.
- Invalid HTTP/SSE servers (missing `url`) are skipped.
- Servers already tracked as Ship-managed in runtime state are skipped.
- Existing IDs in Ship config are deduped.

Scope tagging:

- Imports from project config are tagged `scope = "project"`.
- Imports from global config are tagged `scope = "global"`.

## Export Guarantees

When exporting to provider configs:

- `ship` MCP server is always injected.
- User-defined provider MCP entries are preserved.
- Previously Ship-managed entries are replaced with current resolved set.
- Disabled servers are not exported.
- Active mode MCP filters are applied before export.

## Provider-Specific Targets

- Claude: `.mcp.json`
- Gemini: `.gemini/settings.json`
- Codex: `.codex/config.toml`

## Test Coverage

Runtime coverage lives in:

- `core/runtime/src/agents/export/sections/tests.rs`

Key cases:

- project/global fallback behavior
- project-over-global precedence
- reserved/invalid entry filtering
- dedupe and managed-state safety
- provider-specific export shape assertions
