import { createFileRoute } from '@tanstack/react-router'
import { clearTokenCookie } from '#/lib/github-app'
import { requireSession } from '#/lib/session-auth'

export const Route = createFileRoute('/api/github/disconnect')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const sessionResult = await requireSession(request)
        if (sessionResult instanceof Response) return sessionResult

        return Response.json(
          { ok: true },
          { headers: { 'Set-Cookie': clearTokenCookie() } },
        )
      },
    },
  },
})
