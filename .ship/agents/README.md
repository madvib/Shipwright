# agents/

Agent runtime configuration and policy.

- `mcp.toml`: MCP server registry (single source of truth).
  - MCP import/export contract: `docs/mcp-import-export.md`
- Modes: persisted in SQLite runtime state.
- `permissions.toml`: sandbox and command guardrails.
  - Canonical schema: `docs/agent-permissions-schema.md`
- `rules/`: always-on guardrails.
- Skills live outside repo-local state:
  - global: `~/.ship/skills`
  - project-scoped: `~/.ship/projects/<project>/skills`
- `profiles.md`: task profile map for daily execution.
- `skill-library-strategy.md`: published-skill curation plan (`skills.sh`).
- `published-skills.md`: candidate external skills and install commands.
