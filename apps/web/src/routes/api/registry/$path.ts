// GET /api/registry/packages/:path
//
// Public endpoint — no auth required.
// Path param is URL-encoded package path (e.g. 'github.com%2Fowner%2Frepo').
// Returns full package detail with versions and skills.

import { createFileRoute } from '@tanstack/react-router'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { getRegistryDb } from '#/lib/d1'

export const Route = createFileRoute('/api/registry/$path')({
  server: {
    handlers: {
      GET: async ({ params }) => {
        const d1 = getRegistryDb()
        if (!d1)
          return Response.json(
            { error: 'Database unavailable' },
            { status: 503 },
          )

        const packagePath = decodeURIComponent(params.path)
        if (!packagePath) {
          return Response.json(
            { error: 'Missing package path' },
            { status: 400 },
          )
        }

        const repos = createRegistryRepositories(d1)
        const pkg = await repos.getPackage(packagePath)
        if (!pkg) {
          return Response.json(
            { error: `Package not found: ${packagePath}` },
            { status: 404 },
          )
        }

        const [versions, skills] = await Promise.all([
          repos.getPackageVersions(pkg.id),
          repos.getPackageSkills(pkg.id),
        ])

        return Response.json({
          package: pkg,
          versions: versions.map((v) => ({
            id: v.id,
            version: v.version,
            gitTag: v.gitTag,
            commitSha: v.commitSha,
            skills: v.skillsJson ? JSON.parse(v.skillsJson) : [],
            agents: v.agentsJson ? JSON.parse(v.agentsJson) : [],
            indexedAt: v.indexedAt,
          })),
          skills: skills.map((s) => ({
            id: s.id,
            skillId: s.skillId,
            name: s.name,
            description: s.description,
            contentHash: s.contentHash,
          })),
        })
      },
    },
  },
})
