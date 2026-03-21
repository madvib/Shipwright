import { createFileRoute } from '@tanstack/react-router'
import { clearTokenCookie } from '#/lib/github-app'

export const Route = createFileRoute('/api/github/disconnect')({
  server: {
    handlers: {
      POST: () => {
        return Response.json(
          { ok: true },
          { headers: { 'Set-Cookie': clearTokenCookie() } },
        )
      },
    },
  },
})
