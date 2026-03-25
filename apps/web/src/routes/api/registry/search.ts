// GET /api/registry/search?q=<query>&scope=<scope>&sort=<sort>&page=<n>&limit=<n>
//
// Public endpoint — no auth required.
// Searches packages by name, description, and path.
// sort: 'installs' (default), 'recent' (by indexed_at DESC), 'name' (alphabetical)

import { createFileRoute } from '@tanstack/react-router'
import { createRegistryRepositories } from '#/db/registry-repositories'
import { getRegistryDb } from '#/lib/d1'
import { checkRateLimit, rateLimitResponse } from '#/lib/rate-limit'

const MAX_LIMIT = 100
const DEFAULT_LIMIT = 20

const VALID_SCOPES = new Set(['official', 'unofficial', 'community'])
const VALID_SORTS = new Set(['installs', 'recent', 'name'])

export const Route = createFileRoute('/api/registry/search')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const rl = await checkRateLimit(request, 'RATE_LIMITER_SEARCH', 60)
        if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

        const d1 = getRegistryDb()
        if (!d1)
          return Response.json(
            { error: 'Database unavailable' },
            { status: 503 },
          )

        const url = new URL(request.url)
        const rawQuery = url.searchParams.get('q')

        if (rawQuery !== null && rawQuery.length > 200) {
          return Response.json(
            { error: 'Query too long (max 200 characters)' },
            { status: 400 },
          )
        }

        const query = rawQuery || undefined
        const scope = url.searchParams.get('scope') || undefined
        const sort = url.searchParams.get('sort') || undefined

        if (scope && !VALID_SCOPES.has(scope)) {
          return Response.json(
            { error: `Invalid scope: ${scope}. Must be official, unofficial, or community` },
            { status: 400 },
          )
        }

        if (sort && !VALID_SORTS.has(sort)) {
          return Response.json(
            { error: `Invalid sort: ${sort}. Must be installs, recent, or name` },
            { status: 400 },
          )
        }

        const pageParam = parseInt(url.searchParams.get('page') || '1', 10)
        const limitParam = parseInt(
          url.searchParams.get('limit') || String(DEFAULT_LIMIT),
          10,
        )

        const page = Number.isFinite(pageParam) && pageParam > 0 ? pageParam : 1
        const limit =
          Number.isFinite(limitParam) && limitParam > 0
            ? Math.min(limitParam, MAX_LIMIT)
            : DEFAULT_LIMIT

        const repos = createRegistryRepositories(d1)
        const sortOrder = (sort ?? 'installs') as 'installs' | 'recent' | 'name'
        const result = await repos.searchPackages(query, scope, page, limit, sortOrder)

        return Response.json({
          packages: result.packages.map((p) => ({
            id: p.id,
            path: p.path,
            name: p.name,
            description: p.description,
            scope: p.scope,
            latestVersion: p.latestVersion,
            updatedAt: p.updatedAt,
            installs: p.installs,
            stars: p.stars,
            deprecatedBy: p.deprecatedBy,
            repoUrl: p.repoUrl,
            claimedBy: p.claimedBy,
          })),
          total: result.total,
          page: result.page,
        })
      },
    },
  },
})
