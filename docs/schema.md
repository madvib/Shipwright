# Schema Reference

This reference is maintained as the `ship-schema-reference` skill. Agents get it in their compiled context automatically.

For the full reference, see [.ship/skills/ship-schema-reference/SKILL.md](../.ship/skills/ship-schema-reference/SKILL.md).

## JSON Schemas

Editor autocompletion is provided by JSON Schemas in the `schemas/` directory:

- [ship.schema.json](../schemas/ship.schema.json) — project manifest
- [agent.schema.json](../schemas/agent.schema.json) — agent profiles
- [mcp.schema.json](../schemas/mcp.schema.json) — MCP server definitions
- [permissions.schema.json](../schemas/permissions.schema.json) — permission presets

Reference them in your config files:

```jsonc
{ "$schema": "https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json" }
```

Or use relative paths for local development:

```jsonc
{ "$schema": "../../schemas/agent.schema.json" }
```
