import { createFileRoute } from '@tanstack/react-router'
import { buildAuthorizeUrl, setStateCookie } from '#/lib/github-app'

export const Route = createFileRoute('/api/github/oauth')({
  server: {
    handlers: {
      /** Redirect to GitHub OAuth authorization page. */
      GET: ({ request }) => {
        const clientId = process.env.GITHUB_CLIENT_ID
        if (!clientId) {
          return Response.json({ error: 'GitHub App not configured' }, { status: 500 })
        }

        const url = new URL(request.url)
        const redirectUri = `${url.origin}/api/github/callback`

        // Generate CSRF state token
        const state = crypto.randomUUID()
        const authorizeUrl = buildAuthorizeUrl(clientId, redirectUri, state)

        return new Response(null, {
          status: 302,
          headers: {
            Location: authorizeUrl,
            'Set-Cookie': setStateCookie(state),
          },
        })
      },
    },
  },
})
