import { drizzle } from 'drizzle-orm/d1'
import { and, asc, desc, eq, like, or, sql } from 'drizzle-orm'

import {
  packages,
  packageVersions,
  packageSkills,
  type InsertPackage,
  type InsertPackageVersion,
  type InsertPackageSkill,
  type Package,
  type PackageVersion,
  type PackageSkill,
} from './registry-schema'

// ---------------------------------------------------------------------------
// Registry repository interface
// ---------------------------------------------------------------------------

export type SortOrder = 'installs' | 'recent' | 'name'

export interface SearchResult {
  packages: Package[]
  total: number
  page: number
}

export interface RegistryRepositories {
  searchPackages(
    query: string | undefined,
    scope: string | undefined,
    page: number,
    limit: number,
    sort?: SortOrder,
  ): Promise<SearchResult>

  getPackage(path: string): Promise<Package | null>

  upsertPackage(data: InsertPackage): Promise<Package>

  getLatestVersion(packageId: string): Promise<PackageVersion | null>

  getPackageVersions(packageId: string): Promise<PackageVersion[]>

  getPackageSkills(
    packageId: string,
    versionId?: string,
  ): Promise<PackageSkill[]>

  incrementInstalls(packageId: string): Promise<number>

  createPackageVersion(data: InsertPackageVersion): Promise<PackageVersion>

  createPackageSkill(data: InsertPackageSkill): Promise<PackageSkill>

  deletePackageSkillsByVersion(versionId: string): Promise<void>

  incrementStars(packageId: string): Promise<number>

  deprecatePackage(packageId: string, deprecatedBy: string): Promise<void>

  claimPackage(path: string, userId: string, newScope: string): Promise<boolean>

  updatePackageVersionHash(versionId: string, contentHash: string): Promise<void>
}

export function createRegistryRepositories(
  d1: D1Database,
): RegistryRepositories {
  const db = drizzle(d1)

  return {
    async searchPackages(query, scope, page, limit, sort = 'installs' as SortOrder) {
      const conditions = []
      if (scope) {
        conditions.push(eq(packages.scope, scope))
      }
      if (query) {
        const pattern = `%${query}%`
        conditions.push(
          or(
            like(packages.name, pattern),
            like(packages.description, pattern),
            like(packages.path, pattern),
          )!,
        )
      }

      const where = conditions.length > 0 ? and(...conditions) : undefined
      const offset = (page - 1) * limit

      const orderBy =
        sort === 'recent'
          ? desc(packages.indexedAt)
          : sort === 'name'
            ? asc(packages.name)
            : desc(packages.installs)

      const [rows, countResult] = await Promise.all([
        db
          .select()
          .from(packages)
          .where(where)
          .orderBy(orderBy)
          .limit(limit)
          .offset(offset)
          .all(),
        db
          .select({ count: sql<number>`count(*)` })
          .from(packages)
          .where(where)
          .get(),
      ])

      return {
        packages: rows,
        total: countResult?.count ?? 0,
        page,
      }
    },

    async getPackage(path) {
      const row = await db
        .select()
        .from(packages)
        .where(eq(packages.path, path))
        .get()
      return row ?? null
    },

    async upsertPackage(data) {
      await db
        .insert(packages)
        .values(data)
        .onConflictDoUpdate({
          target: packages.path,
          set: {
            name: data.name,
            description: data.description,
            latestVersion: data.latestVersion,
            contentHash: data.contentHash,
            defaultBranch: data.defaultBranch,
            scope: data.scope,
            claimedBy: data.claimedBy,
            updatedAt: data.updatedAt,
          },
        })
      const row = await db
        .select()
        .from(packages)
        .where(eq(packages.path, data.path))
        .get()
      if (!row)
        throw new Error(
          `upsertPackage: row not found after write (path=${data.path})`,
        )
      return row
    },

    async getLatestVersion(packageId) {
      const row = await db
        .select()
        .from(packageVersions)
        .where(eq(packageVersions.packageId, packageId))
        .orderBy(desc(packageVersions.indexedAt))
        .limit(1)
        .get()
      return row ?? null
    },

    async getPackageVersions(packageId) {
      return db
        .select()
        .from(packageVersions)
        .where(eq(packageVersions.packageId, packageId))
        .orderBy(desc(packageVersions.indexedAt))
        .all()
    },

    async getPackageSkills(packageId, versionId) {
      const conditions = [eq(packageSkills.packageId, packageId)]
      if (versionId) {
        conditions.push(eq(packageSkills.versionId, versionId))
      }
      return db
        .select()
        .from(packageSkills)
        .where(and(...conditions))
        .all()
    },

    async incrementInstalls(packageId) {
      await db
        .update(packages)
        .set({ installs: sql`${packages.installs} + 1` })
        .where(eq(packages.id, packageId))
      const row = await db
        .select({ installs: packages.installs })
        .from(packages)
        .where(eq(packages.id, packageId))
        .get()
      return row?.installs ?? 0
    },

    async createPackageVersion(data) {
      await db.insert(packageVersions).values(data)
      const row = await db
        .select()
        .from(packageVersions)
        .where(eq(packageVersions.id, data.id))
        .get()
      if (!row)
        throw new Error(
          `createPackageVersion: row not found after write (id=${data.id})`,
        )
      return row
    },

    async createPackageSkill(data) {
      await db.insert(packageSkills).values(data)
      const row = await db
        .select()
        .from(packageSkills)
        .where(eq(packageSkills.id, data.id))
        .get()
      if (!row)
        throw new Error(
          `createPackageSkill: row not found after write (id=${data.id})`,
        )
      return row
    },

    async deletePackageSkillsByVersion(versionId) {
      await db
        .delete(packageSkills)
        .where(eq(packageSkills.versionId, versionId))
    },

    async incrementStars(packageId) {
      await db
        .update(packages)
        .set({ stars: sql`${packages.stars} + 1` })
        .where(eq(packages.id, packageId))
      const row = await db
        .select({ stars: packages.stars })
        .from(packages)
        .where(eq(packages.id, packageId))
        .get()
      return row?.stars ?? 0
    },

    async deprecatePackage(packageId, deprecatedBy) {
      await db
        .update(packages)
        .set({ deprecatedBy, updatedAt: Date.now() })
        .where(eq(packages.id, packageId))
    },

    async updatePackageVersionHash(versionId, contentHash) {
      await db
        .update(packageVersions)
        .set({ contentHash })
        .where(eq(packageVersions.id, versionId))
    },

    async claimPackage(path, userId, newScope) {
      const result = await db
        .update(packages)
        .set({ claimedBy: userId, scope: newScope, updatedAt: Date.now() })
        .where(
          and(
            eq(packages.path, path),
            or(sql`${packages.claimedBy} IS NULL`, eq(packages.claimedBy, '')),
          ),
        )
      const rows = (result as unknown as { rowsAffected: number }).rowsAffected
      return rows > 0
    },
  }
}
