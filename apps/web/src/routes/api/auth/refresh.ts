// POST /api/auth/refresh
// Header: Authorization: Bearer <token>
// Returns: { token: string }
//
// Accepts a valid or recently-expired JWT and re-issues a fresh one.
// Grace window: tokens issued within the last 30 days may be refreshed
// even after expiry, provided the user still exists in D1.

import { createFileRoute } from '@tanstack/react-router'
import { signJwt, verifyJwt, getSecret, type JwtPayload } from '#/lib/cloud-auth'
import { getAuthDb } from '#/lib/d1'

function base64urlDecode(input: string): Uint8Array {
  const padded = input
    .replace(/-/g, '+')
    .replace(/_/g, '/')
    .padEnd(input.length + ((4 - (input.length % 4)) % 4), '=')
  const binary = atob(padded)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i)
  return bytes
}

/** Parse a JWT payload without verifying the signature. */
function decodeJwtPayload(token: string): JwtPayload | null {
  const parts = token.split('.')
  if (parts.length !== 3) return null
  try {
    const raw = new TextDecoder().decode(base64urlDecode(parts[1]))
    return JSON.parse(raw) as JwtPayload
  } catch {
    return null
  }
}

const GRACE_WINDOW_SECONDS = 30 * 24 * 60 * 60 // 30 days

export const Route = createFileRoute('/api/auth/refresh')({
  server: {
    handlers: {
      POST: async ({ request }) => {
        const authHeader = request.headers.get('Authorization')
        if (!authHeader?.startsWith('Bearer ')) {
          return Response.json({ error: 'Missing Authorization header' }, { status: 401 })
        }
        const token = authHeader.slice(7)

        const secret = getSecret()
        if (!secret) {
          return Response.json(
            { error: 'Server misconfiguration: missing secret' },
            { status: 500 },
          )
        }

        const db = getAuthDb()
        if (!db) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        // Try normal verification first (covers non-expired tokens)
        let payload = await verifyJwt(token, secret)

        if (!payload) {
          // Token invalid or expired — decode without verification to check grace window
          const raw = decodeJwtPayload(token)
          if (!raw?.sub || !raw?.org || typeof raw.iat !== 'number') {
            return Response.json({ error: 'Invalid token' }, { status: 401 })
          }

          const now = Math.floor(Date.now() / 1000)
          if (now - raw.iat > GRACE_WINDOW_SECONDS) {
            return Response.json(
              { error: 'Session expired, please log in again' },
              { status: 401 },
            )
          }

          payload = raw
        }

        // Confirm user still exists
        const user = await db
          .prepare('SELECT id FROM user WHERE id = ?')
          .bind(payload.sub)
          .first<{ id: string }>()

        if (!user) {
          return Response.json({ error: 'User not found' }, { status: 401 })
        }

        const newToken = await signJwt({ sub: payload.sub, org: payload.org }, secret)
        return Response.json({ token: newToken })
      },
    },
  },
})
