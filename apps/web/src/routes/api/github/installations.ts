// GET /api/github/installations
//
// Returns all GitHub App installations and their connected repos.
// Used by the import page to show repos where the user has installed the Ship app.

import { createFileRoute } from '@tanstack/react-router'
import { getRegistryDb } from '#/lib/d1'
import { requireSession } from '#/lib/session-auth'

interface InstallationRepo {
  id: number
  full_name: string
  private: boolean
}

interface InstallationRow {
  installation_id: number
  account_login: string
  account_type: string
  repos_json: string
}

export const Route = createFileRoute('/api/github/installations')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const sessionResult = await requireSession(request)
        if (sessionResult instanceof Response) return sessionResult

        const d1 = getRegistryDb()
        if (!d1) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        try {
          // Filter to installations where account_login matches one of the user's org slugs
          const rows = await d1
            .prepare(
              `SELECT gi.installation_id, gi.account_login, gi.account_type, gi.repos_json
               FROM github_installations gi
               INNER JOIN orgs o ON LOWER(gi.account_login) = o.slug
               INNER JOIN org_members om ON om.org_id = o.id AND om.user_id = ?
               ORDER BY gi.updated_at DESC`,
            )
            .bind(sessionResult.sub)
            .all<InstallationRow>()

          const installations = (rows.results ?? []).map((row: InstallationRow) => ({
            installation_id: row.installation_id,
            account_login: row.account_login,
            account_type: row.account_type,
            repos: JSON.parse(row.repos_json) as InstallationRepo[],
          }))

          return Response.json({ installations })
        } catch {
          return Response.json({ installations: [] })
        }
      },
    },
  },
})
