// POST /api/registry/:path/deprecate
//
// Authenticated endpoint — marks a package as deprecated.
// Only the package owner (claimedBy) can deprecate.

import { createFileRoute } from '@tanstack/react-router'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { getD1 } from '#/lib/d1'
import { requireSession } from '#/lib/session-auth'
import { checkRateLimit, rateLimitResponse } from '#/lib/rate-limit'

interface DeprecateBody {
  deprecated_by: string
}

export const Route = createFileRoute('/api/registry/$path/deprecate')({
  server: {
    handlers: {
      POST: async ({ request, params }) => {
        const rl = await checkRateLimit(request, 'RATE_LIMITER_CLAIM', 3600)
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

        let body: DeprecateBody
        try {
          body = (await request.json()) as DeprecateBody
        } catch {
          return Response.json(
            { error: 'Invalid JSON body' },
            { status: 400 },
          )
        }

        if (!body.deprecated_by || typeof body.deprecated_by !== 'string') {
          return Response.json(
            { error: 'Missing deprecated_by field' },
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

        if (pkg.claimedBy !== auth.sub) {
          return Response.json(
            { error: 'Only the package owner can deprecate this package' },
            { status: 403 },
          )
        }

        await repos.deprecatePackage(pkg.id, body.deprecated_by)
        return Response.json({
          ok: true,
          deprecated_by: body.deprecated_by,
        })
      },
    },
  },
})
