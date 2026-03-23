# CLI Reference

This reference is maintained as the `ship-cli-reference` skill. Agents get it in their compiled context automatically.

For the full reference, see [.ship/skills/ship-cli-reference/SKILL.md](../.ship/skills/ship-cli-reference/SKILL.md).

Quick command list:

```
ship init                         Scaffold .ship/
ship use <agent>                  Activate and compile
ship compile                      Recompile active agent
ship status                       Show active agent
ship validate                     Check config for errors

ship agent list|create|edit|clone|delete
ship skill add|list|remove|create
ship mcp serve|add|add-stdio|list|remove

ship add <package>                Add dependency
ship install [--frozen]           Resolve and fetch deps
ship publish [--dry-run]          Publish to registry

ship docs <topic>                 Extended help
ship view                         Terminal UI
```
