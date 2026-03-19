import { index, integer, sqliteTable, text, uniqueIndex } from 'drizzle-orm/sqlite-core'

export const user = sqliteTable('user', {
  id: text('id').notNull().primaryKey(),
  name: text('name').notNull(),
  email: text('email').notNull().unique(),
  emailVerified: integer('emailVerified', { mode: 'boolean' }).notNull(),
  image: text('image'),
  createdAt: integer('createdAt', { mode: 'timestamp' }).notNull(),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }).notNull(),
})

export const session = sqliteTable('session', {
  id: text('id').notNull().primaryKey(),
  expiresAt: integer('expiresAt', { mode: 'timestamp' }).notNull(),
  token: text('token').notNull().unique(),
  createdAt: integer('createdAt', { mode: 'timestamp' }).notNull(),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }).notNull(),
  ipAddress: text('ipAddress'),
  userAgent: text('userAgent'),
  userId: text('userId')
    .notNull()
    .references(() => user.id),
})

export const account = sqliteTable('account', {
  id: text('id').notNull().primaryKey(),
  accountId: text('accountId').notNull(),
  providerId: text('providerId').notNull(),
  userId: text('userId')
    .notNull()
    .references(() => user.id),
  accessToken: text('accessToken'),
  refreshToken: text('refreshToken'),
  idToken: text('idToken'),
  accessTokenExpiresAt: integer('accessTokenExpiresAt', { mode: 'timestamp' }),
  refreshTokenExpiresAt: integer('refreshTokenExpiresAt', { mode: 'timestamp' }),
  scope: text('scope'),
  password: text('password'),
  createdAt: integer('createdAt', { mode: 'timestamp' }).notNull(),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }).notNull(),
})

export const verification = sqliteTable('verification', {
  id: text('id').notNull().primaryKey(),
  identifier: text('identifier').notNull(),
  value: text('value').notNull(),
  expiresAt: integer('expiresAt', { mode: 'timestamp' }).notNull(),
  createdAt: integer('createdAt', { mode: 'timestamp' }),
  updatedAt: integer('updatedAt', { mode: 'timestamp' }),
})

// ---------------------------------------------------------------------------
// User data tables
// ---------------------------------------------------------------------------

export const libraries = sqliteTable(
  'libraries',
  {
    id: text('id').notNull().primaryKey(),
    orgId: text('org_id').notNull(),
    userId: text('user_id').notNull(),
    name: text('name').notNull(),
    slug: text('slug'),
    data: text('data').notNull().default('{}'),
    createdAt: integer('created_at').notNull(),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [
    index('libraries_org_user').on(t.orgId, t.userId),
    uniqueIndex('libraries_org_slug').on(t.orgId, t.slug),
  ],
)

export type Library = typeof libraries.$inferSelect
export type InsertLibrary = typeof libraries.$inferInsert

export const profiles = sqliteTable(
  'profiles',
  {
    id: text('id').notNull().primaryKey(),
    orgId: text('org_id').notNull(),
    userId: text('user_id').notNull(),
    name: text('name').notNull(),
    content: text('content').notNull(),
    provider: text('provider'),
    createdAt: integer('created_at').notNull(),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [index('profiles_org_user').on(t.orgId, t.userId)],
)

export type Profile = typeof profiles.$inferSelect
export type InsertProfile = typeof profiles.$inferInsert

export const workflows = sqliteTable(
  'workflows',
  {
    id: text('id').notNull().primaryKey(),
    orgId: text('org_id').notNull(),
    userId: text('user_id').notNull(),
    name: text('name').notNull(),
    definition: text('definition').notNull().default('{}'),
    createdAt: integer('created_at').notNull(),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [index('workflows_org_user').on(t.orgId, t.userId)],
)

export type Workflow = typeof workflows.$inferSelect
export type InsertWorkflow = typeof workflows.$inferInsert

// ---------------------------------------------------------------------------
// Registry tables
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
    claimedBy: text('claimed_by').references(() => user.id),
    deprecatedBy: text('deprecated_by'),
    stars: integer('stars').notNull().default(0),
    installs: integer('installs').notNull().default(0),
    indexedAt: integer('indexed_at').notNull(),
    updatedAt: integer('updated_at').notNull(),
  },
  (t) => [
    index('packages_scope_installs').on(t.scope, t.installs),
    index('packages_path').on(t.path),
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
    contentLength: integer('content_length').notNull().default(0),
  },
  (t) => [
    index('pkg_skills_content_hash').on(t.contentHash),
    index('pkg_skills_package').on(t.packageId),
  ],
)

export type PackageSkill = typeof packageSkills.$inferSelect
export type InsertPackageSkill = typeof packageSkills.$inferInsert
