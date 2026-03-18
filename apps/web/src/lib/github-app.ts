/**
 * GitHub App OAuth utilities.
 *
 * Environment variables (set in .env.local):
 *   GITHUB_APP_CLIENT_ID  — GitHub App OAuth client ID
 *   GITHUB_APP_CLIENT_SECRET — GitHub App OAuth client secret
 */

// ── OAuth flow helpers ───────────────────────────────────────────────────────

const GITHUB_OAUTH_AUTHORIZE = 'https://github.com/login/oauth/authorize'
const GITHUB_OAUTH_TOKEN = 'https://github.com/login/oauth/access_token'

/** Scopes needed: repo contents (read), pull requests (write), metadata (read). */
const SCOPES = 'repo'

export function buildAuthorizeUrl(clientId: string, redirectUri: string, state: string): string {
  const params = new URLSearchParams({
    client_id: clientId,
    redirect_uri: redirectUri,
    scope: SCOPES,
    state,
  })
  return `${GITHUB_OAUTH_AUTHORIZE}?${params}`
}

export interface TokenResponse {
  access_token: string
  token_type: string
  scope: string
}

export async function exchangeCodeForToken(
  code: string,
  clientId: string,
  clientSecret: string,
): Promise<TokenResponse> {
  const res = await fetch(GITHUB_OAUTH_TOKEN, {
    method: 'POST',
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      client_id: clientId,
      client_secret: clientSecret,
      code,
    }),
  })

  if (!res.ok) throw new Error(`GitHub token exchange failed: ${res.status}`)

  const data = (await res.json()) as Record<string, unknown>
  if (data.error) throw new Error(`GitHub OAuth error: ${data.error_description ?? data.error}`)

  return data as unknown as TokenResponse
}

// ── Authenticated GitHub API helpers ─────────────────────────────────────────

function ghHeaders(token: string): Record<string, string> {
  return {
    Authorization: `Bearer ${token}`,
    Accept: 'application/vnd.github.v3+json',
    'User-Agent': 'ship-studio/1.0',
  }
}

export interface GitHubUser {
  login: string
  avatar_url: string
}

export async function getUser(token: string): Promise<GitHubUser> {
  const res = await fetch('https://api.github.com/user', { headers: ghHeaders(token) })
  if (!res.ok) throw new Error(`GitHub API /user failed: ${res.status}`)
  return (await res.json()) as GitHubUser
}

export interface GitHubRepo {
  full_name: string
  name: string
  owner: { login: string }
  private: boolean
  default_branch: string
  description: string | null
}

export async function listRepos(token: string, page = 1): Promise<GitHubRepo[]> {
  const params = new URLSearchParams({
    sort: 'pushed',
    per_page: '30',
    page: String(page),
  })
  const res = await fetch(`https://api.github.com/user/repos?${params}`, {
    headers: ghHeaders(token),
  })
  if (!res.ok) throw new Error(`GitHub API /user/repos failed: ${res.status}`)
  return (await res.json()) as GitHubRepo[]
}

// ── Cookie helpers ───────────────────────────────────────────────────────────

const TOKEN_COOKIE = 'gh_token'
const STATE_COOKIE = 'gh_oauth_state'

export function setTokenCookie(token: string): string {
  return `${TOKEN_COOKIE}=${token}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=86400`
}

export function setStateCookie(state: string): string {
  return `${STATE_COOKIE}=${state}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=600`
}

export function clearStateCookie(): string {
  return `${STATE_COOKIE}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0`
}

export function clearTokenCookie(): string {
  return `${TOKEN_COOKIE}=; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=0`
}

export function getTokenFromCookie(request: Request): string | null {
  return getCookieValue(request, TOKEN_COOKIE)
}

export function getStateFromCookie(request: Request): string | null {
  return getCookieValue(request, STATE_COOKIE)
}

function getCookieValue(request: Request, name: string): string | null {
  const header = request.headers.get('Cookie') ?? ''
  const match = header.match(new RegExp(`(?:^|;\\s*)${name}=([^;]+)`))
  return match?.[1] ?? null
}

// ── PR creation ──────────────────────────────────────────────────────────────

interface TreeEntry {
  path: string
  mode: '100644'
  type: 'blob'
  content: string
}

/**
 * Create a PR that adds .ship/ configuration and updates .gitignore.
 * Uses the Git Data API to create a tree + commit on a new branch.
 */
