import { createFileRoute } from '@tanstack/react-router'
import { requireSession } from '#/lib/session-auth'

export const Route = createFileRoute('/api/github/disconnect')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const sessionResult = await requireSession(request)
        if (sessionResult instanceof Response) return sessionResult

        // GitHub account is linked via Better Auth social provider.
        // To fully disconnect, the user should re-authenticate.
        // The client should clear its connection state and prompt re-login.
        return Response.json({ ok: true, action: 'reauthenticate' })
      },
    },
  },
})
