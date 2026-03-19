// POST /api/registry/packages/:path/install
//
// Public endpoint — no auth required.
// Increments the install counter for a package.
// Fire-and-forget from CLI or web client.

import { createFileRoute } from '@tanstack/react-router'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { getD1 } from '#/lib/d1'

export const Route = createFileRoute('/api/registry/$path/install')({
  server: {
    handlers: {
      POST: async ({ request, params }) => {
        const d1 = getD1()
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

        const installs = await repos.incrementInstalls(pkg.id)
        return Response.json({ installs })
      },
    },
  },
})
