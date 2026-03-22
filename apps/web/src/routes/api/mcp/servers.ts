// GET /api/mcp/servers?q=<query>&vetted=<bool>&limit=<n>
//
// Public endpoint — no auth required.
// Proxies the official MCP registry with D1-backed caching.
// Falls back to curated list when registry and cache are both unavailable.

import { createFileRoute } from '@tanstack/react-router'
import { getRegistryDb } from '#/lib/d1'
import { fetchMcpServers } from '#/lib/mcp-registry'
import { checkRateLimit, rateLimitResponse } from '#/lib/rate-limit'

const MAX_LIMIT = 100
const DEFAULT_LIMIT = 20

export const Route = createFileRoute('/api/mcp/servers')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const rl = await checkRateLimit(request, 'RATE_LIMITER_SEARCH', 60)
        if (!rl.allowed) return rateLimitResponse(rl.retryAfter)

        const url = new URL(request.url)
        const rawQuery = url.searchParams.get('q')

        if (rawQuery !== null && rawQuery.length > 200) {
          return Response.json(
            { error: 'Query too long (max 200 characters)' },
            { status: 400 },
          )
        }

        const query = rawQuery || undefined
        const vettedParam = url.searchParams.get('vetted')
        const vetted = vettedParam === null ? true : vettedParam !== 'false'

        const limitParam = parseInt(
          url.searchParams.get('limit') || String(DEFAULT_LIMIT),
          10,
        )
        const limit =
          Number.isFinite(limitParam) && limitParam > 0
            ? Math.min(limitParam, MAX_LIMIT)
            : DEFAULT_LIMIT

        const db = getRegistryDb()
        const result = await fetchMcpServers(db, query, vetted)

        return Response.json({
          servers: result.servers.slice(0, limit),
          cached: result.cached,
        })
      },
    },
  },
})
