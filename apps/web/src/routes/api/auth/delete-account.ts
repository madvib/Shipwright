// POST /api/auth/delete-account
//
// Authenticated endpoint — permanently deletes the user's account.
// Removes: profiles, libraries, workflows, Better Auth records (session, account, user).

import { createFileRoute } from '@tanstack/react-router'
import { requireSession } from '#/lib/session-auth'
import { getD1 } from '#/lib/d1'
import { checkRateLimit, rateLimitResponse } from '#/lib/rate-limit'

export const Route = createFileRoute('/api/auth/delete-account')({
  server: {
    handlers: {
      POST: async ({ request }) => {
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

        const userId = auth.sub

        // Delete user data tables (order: leaves first, then auth tables)
        await d1.batch([
          d1.prepare('DELETE FROM profiles WHERE user_id = ?').bind(userId),
          d1.prepare('DELETE FROM libraries WHERE user_id = ?').bind(userId),
          d1.prepare('DELETE FROM workflows WHERE user_id = ?').bind(userId),
          d1.prepare('DELETE FROM session WHERE userId = ?').bind(userId),
          d1.prepare('DELETE FROM account WHERE userId = ?').bind(userId),
          d1.prepare('DELETE FROM user WHERE id = ?').bind(userId),
        ])

        return Response.json({ ok: true })
      },
    },
  },
})
