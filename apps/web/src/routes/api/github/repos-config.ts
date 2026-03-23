// GET /api/github/repos-config?owner=xxx&repo=yyy
//
// Fetches .ship/ config from a connected GitHub repo and returns a ProjectLibrary.
// Uses the authenticated user's GitHub token from the cookie.

import { createFileRoute } from '@tanstack/react-router'
import { getGitHubToken } from '#/lib/github-token'
import { fetchRepoFiles } from '#/lib/fetch-repo-files'
import { extractLibrary } from '#/lib/github-import'

export const Route = createFileRoute('/api/github/repos-config')({
  server: {
    handlers: {
      GET: async ({ request }) => {
        const token = await getGitHubToken(request)
        if (!token) {
          return Response.json({ error: 'Not authenticated' }, { status: 401 })
        }

        const url = new URL(request.url)
        const owner = url.searchParams.get('owner')
        const repo = url.searchParams.get('repo')

        if (!owner || !repo) {
          return Response.json(
            { error: 'Missing owner or repo query parameter' },
            { status: 400 },
          )
        }

        try {
          const files = await fetchRepoFiles(owner, repo, token)

          if (files === 'not_found') {
            return Response.json(
              { error: 'Repository not found or no access' },
              { status: 404 },
            )
          }

          const library = extractLibrary(files)

          if (!library) {
            return Response.json(
              { error: 'No agent config found in this repository', files_checked: Object.keys(files) },
              { status: 422 },
            )
          }

          return Response.json(library)
        } catch (err) {
          const msg = err instanceof Error ? err.message : 'Failed to fetch repo config'
          return Response.json({ error: msg }, { status: 502 })
        }
      },
    },
  },
})
