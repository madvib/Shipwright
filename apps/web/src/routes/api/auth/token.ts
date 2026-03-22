// POST /api/auth/token
// CLI exchanges a server-issued auth code + PKCE verifier for a signed JWT.
// The auth code is created by /auth/cli-callback after GitHub OAuth completes.

import { createFileRoute } from '@tanstack/react-router'
import { signJwt, getSecret } from '#/lib/cloud-auth'
import { getAuthDb } from '#/lib/d1'

async function sha256Base64url(input: string): Promise<string> {
  const data = new TextEncoder().encode(input)
  const hash = await crypto.subtle.digest('SHA-256', data)
  const bytes = new Uint8Array(hash)
  let binary = ''
  for (const b of bytes) binary += String.fromCharCode(b)
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '')
}

export const Route = createFileRoute('/api/auth/token')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        let body: unknown
        try {
          body = await request.json()
        } catch {
          return Response.json({ error: 'Invalid JSON body' }, { status: 400 })
        }

        const b = body as Record<string, unknown>
        if (!body || typeof b['code'] !== 'string' || typeof b['verifier'] !== 'string') {
          return Response.json({ error: 'Missing code or verifier' }, { status: 400 })
        }

        const { code, verifier } = b as { code: string; verifier: string }

        const db = getAuthDb()
        if (!db) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        const secret = getSecret()
        if (!secret) {
          return Response.json(
            { error: 'Server misconfiguration: missing secret' },
            { status: 500 },
          )
        }

        const row = await db
          .prepare(
            'SELECT user_id, code_challenge, created_at, used FROM cli_auth_codes WHERE code = ?',
          )
          .bind(code)
          .first<{
            user_id: string
            code_challenge: string
            created_at: number
            used: number
          }>()

        if (!row) {
          return Response.json({ error: 'Invalid auth code' }, { status: 401 })
        }
        if (row.used) {
          return Response.json({ error: 'Auth code already used' }, { status: 401 })
        }
        if (Date.now() - row.created_at > 5 * 60 * 1000) {
          return Response.json({ error: 'Auth code expired' }, { status: 401 })
        }

        // Verify PKCE: SHA256(verifier) must match the stored code_challenge (S256 method)
        const computed = await sha256Base64url(verifier)
        if (computed !== row.code_challenge) {
          return Response.json({ error: 'PKCE verification failed' }, { status: 401 })
        }

        await db.prepare('UPDATE cli_auth_codes SET used = 1 WHERE code = ?').bind(code).run()

        const token = await signJwt({ sub: row.user_id, org: row.user_id }, secret)
        return Response.json({ token })
      },
    },
  },
})
