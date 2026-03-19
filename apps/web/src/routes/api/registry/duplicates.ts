// GET /api/registry/duplicates?hash=<content_hash>
// Returns all packages containing a skill with the given content hash.
// Public endpoint — no auth required.

import { createFileRoute } from '@tanstack/react-router'
import { getD1 } from '#/lib/d1'
import { createRegistryRepositories } from '#/db/registry-repositories'

export const Route = createFileRoute('/api/registry/duplicates')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const url = new URL(request.url)
        const hash = url.searchParams.get('hash')

        if (!hash) {
          return Response.json({ error: 'Missing hash query parameter' }, { status: 400 })
        }

        // Validate hash format: "sha256:<64 hex chars>"
        if (!/^sha256:[a-f0-9]{64}$/.test(hash)) {
          return Response.json(
            { error: 'Invalid hash format. Expected sha256:<64 hex chars>' },
            { status: 400 },
          )
        }

        const d1 = getD1()
        if (!d1) return Response.json({ error: 'Database unavailable' }, { status: 503 })

        const repos = createRegistryRepositories(d1)
        // Search for skills with matching content hash across all packages
        // TODO: Add getSkillDuplicates to RegistryRepositories interface
        const stmt = d1.prepare(
          `SELECT p.path as package_path, p.name as package_name, ps.name as skill_name
           FROM package_skills ps JOIN packages p ON ps.package_id = p.id
           WHERE ps.content_hash = ?`
        )
        const { results } = await stmt.bind(hash).all<{ package_path: string; package_name: string; skill_name: string }>()

        return Response.json({
          hash,
          packages: (results ?? []).map((r) => ({
            path: r.package_path,
            name: r.package_name,
            skill_name: r.skill_name,
          })),
        })
      },
    },
  },
})
