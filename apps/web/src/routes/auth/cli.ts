// GET /auth/cli — initiates GitHub OAuth for the CLI (PKCE flow)
// Accepts code_challenge + redirect_uri, stores state in D1, redirects to GitHub.

import { createFileRoute } from '@tanstack/react-router'
import { getD1, nanoid } from '#/lib/d1'

function getEnv(key: string): string | null {
  return (
    ((globalThis as Record<string, unknown>)[key] as string | undefined) ??
    process.env[key] ??
    null
  )
}

export const Route = createFileRoute('/auth/cli')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const url = new URL(request.url)
        const codeChallenge = url.searchParams.get('code_challenge')
        const redirectUri = url.searchParams.get('redirect_uri')

        if (!codeChallenge || !redirectUri) {
          return Response.json(
            { error: 'Missing code_challenge or redirect_uri' },
            { status: 400 },
          )
        }

        // Only allow localhost redirect URIs (CLI callback server)
        let parsed: URL
        try {
          parsed = new URL(redirectUri)
        } catch {
          return Response.json({ error: 'Invalid redirect_uri' }, { status: 400 })
        }
        if (parsed.hostname !== 'localhost' && parsed.hostname !== '127.0.0.1') {
          return Response.json(
            { error: 'redirect_uri must be localhost' },
            { status: 400 },
          )
        }

        const clientId = getEnv('GITHUB_CLIENT_ID')
        if (!clientId) {
          return Response.json({ error: 'GitHub OAuth not configured' }, { status: 500 })
        }

        const db = getD1()
        if (!db) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        const state = nanoid()
        await db
          .prepare(
            'INSERT INTO cli_auth_state (state, code_challenge, redirect_uri, created_at) VALUES (?, ?, ?, ?)',
          )
          .bind(state, codeChallenge, redirectUri, Date.now())
          .run()

        const callbackUrl = `${url.origin}/auth/cli-callback`
        const ghUrl = new URL('https://github.com/login/oauth/authorize')
        ghUrl.searchParams.set('client_id', clientId)
        ghUrl.searchParams.set('redirect_uri', callbackUrl)
        ghUrl.searchParams.set('scope', 'read:user user:email')
        ghUrl.searchParams.set('state', state)

        return new Response(null, {
          status: 302,
          headers: { Location: ghUrl.toString() },
        })
      },
    },
  },
})
