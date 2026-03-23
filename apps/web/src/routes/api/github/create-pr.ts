import { createFileRoute } from '@tanstack/react-router'
import { createConfigPr } from '#/lib/github-app'
import { getGitHubToken } from '#/lib/github-token'
import { extractLibrary, type RepoFiles } from '#/lib/github-import'
import { libraryToShipFiles } from '#/lib/ship-config'

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

async function fetchRepoFilesAuth(
  owner: string,
  repo: string,
  token: string,
): Promise<RepoFiles | 'not_found'> {
  const headers: Record<string, string> = {
    Authorization: `Bearer ${token}`,
    Accept: 'application/vnd.github.v3+json',
    'User-Agent': 'ship-studio/1.0',
  }

  const treeRes = await fetch(
    `https://api.github.com/repos/${owner}/${repo}/git/trees/HEAD?recursive=1`,
    { headers },
  )
  if (treeRes.status === 404 || treeRes.status === 403) return 'not_found'
  if (!treeRes.ok) return 'not_found'

  const tree = (await treeRes.json()) as GitHubTreeResponse
  const blobs = tree.tree.filter((item) => item.type === 'blob' && RELEVANT(item.path))

  const files: RepoFiles = {}
  await Promise.all(
    blobs.map(async (item) => {
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

interface CreatePrBody {
  owner: string
  repo: string
  default_branch: string
  mode_name?: string
}

export const Route = createFileRoute('/api/github/create-pr')({
  server: {
    handlers: {
      /** Import config from repo and create a PR adding .ship/ directory. */
      POST: async ({ request }) => {
        const token = await getGitHubToken(request)
        if (!token) {
          return Response.json({ error: 'Not authenticated' }, { status: 401 })
        }

        let body: CreatePrBody
        try {
          body = (await request.json()) as CreatePrBody
        } catch {
          return Response.json({ error: 'Invalid JSON' }, { status: 400 })
        }

        const { owner, repo, default_branch, mode_name } = body
        if (!owner || !repo || !default_branch) {
          return Response.json({ error: 'Missing owner, repo, or default_branch' }, { status: 400 })
        }

        try {
          // 1. Fetch existing config from repo
          const repoFiles = await fetchRepoFilesAuth(owner, repo, token)
          if (repoFiles === 'not_found') {
            return Response.json({ error: 'Repository not found or no access' }, { status: 404 })
          }

          // 2. Extract library using existing logic
          const library = extractLibrary(repoFiles)

          // 3. Convert to .ship/ format (even if no existing config, create minimal)
          const shipFiles = libraryToShipFiles(
            library ?? { modes: [], active_agent: null, mcp_servers: [], skills: [], rules: [], agent_profiles: [], claude_team_agents: [], env: {}, available_models: [] },
            mode_name ?? 'default',
          )

          // 4. Create PR
          const pr = await createConfigPr(token, owner, repo, default_branch, shipFiles)

          return Response.json({ html_url: pr.html_url, number: pr.number })
        } catch (err) {
          const msg = err instanceof Error ? err.message : 'PR creation failed'
          return Response.json({ error: msg }, { status: 502 })
        }
      },
    },
  },
})
