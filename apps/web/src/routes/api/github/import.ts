import { createFileRoute } from '@tanstack/react-router'
import { parseGithubUrl, extractLibrary } from '#/lib/github-import'
import { fetchRepoFiles } from '#/lib/fetch-repo-files'

export const Route = createFileRoute('/api/github/import')({
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
          !('url' in body) ||
          typeof (body as Record<string, unknown>).url !== 'string'
        ) {
          return Response.json({ error: 'Missing or invalid url field' }, { status: 400 })
        }

        const url = (body as { url: string }).url
        const parsed = parseGithubUrl(url)
        if (!parsed) {
          return Response.json({ error: 'Malformed GitHub URL' }, { status: 400 })
        }

        const token = process.env.GITHUB_TOKEN
        const result = await fetchRepoFiles(parsed.owner, parsed.repo, token || undefined)

        if (result === 'not_found') {
          return Response.json({ error: 'Repository not found' }, { status: 404 })
        }

        const library = extractLibrary(result)
        if (!library) {
          return Response.json(
            { error: 'No extractable agent config found in this repository' },
            { status: 422 },
          )
        }

        return Response.json(library)
      },
    },
  },
})
