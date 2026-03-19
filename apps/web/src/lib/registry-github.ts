// GitHub API helpers for the registry publish flow.
// Fetches files from repos, parses ship.toml manifests.

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

/**
 * Parse a minimal ship.toml manifest.
 *
 * This is a lightweight TOML parser that handles only the subset we need:
 * [module] and [exports] sections with simple key = "value" and arrays.
 */
export function parseShipToml(content: string): ParsedShipToml {
  const result: ParsedShipToml = {}
  let currentSection = ''

  for (const line of content.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#')) continue

    const sectionMatch = trimmed.match(/^\[(\w+)\]$/)
    if (sectionMatch) {
      currentSection = sectionMatch[1]
      if (currentSection === 'module') result.module = { name: '' }
      if (currentSection === 'exports') result.exports = {}
      continue
    }

    const kvMatch = trimmed.match(/^(\w+)\s*=\s*(.+)$/)
    if (!kvMatch) continue

    const [, key, rawValue] = kvMatch
    const value = parseTomlValue(rawValue)

    if (currentSection === 'module' && result.module) {
      if (key === 'name') result.module.name = value as string
      if (key === 'version') result.module.version = value as string
      if (key === 'description') result.module.description = value as string
      if (key === 'license') result.module.license = value as string
    }

    if (currentSection === 'exports' && result.exports) {
      if (key === 'skills') result.exports.skills = value as string[]
      if (key === 'agents') result.exports.agents = value as string[]
    }
  }

  return result
}

function parseTomlValue(raw: string): string | string[] {
  const trimmed = raw.trim()

  // Array: ["a", "b", "c"]
  if (trimmed.startsWith('[')) {
    const inner = trimmed.slice(1, -1)
    return inner
      .split(',')
      .map((s) => s.trim().replace(/^["']|["']$/g, ''))
      .filter(Boolean)
  }

  // Quoted string
  if (trimmed.startsWith('"') || trimmed.startsWith("'")) {
    return trimmed.slice(1, -1)
  }

  return trimmed
}
