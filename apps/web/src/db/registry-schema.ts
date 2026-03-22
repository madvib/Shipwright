import { index, integer, sqliteTable, text } from 'drizzle-orm/sqlite-core'

// ---------------------------------------------------------------------------
// Registry tables — packages, versions, skills
// ---------------------------------------------------------------------------

export const packages = sqliteTable(
  'packages',
  {
    id: text('id').notNull().primaryKey(),
    path: text('path').notNull().unique(),
    scope: text('scope').notNull(), // 'official' | 'unofficial' | 'community'
    name: text('name').notNull(),
    description: text('description'),
    repoUrl: text('repo_url').notNull(),
    defaultBranch: text('default_branch').notNull().default('main'),
    latestVersion: text('latest_version'),
    contentHash: text('content_hash'),
    sourceType: text('source_type').notNull().default('native'), // 'native' | 'imported'
    tags: text('tags'), // JSON array
    claimedBy: text('claimed_by'), // references user.id in AUTH_DB (cross-DB, no FK)
    deprecatedBy: text('deprecated_by'),
    stars: integer('stars').notNull().default(0),
    installs: integer('installs').notNull().default(0),
    indexedAt: integer('indexed_at').notNull(),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [
    index('packages_scope_installs').on(t.scope, t.installs),
    index('packages_path').on(t.path),
    index('packages_name').on(t.name),
    index('packages_description').on(t.description),
    index('packages_claimed_by').on(t.claimedBy),
  ],
)

export type Package = typeof packages.$inferSelect
export type InsertPackage = typeof packages.$inferInsert

export const packageVersions = sqliteTable(
  'package_versions',
  {
    id: text('id').notNull().primaryKey(),
    packageId: text('package_id')
      .notNull()
      .references(() => packages.id),
    version: text('version').notNull(),
    gitTag: text('git_tag').notNull(),
    commitSha: text('commit_sha').notNull(),
    contentHash: text('content_hash'),
    skillsJson: text('skills_json'),
    agentsJson: text('agents_json'),
    indexedAt: integer('indexed_at').notNull(),
  },
  (t) => [
    index('pkg_versions_package_indexed').on(t.packageId, t.indexedAt),
  ],
)

export type PackageVersion = typeof packageVersions.$inferSelect
export type InsertPackageVersion = typeof packageVersions.$inferInsert

export const packageSkills = sqliteTable(
  'package_skills',
  {
    id: text('id').notNull().primaryKey(),
    packageId: text('package_id')
      .notNull()
      .references(() => packages.id),
    versionId: text('version_id')
      .notNull()
      .references(() => packageVersions.id),
    skillId: text('skill_id').notNull(),
    name: text('name').notNull(),
    description: text('description'),
    contentHash: text('content_hash').notNull(),
  },
  (t) => [
    index('pkg_skills_content_hash').on(t.contentHash),
    index('pkg_skills_package').on(t.packageId),
    index('pkg_skills_package_version').on(t.packageId, t.versionId),
  ],
)

export type PackageSkill = typeof packageSkills.$inferSelect
export type InsertPackageSkill = typeof packageSkills.$inferInsert

// ---------------------------------------------------------------------------
// GitHub App installations
// ---------------------------------------------------------------------------

export const githubInstallations = sqliteTable(
  'github_installations',
  {
    id: text('id').notNull().primaryKey(),
    installationId: integer('installation_id').notNull().unique(),
    accountLogin: text('account_login').notNull(),
    accountType: text('account_type').notNull(), // 'User' | 'Organization'
    reposJson: text('repos_json').notNull().default('[]'),
    createdAt: integer('created_at').notNull(),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [index('gh_install_account').on(t.accountLogin)],
)

export type GithubInstallation = typeof githubInstallations.$inferSelect
export type InsertGithubInstallation = typeof githubInstallations.$inferInsert

// ---------------------------------------------------------------------------
// MCP servers — replaces mcp_servers_cache
// ---------------------------------------------------------------------------

export const mcpServers = sqliteTable('mcp_servers', {
  name: text('name').notNull().primaryKey(),
  title: text('title'),
  description: text('description'),
  homepage: text('homepage'),
  vendor: text('vendor'),
  tags: text('tags'), // JSON array
  packageRegistry: text('package_registry'),
  command: text('command'),
  args: text('args'), // JSON array
  imageUrl: text('image_url'),
  status: text('status').notNull().default('active'), // active | deprecated | deleted
  vetted: integer('vetted').notNull().default(0),
  syncedAt: integer('synced_at'),
})

export type McpServerRow = typeof mcpServers.$inferSelect
export type InsertMcpServer = typeof mcpServers.$inferInsert
