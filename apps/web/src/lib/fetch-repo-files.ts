// Shared GitHub repo file fetching logic.
// Extracted from /api/github/import.ts so that both the import endpoint
// and the registry seed pipeline can reuse it.

import type { RepoFiles } from '#/lib/github-import'

type GitHubTreeItem = { path: string; type: string; url: string }
type GitHubTreeResponse = { tree: GitHubTreeItem[]; truncated?: boolean }
type GitHubBlobResponse = { content?: string; encoding?: string }

/** Files we consider "agent config" — same filter used by the import endpoint. */
export const RELEVANT = (path: string): boolean =>
  path === 'CLAUDE.md' ||
  path === 'AGENTS.md' ||
  path === '.mcp.json' ||
  path === '.gemini/GEMINI.md' ||
  (path.startsWith('.cursor/rules/') && path.endsWith('.mdc')) ||
  path.startsWith('.ship/agents/')

/**
 * Fetch all relevant agent config files from a GitHub repo.
 * Returns a flat map of path -> content, or 'not_found' if the repo doesn't exist.
 */
export async function fetchRepoFiles(
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
