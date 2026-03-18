import { drizzle } from 'drizzle-orm/d1'
import { and, eq } from 'drizzle-orm'

import {
  libraries,
  profiles,
  workflows,
  type InsertLibrary,
  type InsertProfile,
  type InsertWorkflow,
  type Library,
  type Profile,
  type Workflow,
} from './schema'

// ---------------------------------------------------------------------------
// Repository interface
//
// createRepositories accepts a D1Database but callers receive a plain object
// of typed async functions — no D1 types leak past this boundary. The backend
// can be swapped for Durable Objects / Rivet actors by replacing this factory
// without changing any call sites.
// ---------------------------------------------------------------------------

export interface Repositories {
  // Libraries
  getLibraries(orgId: string, userId: string): Promise<Library[]>
  getLibrary(id: string, orgId: string): Promise<Library | null>
  upsertLibrary(data: InsertLibrary): Promise<Library>
  deleteLibrary(id: string, orgId: string): Promise<void>

  // Profiles
  getProfiles(orgId: string, userId: string): Promise<Profile[]>
  getProfile(id: string, orgId: string): Promise<Profile | null>
  upsertProfile(data: InsertProfile): Promise<Profile>
  deleteProfile(id: string, orgId: string): Promise<void>

  // Workflows
  getWorkflows(orgId: string, userId: string): Promise<Workflow[]>
  getWorkflow(id: string, orgId: string): Promise<Workflow | null>
  upsertWorkflow(data: InsertWorkflow): Promise<Workflow>
  deleteWorkflow(id: string, orgId: string): Promise<void>
}

export function createRepositories(d1: D1Database): Repositories {
  const db = drizzle(d1)

  return {
    // -----------------------------------------------------------------------
    // Libraries
    // -----------------------------------------------------------------------

    async getLibraries(orgId, userId) {
      return db
        .select()
        .from(libraries)
        .where(and(eq(libraries.orgId, orgId), eq(libraries.userId, userId)))
        .all()
    },

    async getLibrary(id, orgId) {
      const row = await db
        .select()
        .from(libraries)
        .where(and(eq(libraries.id, id), eq(libraries.orgId, orgId)))
        .get()
      return row ?? null
    },

    async upsertLibrary(data) {
      await db
        .insert(libraries)
        .values(data)
        .onConflictDoUpdate({
          target: libraries.id,
          set: {
            name: data.name,
            slug: data.slug,
            data: data.data,
            updatedAt: data.updatedAt,
          },
        })
      const row = await db
        .select()
        .from(libraries)
        .where(eq(libraries.id, data.id))
        .get()
      if (!row) throw new Error(`upsertLibrary: row not found after write (id=${data.id})`)
      return row
    },

    async deleteLibrary(id, orgId) {
      await db
        .delete(libraries)
        .where(and(eq(libraries.id, id), eq(libraries.orgId, orgId)))
    },

    // -----------------------------------------------------------------------
    // Profiles
    // -----------------------------------------------------------------------

    async getProfiles(orgId, userId) {
      return db
        .select()
        .from(profiles)
        .where(and(eq(profiles.orgId, orgId), eq(profiles.userId, userId)))
        .all()
    },

    async getProfile(id, orgId) {
      const row = await db
        .select()
        .from(profiles)
        .where(and(eq(profiles.id, id), eq(profiles.orgId, orgId)))
        .get()
      return row ?? null
    },

    async upsertProfile(data) {
      await db
        .insert(profiles)
        .values(data)
        .onConflictDoUpdate({
          target: profiles.id,
          set: {
            name: data.name,
            content: data.content,
            provider: data.provider,
            updatedAt: data.updatedAt,
          },
        })
      const row = await db
        .select()
        .from(profiles)
        .where(eq(profiles.id, data.id))
        .get()
      if (!row) throw new Error(`upsertProfile: row not found after write (id=${data.id})`)
      return row
    },

    async deleteProfile(id, orgId) {
      await db
        .delete(profiles)
        .where(and(eq(profiles.id, id), eq(profiles.orgId, orgId)))
    },

    // -----------------------------------------------------------------------
    // Workflows
    // -----------------------------------------------------------------------

    async getWorkflows(orgId, userId) {
      return db
        .select()
        .from(workflows)
        .where(and(eq(workflows.orgId, orgId), eq(workflows.userId, userId)))
        .all()
    },

    async getWorkflow(id, orgId) {
      const row = await db
        .select()
        .from(workflows)
        .where(and(eq(workflows.id, id), eq(workflows.orgId, orgId)))
        .get()
      return row ?? null
    },

    async upsertWorkflow(data) {
      await db
        .insert(workflows)
        .values(data)
        .onConflictDoUpdate({
          target: workflows.id,
          set: {
            name: data.name,
            definition: data.definition,
            updatedAt: data.updatedAt,
          },
        })
      const row = await db
        .select()
        .from(workflows)
        .where(eq(workflows.id, data.id))
        .get()
      if (!row) throw new Error(`upsertWorkflow: row not found after write (id=${data.id})`)
      return row
    },

    async deleteWorkflow(id, orgId) {
      await db
        .delete(workflows)
        .where(and(eq(workflows.id, id), eq(workflows.orgId, orgId)))
    },
  }
}
