# D1 Database Schema

Audited 2026-03-21. Status: ACTIVE = has consumers, DEAD = zero references outside migrations.

## 0001 -- Better Auth (`user`, `session`, `account`, `verification`)

All four tables are managed by the better-auth library via Drizzle adapter.
`user` is also directly queried by `/api/me`, `/auth/cli-callback`, `/api/auth/refresh`.
`session` and `account` are cleaned up by `/api/auth/delete-account`.
`verification` is used by better-auth for email verification.

**user**: id PK, name, email (UNIQUE), emailVerified, image, createdAt, updatedAt
**session**: id PK, expiresAt, token (UNIQUE), createdAt, updatedAt, ipAddress, userAgent, userId FK->user
**account**: id PK, accountId, providerId, userId FK->user, accessToken, refreshToken, idToken, accessTokenExpiresAt, refreshTokenExpiresAt, scope, password, createdAt, updatedAt
**verification**: id PK, identifier, value, expiresAt, createdAt, updatedAt

Status: all ACTIVE.

## 0002 -- Pre-org app tables (DEAD)

**workspace** (singular): id PK, name, userId FK->user, createdAt, updatedAt
**project**: id PK, name, workspaceId FK->workspace, createdAt, updatedAt

Status: both DEAD. Superseded by `workspaces` (plural) in 0003. Zero code references.
No Drizzle schema definitions. Dropped in `0009_cleanup.sql`.

## 0003 -- Org model

**orgs**: id PK, name, slug (UNIQUE), created_at
Consumers: `/auth/cli-callback`, `/api/me`

**org_members**: id PK, org_id FK->orgs, user_id FK->user, role (DEFAULT 'member'), created_at. UNIQUE(org_id, user_id)
Consumers: `/auth/cli-callback`

**workspaces** (plural): id PK, org_id FK->orgs, name, branch (DEFAULT 'main'), status (DEFAULT 'idle'), created_at
Consumers: `/api/workspaces` (GET, POST)

**agent_sessions**: id PK, workspace_id FK->workspaces, provider, started_at, ended_at
Status: DEAD. Zero code references. No Drizzle schema. Dropped in `0009_cleanup.sql`.

## 0004 -- Cloud job queue

**cloud_jobs**: id PK, org_id FK->orgs, workspace_id FK->workspaces, type, status (DEFAULT 'pending'), payload, created_at, updated_at
Consumers: `/api/jobs` (GET)

## 0005 -- CLI auth (PKCE)

**cli_auth_state**: state PK, code_challenge, redirect_uri, created_at
Consumers: `/auth/cli` (INSERT), `/auth/cli-callback` (SELECT, DELETE)

**cli_auth_codes**: code PK, user_id FK->user, org_id FK->orgs, code_challenge, created_at, used (DEFAULT 0)
Consumers: `/auth/cli-callback` (INSERT), `/api/auth/token` (SELECT, UPDATE)

## 0006 -- User data

**libraries**: id PK, org_id, user_id, name, slug, data (DEFAULT '{}'), created_at, updated_at
Indexes: `libraries_org_user(org_id, user_id)`, `libraries_org_slug(org_id, slug)` UNIQUE
Consumers: `repositories.ts`, `/api/libraries`, `/api/libraries/$id`, `/api/auth/delete-account`

**profiles**: id PK, org_id, user_id, name, content, provider, created_at, updated_at
Index: `profiles_org_user(org_id, user_id)`
Consumers: `repositories.ts`, `/api/profiles`, `/api/profiles/$id`, `/api/auth/delete-account`

**workflows**: id PK, org_id, user_id, name, definition (DEFAULT '{}'), created_at, updated_at
Index: `workflows_org_user(org_id, user_id)`
Consumers: `repositories.ts`, `/api/workflows`, `/api/workflows/$id`

## 0007 -- Package registry

**packages**: id PK, path (UNIQUE), scope (DEFAULT 'community'), name, description, repo_url, default_branch (DEFAULT 'main'), latest_version, content_hash, source_type (DEFAULT 'native'), claimed_by FK->user, deprecated_by, stars (DEFAULT 0), installs (DEFAULT 0), indexed_at, updated_at
Indexes: `packages_scope_installs(scope, installs)`, `packages_path(path)`
Consumers: `registry-repositories.ts`, all `/api/registry/` endpoints

**package_versions**: id PK, package_id FK->packages, version, git_tag, commit_sha, content_hash, skills_json, agents_json, indexed_at
Index: `pkg_versions_package_indexed(package_id, indexed_at)`
Consumers: `registry-repositories.ts`, `/api/registry/$path`, publish, webhook

**package_skills**: id PK, package_id FK->packages, version_id FK->package_versions, skill_id, name, description, content_hash, content_length (DEFAULT 0)
Indexes: `pkg_skills_content_hash(content_hash)`, `pkg_skills_package(package_id)`
Consumers: `registry-repositories.ts`, `/api/registry/$path`, publish, seed, duplicates

## 0008 -- MCP server cache

**mcp_servers_cache**: id PK, name, description, homepage, tags (JSON), vendor, package_registry, command, args (JSON), vetted (DEFAULT 0), image_url, cached_at
Index: `idx_mcp_cache_vetted(vetted)`
Consumers: `lib/mcp-registry.ts`, `/api/mcp/servers`
Note: cache layer only, not source of truth. 24h TTL.

## Dead tables summary

| Table | Migration | Reason | Cleanup |
|-------|-----------|--------|---------|
| `workspace` (singular) | 0002 | Superseded by `workspaces` (0003) | 0009_cleanup.sql |
| `project` | 0002 | Never used | 0009_cleanup.sql |
| `agent_sessions` | 0003 | Never used | 0009_cleanup.sql |
