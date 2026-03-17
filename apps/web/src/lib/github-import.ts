import type { McpServerConfig, ProjectLibrary, Rule, Skill } from '@ship/ui'

export interface GithubRepo {
  owner: string
  repo: string
}

/** Parse a GitHub URL and return owner/repo, or null if invalid. */
export function parseGithubUrl(url: string): GithubRepo | null {
  let parsed: URL
  try {
    parsed = new URL(url)
  } catch {
    return null
  }
  if (parsed.hostname !== 'github.com') return null
  const parts = parsed.pathname.replace(/^\//, '').split('/')
  if (parts.length < 2 || !parts[0] || !parts[1]) return null
  return { owner: parts[0], repo: parts[1].replace(/\.git$/, '') }
}

/** Map of repo-relative file path → file content (UTF-8). */
export type RepoFiles = Record<string, string>

/** Extract a ProjectLibrary from a flat map of file paths to contents. Pure, no side effects. */
export function extractLibrary(files: RepoFiles): ProjectLibrary | null {
  // Priority 1: .ship/agents/ → native Ship project
  const hasShipProject = Object.keys(files).some(k => k.startsWith('.ship/agents/'))
  if (hasShipProject) return extractFromShipProject(files)

  const rules: Rule[] = []
  const mcpServers: McpServerConfig[] = []

  // Priority 2: CLAUDE.md → rules
  if (files['CLAUDE.md']) {
    rules.push({ file_name: 'CLAUDE.md', content: files['CLAUDE.md'] })
  }

  // Priority 3: .mcp.json → mcp_servers
  if (files['.mcp.json']) {
    mcpServers.push(...parseMcpJson(files['.mcp.json']))
  }

  // Priority 4: .cursor/rules/*.mdc → rules
  for (const [path, content] of Object.entries(files)) {
    if (path.startsWith('.cursor/rules/') && path.endsWith('.mdc')) {
      rules.push({ file_name: path.split('/').pop()!, content })
    }
  }

  // Priority 5: AGENTS.md → rules
  if (files['AGENTS.md']) {
    rules.push({ file_name: 'AGENTS.md', content: files['AGENTS.md'] })
  }

  // Priority 6: .gemini/GEMINI.md → rules
  if (files['.gemini/GEMINI.md']) {
    rules.push({ file_name: 'GEMINI.md', content: files['.gemini/GEMINI.md'] })
  }

  if (rules.length === 0 && mcpServers.length === 0) return null

  return { modes: [], active_mode: null, mcp_servers: mcpServers, skills: [], rules, permissions: null }
}

function extractFromShipProject(files: RepoFiles): ProjectLibrary {
  const rules: Rule[] = []
  const skills: Skill[] = []
  const mcpServers: McpServerConfig[] = []

  for (const [path, content] of Object.entries(files)) {
    // .ship/agents/rules/*.md → rules
    if (/^\.ship\/agents\/rules\/[^/]+\.md$/.test(path)) {
      rules.push({ file_name: path.split('/').pop()!, content })
    }
    // .ship/agents/skills/<id>/SKILL.md → skills
    const skillMatch = path.match(/^\.ship\/agents\/skills\/([^/]+)\/SKILL\.md$/)
    if (skillMatch) {
      skills.push({ id: skillMatch[1], name: skillMatch[1], content })
    }
  }

  if (files['.ship/agents/mcp.toml']) {
    mcpServers.push(...parseShipMcpToml(files['.ship/agents/mcp.toml']))
  }

  return { modes: [], active_mode: null, mcp_servers: mcpServers, skills, rules, permissions: null }
}

function parseMcpJson(content: string): McpServerConfig[] {
  try {
    const data = JSON.parse(content) as Record<string, unknown>
    const servers = (data.mcpServers ?? data.mcp_servers ?? {}) as Record<string, Record<string, unknown>>
    return Object.entries(servers).map(([name, cfg]) => ({
      name,
      command: typeof cfg.command === 'string' ? cfg.command : '',
      ...(Array.isArray(cfg.args) ? { args: cfg.args as string[] } : {}),
      ...(cfg.env && typeof cfg.env === 'object' ? { env: cfg.env as Record<string, string> } : {}),
      ...(typeof cfg.url === 'string' ? { url: cfg.url } : {}),
    }))
  } catch {
    return []
  }
}

/**
 * Minimal [[servers]] TOML parser — handles only the fields used in .ship/agents/mcp.toml.
 * Not a general TOML parser.
 */
function parseShipMcpToml(content: string): McpServerConfig[] {
  const servers: McpServerConfig[] = []
  const blocks = content.split(/(?=\[\[servers\]\])/g).slice(1)

  for (const block of blocks) {
    let name = ''
    let command = ''
    let url: string | undefined
    let args: string[] | undefined

    for (const line of block.split('\n')) {
      const m = line.match(/^\s*(\w+)\s*=\s*(.+)$/)
      if (!m) continue
      const [, key, rawVal] = m
      if (key === 'id' || key === 'name') name = tomlUnquote(rawVal)
      else if (key === 'command') command = tomlUnquote(rawVal)
      else if (key === 'url') url = tomlUnquote(rawVal)
      else if (key === 'args') args = tomlStringArray(rawVal)
    }

    if (name && (command || url)) {
      servers.push({ name, command, ...(args ? { args } : {}), ...(url ? { url } : {}) })
    }
  }

  return servers
}

function tomlUnquote(s: string): string {
  return s.trim().replace(/^["']|["']$/g, '')
}

function tomlStringArray(s: string): string[] {
  const m = s.match(/\[([^\]]*)\]/)
  if (!m) return []
  return m[1]
    .split(',')
    .map(v => tomlUnquote(v))
    .filter(Boolean)
}
