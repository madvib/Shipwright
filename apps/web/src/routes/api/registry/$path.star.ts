// POST /api/registry/:path/star
//
// Authenticated endpoint — increments the star counter for a package.
// Simple increment model for v0.1 (no unstar / no per-user tracking).

import { createFileRoute } from '@tanstack/react-router'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { getD1 } from '#/lib/d1'
import { requireSession } from '#/lib/session-auth'
import { checkRateLimit, rateLimitResponse } from '#/lib/rate-limit'

export const Route = createFileRoute('/api/registry/$path/star')({
  server: {
    handlers: {
      POST: async ({ request, params }) => {
        const rl = await checkRateLimit(request, 'RATE_LIMITER_STAR', 60)
        if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

        const auth = await requireSession(request)
        if (auth instanceof Response) return auth

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

        const stars = await repos.incrementStars(pkg.id)
        return Response.json({ stars, starred: true })
      },
    },
  },
})
