# D1 Schema Architecture

Ship Studio uses **two separate D1 databases** to enforce separation of concerns.

## ship-auth (AUTH_DB)

Auth-only database. Tables managed by Better Auth plus CLI auth state.

| Table | Purpose |
|-------|---------|
| `user` | Better Auth user records |
| `session` | Better Auth sessions |
| `account` | Better Auth OAuth accounts |
| `verification` | Better Auth verification tokens |
| `cli_auth_state` | PKCE flow state for CLI login |
| `cli_auth_codes` | Short-lived auth codes issued after CLI OAuth |

Better Auth column names are **sacred** -- do not rename `createdAt`, `updatedAt`, `emailVerified` etc.

## ship-registry (REGISTRY_DB)

Public catalog and MCP aggregator. No user PII stored here.

| Table | Purpose |
|-------|---------|
| `packages` | Package registry entries |
| `package_versions` | Version records per package |
| `package_skills` | Skill metadata per version |
| `github_installations` | GitHub App installation tracking |
| `mcp_servers` | MCP server registry (replaces mcp_servers_cache) |

Cross-DB references (e.g. `packages.claimed_by` referencing `user.id`) use plain TEXT columns with no FK constraint.

## Removed Tables

The following tables from the legacy single-DB schema have been deleted:

- `libraries`, `profiles`, `workflows` -- user data moved to localStorage/GitHub
- `orgs`, `org_members` -- fake multi-tenancy removed (org always = user.id)
- `workspaces`, `cloud_jobs` -- unused cloud features
- `mcp_servers_cache` -- replaced by `mcp_servers` with a proper schema

## Migration Files

- `migrations/auth/0001_initial.sql` -- AUTH_DB initial schema
- `migrations/registry/0001_initial.sql` -- REGISTRY_DB initial schema
