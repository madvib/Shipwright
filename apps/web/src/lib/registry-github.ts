// GitHub API helpers for the registry publish flow.
// Fetches files from repos, parses ship.jsonc manifests.

export interface ParsedGitHubUrl {
  owner: string
  repo: string
}

export interface ShipModule {
  name: string
  version?: string
  description?: string
  license?: string
}

export interface ShipExports {
  skills?: string[]
  agents?: string[]
}

export interface ParsedShipToml {
  module?: ShipModule
  exports?: ShipExports
}

type GitHubBlobResponse = { content?: string; encoding?: string }
type GitHubTreeItem = { path: string; type: string; url: string; sha: string }
type GitHubTreeResponse = { tree: GitHubTreeItem[]; truncated?: boolean; sha: string }

const GITHUB_HEADERS: Record<string, string> = {
  Accept: 'application/vnd.github.v3+json',
  'User-Agent': 'ship-registry/1.0',
}

function getAuthHeaders(): Record<string, string> {
  const token = (globalThis as Record<string, unknown>)['GITHUB_TOKEN'] as
    | string
    | undefined
  const headers = { ...GITHUB_HEADERS }
  if (token) headers['Authorization'] = `Bearer ${token}`
  return headers
}

/** Parse a GitHub URL into owner/repo. Returns null for non-GitHub URLs. */
export function parseGithubUrl(url: string): ParsedGitHubUrl | null {
  const match = url.match(
    /(?:https?:\/\/)?github\.com\/([a-zA-Z0-9._-]+)\/([a-zA-Z0-9._-]+)/,
  )
  if (!match) return null
  return { owner: match[1], repo: match[2].replace(/\.git$/, '') }
}

/** Fetch a single file's content from a GitHub repo at a given ref. */
export async function fetchFileFromGitHub(
  owner: string,
  repo: string,
  path: string,
  ref: string,
): Promise<string | null> {
  const url = `https://api.github.com/repos/${owner}/${repo}/contents/${path}?ref=${ref}`
  const res = await fetch(url, { headers: getAuthHeaders() })
  if (!res.ok) return null

  const data = (await res.json()) as GitHubBlobResponse
  if (data.encoding === 'base64' && data.content) {
    return atob(data.content.replace(/\n/g, ''))
  }
  return null
}

/** Fetch the full file tree from a GitHub repo at a given ref. */
export async function fetchTreeFromGitHub(
  owner: string,
  repo: string,
  ref: string,
): Promise<GitHubTreeItem[]> {
  const url = `https://api.github.com/repos/${owner}/${repo}/git/trees/${ref}?recursive=1`
  const res = await fetch(url, { headers: getAuthHeaders() })
  if (!res.ok) return []

  const data = (await res.json()) as GitHubTreeResponse
  return data.tree.filter((item) => item.type === 'blob')
}

/** Resolve a git ref to a commit SHA via the GitHub API. */
export async function resolveGitHubRef(
  owner: string,
  repo: string,
  ref: string,
): Promise<string | null> {
  const url = `https://api.github.com/repos/${owner}/${repo}/commits/${ref}`
  const res = await fetch(url, { headers: getAuthHeaders() })
  if (!res.ok) return null
  const data = (await res.json()) as { sha?: string }
  return data.sha ?? null
}

/**
 * Fetch the ship.jsonc manifest from a GitHub repo.
 * Returns { content, format } or null if not found.
 */
export async function fetchShipManifest(
  owner: string,
  repo: string,
  ref: string,
): Promise<{ content: string; format: 'jsonc' } | null> {
  const jsonc = await fetchFileFromGitHub(owner, repo, '.ship/ship.jsonc', ref)
  if (jsonc) return { content: jsonc, format: 'jsonc' }
  return null
}

/**
 * Parse a ship.jsonc manifest.
 * The registry only supports JSONC format.
 */
export function parseShipManifest(
  content: string,
  _format: 'jsonc' = 'jsonc',
): ParsedShipToml {
  return parseShipJsonc(content)
}

/**
 * Parse a ship.jsonc manifest.
 * Strips JSONC comments and trailing commas, then extracts module/exports.
 */
export function parseShipJsonc(content: string): ParsedShipToml {
  const stripped = stripJsoncComments(content)
  try {
    const obj = JSON.parse(stripped) as Record<string, unknown>
    const result: ParsedShipToml = {}

    const mod = obj.module as Record<string, unknown> | undefined
    if (mod) {
      result.module = {
        name: (mod.name as string) || '',
        version: mod.version as string | undefined,
        description: mod.description as string | undefined,
        license: mod.license as string | undefined,
      }
    }

    const exp = obj.exports as Record<string, unknown> | undefined
    if (exp) {
      result.exports = {
        skills: Array.isArray(exp.skills) ? (exp.skills as string[]) : undefined,
        agents: Array.isArray(exp.agents) ? (exp.agents as string[]) : undefined,
      }
    }

    return result
  } catch {
    return {}
  }
}

/** Strip // and /* comments plus trailing commas from JSONC. */
function stripJsoncComments(input: string): string {
  let out = ''
  let i = 0
  const len = input.length

  while (i < len) {
    if (input[i] === '"') {
      out += '"'
      i++
      while (i < len) {
        if (input[i] === '\\' && i + 1 < len) {
          out += input[i] + input[i + 1]
          i += 2
        } else if (input[i] === '"') {
          out += '"'
          i++
          break
        } else {
          out += input[i]
          i++
        }
      }
    } else if (input[i] === '/' && i + 1 < len && input[i + 1] === '/') {
      i += 2
      while (i < len && input[i] !== '\n') i++
    } else if (input[i] === '/' && i + 1 < len && input[i + 1] === '*') {
      i += 2
      while (i + 1 < len && !(input[i] === '*' && input[i + 1] === '/')) i++
      if (i + 1 < len) i += 2
    } else {
      out += input[i]
      i++
    }
  }

  // Strip trailing commas before ] or }
  return out.replace(/,(\s*[}\]])/g, '$1')
}

