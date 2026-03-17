import { createFileRoute } from '@tanstack/react-router'
import { parseGithubUrl, extractLibrary, type RepoFiles } from '#/lib/github-import'

type GitHubTreeItem = { path: string; type: string; url: string }
type GitHubTreeResponse = { tree: GitHubTreeItem[]; truncated?: boolean }
type GitHubBlobResponse = { content?: string; encoding?: string }

const RELEVANT = (path: string): boolean =>
  path === 'CLAUDE.md' ||
  path === 'AGENTS.md' ||
  path === '.mcp.json' ||
  path === '.gemini/GEMINI.md' ||
  (path.startsWith('.cursor/rules/') && path.endsWith('.mdc')) ||
  path.startsWith('.ship/agents/')

async function fetchRepoFiles(
  owner: string,
  repo: string,
  token?: string,
): Promise<RepoFiles | 'not_found'> {
  const headers: Record<string, string> = {
    Accept: 'application/vnd.github.v3+json',
    'User-Agent': 'ship-studio/1.0',
  }
  if (token) headers['Authorization'] = `Bearer ${token}`

  const treeRes = await fetch(
    `https://api.github.com/repos/${owner}/${repo}/git/trees/HEAD?recursive=1`,
    { headers },
  )
  if (treeRes.status === 404) return 'not_found'
  if (!treeRes.ok) return 'not_found'

  const tree = (await treeRes.json()) as GitHubTreeResponse
  const blobs = tree.tree.filter(item => item.type === 'blob' && RELEVANT(item.path))

  const files: RepoFiles = {}
  await Promise.all(
    blobs.map(async item => {
      const res = await fetch(item.url, { headers })
      if (!res.ok) return
      const data = (await res.json()) as GitHubBlobResponse
      if (data.encoding === 'base64' && data.content) {
        files[item.path] = atob(data.content.replace(/\n/g, ''))
      }
    }),
  )

  return files
}

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
