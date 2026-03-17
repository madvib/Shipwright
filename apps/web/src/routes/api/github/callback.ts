import { createFileRoute } from '@tanstack/react-router'
import {
  exchangeCodeForToken,
  getStateFromCookie,
  setTokenCookie,
  clearStateCookie,
} from '#/lib/github-app'

export const Route = createFileRoute('/api/github/callback')({
  server: {
    handlers: {
      /** Handle GitHub OAuth callback — exchange code for token, set cookie, redirect. */
      GET: async ({ request }) => {
        const url = new URL(request.url)
        const code = url.searchParams.get('code')
        const state = url.searchParams.get('state')
        const error = url.searchParams.get('error')

        if (error) {
          const desc = url.searchParams.get('error_description') ?? error
          return new Response(null, {
            status: 302,
            headers: { Location: `/studio?gh_error=${encodeURIComponent(desc)}` },
          })
        }

        if (!code || !state) {
          return new Response(null, {
            status: 302,
            headers: { Location: '/studio?gh_error=missing_params' },
          })
        }

        // Verify CSRF state
        const savedState = getStateFromCookie(request)
        if (!savedState || savedState !== state) {
          return new Response(null, {
            status: 302,
            headers: { Location: '/studio?gh_error=state_mismatch' },
          })
        }

        const clientId = process.env.GITHUB_APP_CLIENT_ID
        const clientSecret = process.env.GITHUB_APP_CLIENT_SECRET
        if (!clientId || !clientSecret) {
          return new Response(null, {
            status: 302,
            headers: { Location: '/studio?gh_error=not_configured' },
          })
        }

        try {
          const token = await exchangeCodeForToken(code, clientId, clientSecret)

          const headers = new Headers()
          headers.set('Location', '/studio?gh_connected=1')
          headers.append('Set-Cookie', setTokenCookie(token.access_token))
          headers.append('Set-Cookie', clearStateCookie())
          return new Response(null, { status: 302, headers })
        } catch (err) {
          const msg = err instanceof Error ? err.message : 'token_exchange_failed'
          return new Response(null, {
            status: 302,
            headers: {
              Location: `/studio?gh_error=${encodeURIComponent(msg)}`,
              'Set-Cookie': clearStateCookie(),
            },
          })
        }
      },
    },
  },
})
