// GET /api/github/installations
//
// Returns all GitHub App installations and their connected repos.
// Used by the import page to show repos where the user has installed the Ship app.

import { createFileRoute } from '@tanstack/react-router'
import { getD1 } from '#/lib/d1'

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
      GET: async () => {
        const d1 = getD1()
        if (!d1) {
          return Response.json({ error: 'Database unavailable' }, { status: 503 })
        }

        try {
          const rows = await d1
            .prepare(
              'SELECT installation_id, account_login, account_type, repos_json FROM github_installations ORDER BY updated_at DESC',
            )
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
