// POST /api/auth/token
// Accepts a GitHub OAuth code, exchanges it for a GitHub user, creates/updates
// user + org in D1, and returns a signed JWT.

import { createFileRoute } from '@tanstack/react-router'
import { signJwt, getDb, getSecret } from '#/lib/cloud-auth'

type GitHubUser = {
  id: number
  login: string
  name: string | null
  email: string | null
  avatar_url: string
}

async function exchangeGithubCode(
  code: string,
  clientId: string,
  clientSecret: string,
): Promise<GitHubUser | null> {
  const tokenRes = await fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ client_id: clientId, client_secret: clientSecret, code }),
  })
  if (!tokenRes.ok) return null

  const tokenData = (await tokenRes.json()) as Record<string, unknown>
  const accessToken = tokenData['access_token']
  if (typeof accessToken !== 'string') return null

  const userRes = await fetch('https://api.github.com/user', {
    headers: {
      Authorization: `Bearer ${accessToken}`,
      Accept: 'application/vnd.github.v3+json',
      'User-Agent': 'ship-studio/1.0',
    },
  })
  if (!userRes.ok) return null

  return (await userRes.json()) as GitHubUser
}

function getEnv(key: string): string | null {
  return (
    ((globalThis as Record<string, unknown>)[key] as string | undefined) ??
    process.env[key] ??
    null
  )
}

function nanoid(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('')
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

        if (
          !body ||
          typeof body !== 'object' ||
          typeof (body as Record<string, unknown>)['code'] !== 'string'
        ) {
          return Response.json({ error: 'Missing code field' }, { status: 400 })
        }

        const code = (body as { code: string }).code

        const clientId = getEnv('GITHUB_CLIENT_ID')
        const clientSecret = getEnv('GITHUB_CLIENT_SECRET')
        if (!clientId || !clientSecret) {
          return Response.json({ error: 'Server misconfiguration: missing GitHub OAuth credentials' }, { status: 500 })
        }

        const ghUser = await exchangeGithubCode(code, clientId, clientSecret)
        if (!ghUser) {
          return Response.json({ error: 'GitHub OAuth exchange failed' }, { status: 401 })
        }

        const db = getDb()
        if (!db) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        const secret = getSecret()
        if (!secret) {
          return Response.json({ error: 'Server misconfiguration: missing secret' }, { status: 500 })
        }

        const now = Date.now()
        const githubEmail = ghUser.email ?? `${ghUser.login}@users.noreply.github.com`

        // Upsert user
        const existingUser = await db
          .prepare('SELECT id, name, email FROM user WHERE email = ?')
          .bind(githubEmail)
          .first<{ id: string; name: string; email: string }>()

        const userId = existingUser?.id ?? nanoid()
        const userName = ghUser.name ?? ghUser.login

        if (existingUser) {
          await db
            .prepare('UPDATE user SET name = ?, updatedAt = ? WHERE id = ?')
            .bind(userName, now, userId)
            .run()
        } else {
          await db
            .prepare(
              'INSERT INTO user (id, name, email, emailVerified, image, createdAt, updatedAt) VALUES (?, ?, ?, ?, ?, ?, ?)',
            )
            .bind(userId, userName, githubEmail, 1, ghUser.avatar_url, now, now)
            .run()
        }

        // Find or create org (one personal org per GitHub user, slug = login)
        const orgSlug = ghUser.login.toLowerCase()
        const existingOrg = await db
          .prepare('SELECT id FROM orgs WHERE slug = ?')
          .bind(orgSlug)
          .first<{ id: string }>()

        const orgId = existingOrg?.id ?? nanoid()

        if (!existingOrg) {
          await db
            .prepare('INSERT INTO orgs (id, name, slug, created_at) VALUES (?, ?, ?, ?)')
            .bind(orgId, ghUser.login, orgSlug, now)
            .run()
        }

        // Ensure org membership
        const existingMember = await db
          .prepare('SELECT id FROM org_members WHERE org_id = ? AND user_id = ?')
          .bind(orgId, userId)
          .first<{ id: string }>()

        if (!existingMember) {
          await db
            .prepare('INSERT INTO org_members (id, org_id, user_id, role, created_at) VALUES (?, ?, ?, ?, ?)')
            .bind(nanoid(), orgId, userId, 'owner', now)
            .run()
        }

        const token = await signJwt({ sub: userId, org: orgId }, secret)
        return Response.json({ token })
      },
    },
  },
})