export async function createConfigPr(
  token: string,
  owner: string,
  repo: string,
  defaultBranch: string,
  files: Record<string, string>,
): Promise<{ html_url: string; number: number }> {
  const headers = ghHeaders(token)
  const api = `https://api.github.com/repos/${owner}/${repo}`

  // 1. Get the latest commit SHA on default branch
  const refRes = await fetch(`${api}/git/ref/heads/${defaultBranch}`, { headers })
  if (!refRes.ok) throw new Error(`Failed to get ref: ${refRes.status}`)
  const refData = (await refRes.json()) as { object: { sha: string } }
  const baseSha = refData.object.sha

  // 2. Get the base tree
  const commitRes = await fetch(`${api}/git/commits/${baseSha}`, { headers })
  if (!commitRes.ok) throw new Error(`Failed to get commit: ${commitRes.status}`)
  const commitData = (await commitRes.json()) as { tree: { sha: string } }
  const baseTreeSha = commitData.tree.sha

  // 3. Fetch existing .gitignore to append to it
  let existingGitignore = ''
  try {
    const giRes = await fetch(`${api}/contents/.gitignore?ref=${defaultBranch}`, { headers })
    if (giRes.ok) {
      const giData = (await giRes.json()) as { content?: string; encoding?: string }
      if (giData.encoding === 'base64' && giData.content) {
        existingGitignore = atob(giData.content.replace(/\n/g, ''))
      }
    }
  } catch { /* no .gitignore, that's fine */ }

  // 4. Build tree entries
  const treeEntries: TreeEntry[] = Object.entries(files).map(([path, content]) => ({
    path,
    mode: '100644',
    type: 'blob',
    content,
  }))

  // Add .gitignore update
  const shipIgnoreLines = [
    '# Ship generated artifacts',
    'CLAUDE.md',
    'GEMINI.md',
    'AGENTS.md',
    '.mcp.json',
    '.cursor/mcp.json',
    '.cursor/rules/',
    '.gemini/settings.json',
    '.codex/',
    '.claude/settings.json',
  ]
  const newIgnoreBlock = shipIgnoreLines.join('\n')
  if (!existingGitignore.includes('# Ship generated artifacts')) {
    const separator = existingGitignore.endsWith('\n') || !existingGitignore ? '' : '\n'
    const updatedGitignore = existingGitignore + separator + '\n' + newIgnoreBlock + '\n'
    treeEntries.push({ path: '.gitignore', mode: '100644', type: 'blob', content: updatedGitignore })
  }

  // 5. Create tree
  const treeRes = await fetch(`${api}/git/trees`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ base_tree: baseTreeSha, tree: treeEntries }),
  })
  if (!treeRes.ok) throw new Error(`Failed to create tree: ${treeRes.status}`)
  const treeData = (await treeRes.json()) as { sha: string }

  // 6. Create commit
  const branchName = 'ship/add-config'
  const cmtRes = await fetch(`${api}/git/commits`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      message: 'feat: add Ship agent configuration\n\nAdds .ship/ directory with imported agent config and updates .gitignore\nto exclude generated provider artifacts.',
      tree: treeData.sha,
      parents: [baseSha],
    }),
  })
  if (!cmtRes.ok) throw new Error(`Failed to create commit: ${cmtRes.status}`)
  const cmtData = (await cmtRes.json()) as { sha: string }

  // 7. Create branch
  const branchRes = await fetch(`${api}/git/refs`, {
    method: 'POST',
    headers,
    body: JSON.stringify({ ref: `refs/heads/${branchName}`, sha: cmtData.sha }),
  })
  if (!branchRes.ok && branchRes.status !== 422) {
    throw new Error(`Failed to create branch: ${branchRes.status}`)
  }
  // If 422, branch might already exist — update it
  if (branchRes.status === 422) {
    const updateRes = await fetch(`${api}/git/refs/heads/${branchName}`, {
      method: 'PATCH',
      headers,
      body: JSON.stringify({ sha: cmtData.sha, force: true }),
    })
    if (!updateRes.ok) throw new Error(`Failed to update branch: ${updateRes.status}`)
  }

  // 8. Create PR
  const prBody = [
    '## What is Ship?',
    '',
    '[Ship](https://ship-studio.com) is a universal agent configuration platform.',
    'It compiles a single `.ship/` source-of-truth into provider-specific config files',
    'for Claude Code, Gemini CLI, Codex CLI, and Cursor.',
    '',
    '## What this PR does',
    '',
    '- Adds `.ship/` directory with your imported agent configuration',
    '- Updates `.gitignore` to exclude generated provider artifacts',
    '',
    'After merging, run `ship use` to generate provider files from this config.',
  ].join('\n')

  const prRes = await fetch(`${api}/pulls`, {
    method: 'POST',
    headers,
    body: JSON.stringify({
      title: 'feat: add Ship agent configuration',
      body: prBody,
      head: branchName,
      base: defaultBranch,
    }),
  })

  if (!prRes.ok) {
    const errBody = await prRes.text()
    throw new Error(`Failed to create PR: ${prRes.status} — ${errBody}`)
  }

  return (await prRes.json()) as { html_url: string; number: number }
}
