// GET /auth/cli-callback — GitHub OAuth callback for CLI login
// Exchanges GitHub code for user, upserts to D1, issues a short-lived auth code,
// then redirects to the CLI's local callback server.

import { createFileRoute } from '@tanstack/react-router'
import { getD1, nanoid } from '#/lib/d1'

type GitHubUser = {
  id: number
  login: string
  name: string | null
  email: string | null
  avatar_url: string
}

function getEnv(key: string): string | null {
  return (
    ((globalThis as Record<string, unknown>)[key] as string | undefined) ??
    process.env[key] ??
    null
  )
}

async function exchangeGithubCode(
  code: string,
  clientId: string,
  clientSecret: string,
): Promise<GitHubUser | null> {
  const tokenRes = await fetch('https://github.com/login/oauth/access_token', {
    method: 'POST',
    headers: { Accept: 'application/json', 'Content-Type': 'application/json' },
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

function errorRedirect(msg: string) {
  return new Response(null, {
    status: 302,
    headers: { Location: `/studio?cli_error=${encodeURIComponent(msg)}` },
  })
}

export const Route = createFileRoute('/auth/cli-callback')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const url = new URL(request.url)
        const code = url.searchParams.get('code')
        const state = url.searchParams.get('state')
        const error = url.searchParams.get('error')

        if (error) return errorRedirect(error)
        if (!code || !state) return errorRedirect('missing_params')

        const db = getD1()
        if (!db) return errorRedirect('db_unavailable')

        const pending = await db
          .prepare(
            'SELECT code_challenge, redirect_uri, created_at FROM cli_auth_state WHERE state = ?',
          )
          .bind(state)
          .first<{ code_challenge: string; redirect_uri: string; created_at: number }>()

        if (!pending) return errorRedirect('invalid_state')

        // Delete state immediately — single use
        await db.prepare('DELETE FROM cli_auth_state WHERE state = ?').bind(state).run()

        if (Date.now() - pending.created_at > 10 * 60 * 1000) {
          return errorRedirect('state_expired')
        }

        const clientId = getEnv('GITHUB_APP_CLIENT_ID')
        const clientSecret = getEnv('GITHUB_APP_CLIENT_SECRET')
        if (!clientId || !clientSecret) return errorRedirect('not_configured')

        const ghUser = await exchangeGithubCode(code, clientId, clientSecret)
        if (!ghUser) return errorRedirect('github_exchange_failed')

        const now = Date.now()
        const githubEmail = ghUser.email ?? `${ghUser.login}@users.noreply.github.com`
        const userName = ghUser.name ?? ghUser.login

        // Upsert user
        const existingUser = await db
          .prepare('SELECT id FROM user WHERE email = ?')
          .bind(githubEmail)
          .first<{ id: string }>()

        const userId = existingUser?.id ?? nanoid()
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

        // Find or create personal org
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

        const existingMember = await db
          .prepare('SELECT id FROM org_members WHERE org_id = ? AND user_id = ?')
          .bind(orgId, userId)
          .first<{ id: string }>()

        if (!existingMember) {
          await db
            .prepare(
              'INSERT INTO org_members (id, org_id, user_id, role, created_at) VALUES (?, ?, ?, ?, ?)',
            )
            .bind(nanoid(), orgId, userId, 'owner', now)
            .run()
        }

        // Issue short-lived auth code (5 min TTL)
        const authCode = nanoid()
        await db
          .prepare(
            'INSERT INTO cli_auth_codes (code, user_id, org_id, code_challenge, created_at, used) VALUES (?, ?, ?, ?, ?, 0)',
          )
          .bind(authCode, userId, orgId, pending.code_challenge, now)
          .run()

        // Redirect CLI to its local callback server with the auth code
        const cliRedirect = new URL(pending.redirect_uri)
        cliRedirect.searchParams.set('code', authCode)
        return new Response(null, {
          status: 302,
          headers: { Location: cliRedirect.toString() },
        })
      },
    },
  },
})
