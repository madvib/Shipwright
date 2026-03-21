// MCP registry proxy — fetches from the official MCP registry, caches in D1.
// Falls back to a curated list when the upstream registry is unreachable.

const REGISTRY_API = 'https://registry.modelcontextprotocol.io'
const CACHE_TTL_MS = 24 * 60 * 60 * 1000 // 24 hours

// ── Vetted server IDs ──────────────────────────────────────────────────────

export const VETTED_SERVERS: string[] = [
  'github',
  'filesystem',
  'brave-search',
  'memory',
  'slack',
  'linear',
  'playwright',
  'postgres',
  'puppeteer',
  'sqlite',
  'sentry',
  'fetch',
  'sequential-thinking',
  'everart',
  'google-maps',
]

// ── McpServer type (API response shape) ────────────────────────────────────

export interface McpServer {
  id: string
  name: string
  description: string | null
  homepage: string | null
  tags: string[]
  vendor: string | null
  packageRegistry: string | null
  command: string | null
  args: string[]
  vetted: boolean
  imageUrl: string | null
}

// ── Image URL resolution ───────────────────────────────────────────────────

/** Extract a GitHub org/user avatar URL from a homepage or repository URL. */
function resolveImageUrl(homepage?: string | null): string | null {
  if (!homepage) return null
  try {
    const url = new URL(homepage)
    if (url.hostname === 'github.com') {
      const org = url.pathname.split('/').filter(Boolean)[0]
      if (org) return `https://github.com/${org}.png`
    }
  } catch {
    // Not a valid URL — skip
  }
  return null
}

// ── Upstream registry types ────────────────────────────────────────────────

interface UpstreamServer {
  id?: string
  name: string
  description?: string
  homepage?: string
  repository?: string
  tags?: string[]
  package?: {
    registry?: string
    name?: string
    command?: string
    args?: string[]
  }
  vendor?: {
    name?: string
    url?: string
  }
}

interface UpstreamResponse {
  servers: UpstreamServer[]
}

// ── Curated fallback ───────────────────────────────────────────────────────

const CURATED_FALLBACK: McpServer[] = [
  { id: 'github', name: 'GitHub', description: 'Search repos, manage PRs and issues, create commits.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/github', tags: ['popular', 'dev'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'filesystem', name: 'Filesystem', description: 'Read and write local files within configurable allowed paths.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem', tags: ['popular', 'files'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'brave-search', name: 'Brave Search', description: 'Web search via the Brave Search API.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/brave-search', tags: ['search', 'web'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-brave-search'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'memory', name: 'Memory', description: 'Persistent knowledge graph -- remember facts across sessions.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/memory', tags: ['memory', 'context'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-memory'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'slack', name: 'Slack', description: 'Read channels, post messages, and manage Slack workspaces.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/slack', tags: ['comms', 'popular'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-slack'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'linear', name: 'Linear', description: 'Manage issues, projects, and cycles in Linear.', homepage: 'https://github.com/linear/linear/tree/master/packages/mcp', tags: ['pm', 'issues'], vendor: 'Linear', packageRegistry: 'npm', command: 'npx', args: ['-y', '@linear/mcp-server'], vetted: true, imageUrl: 'https://github.com/linear.png' },
  { id: 'playwright', name: 'Playwright', description: 'Browser automation -- navigate, screenshot, interact with web pages.', homepage: 'https://github.com/executeautomation/mcp-playwright', tags: ['browser', 'testing'], vendor: 'ExecuteAutomation', packageRegistry: 'npm', command: 'npx', args: ['-y', '@executeautomation/playwright-mcp-server'], vetted: true, imageUrl: 'https://github.com/executeautomation.png' },
  { id: 'postgres', name: 'PostgreSQL', description: 'Query and inspect a PostgreSQL database with read-only access.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/postgres', tags: ['database', 'sql'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-postgres'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'puppeteer', name: 'Puppeteer', description: 'Headless browser control for scraping, screenshots, and automation.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer', tags: ['browser', 'scraping'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-puppeteer'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
  { id: 'sqlite', name: 'SQLite', description: 'Read and write a local SQLite database file.', homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite', tags: ['database', 'local'], vendor: 'Anthropic', packageRegistry: 'npm', command: 'npx', args: ['-y', '@modelcontextprotocol/server-sqlite'], vetted: true, imageUrl: 'https://github.com/modelcontextprotocol.png' },
]

// ── Cache row ↔ McpServer mapping ──────────────────────────────────────────

interface CacheRow {
  id: string
  name: string
  description: string | null
  homepage: string | null
  tags: string | null
  vendor: string | null
  package_registry: string | null
  command: string | null
  args: string | null
  vetted: number
  image_url: string | null
  cached_at: number
}

function rowToServer(row: CacheRow): McpServer {
  return {
    id: row.id,
    name: row.name,
    description: row.description,
    homepage: row.homepage,
    tags: row.tags ? JSON.parse(row.tags) : [],
    vendor: row.vendor,
    packageRegistry: row.package_registry,
    command: row.command,
    args: row.args ? JSON.parse(row.args) : [],
    vetted: row.vetted === 1,
    imageUrl: row.image_url,
  }
}

function upstreamToRow(s: UpstreamServer, now: number): CacheRow {
  const id = s.id ?? s.name.toLowerCase().replace(/[^a-z0-9-]/g, '-')
  return {
    id,
    name: s.name,
    description: s.description ?? null,
    homepage: s.homepage ?? null,
    tags: s.tags ? JSON.stringify(s.tags) : null,
    vendor: s.vendor?.name ?? null,
    package_registry: s.package?.registry ?? null,
    command: s.package?.command ?? null,
    args: s.package?.args ? JSON.stringify(s.package.args) : null,
    vetted: VETTED_SERVERS.includes(id) ? 1 : 0,
    image_url: resolveImageUrl(s.homepage) ?? resolveImageUrl(s.repository),
    cached_at: now,
  }
}

// ── Cache operations ───────────────────────────────────────────────────────

async function isCacheFresh(db: D1Database): Promise<boolean> {
  const row = await db
    .prepare('SELECT MAX(cached_at) as latest FROM mcp_servers_cache')
    .first<{ latest: number | null }>()
  if (!row?.latest) return false
  return Date.now() - row.latest < CACHE_TTL_MS
}

async function upsertCache(
  db: D1Database,
  rows: CacheRow[],
): Promise<void> {
  const stmt = db.prepare(
    `INSERT OR REPLACE INTO mcp_servers_cache
       (id, name, description, homepage, tags, vendor, package_registry,
        command, args, vetted, image_url, cached_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
  )
  // D1 batch limit is 100 statements — chunk if needed
  const BATCH_SIZE = 50
  for (let i = 0; i < rows.length; i += BATCH_SIZE) {
    const chunk = rows.slice(i, i + BATCH_SIZE)
    await db.batch(
      chunk.map((r) =>
        stmt.bind(
          r.id, r.name, r.description, r.homepage, r.tags,
          r.vendor, r.package_registry, r.command, r.args,
          r.vetted, r.image_url, r.cached_at,
        ),
      ),
    )
  }
}

// ── Upstream fetch ─────────────────────────────────────────────────────────

async function fetchUpstream(query?: string): Promise<UpstreamServer[]> {
  const params = new URLSearchParams()
  if (query) params.set('q', query)
  params.set('per_page', '50')
  const res = await fetch(
    `${REGISTRY_API}/api/v0/servers?${params}`,
    { signal: AbortSignal.timeout(5000) },
  )
  if (!res.ok) throw new Error(`Registry returned ${res.status}`)
  const data = (await res.json()) as UpstreamResponse
  return data.servers ?? []
}

// ── Public API ─────────────────────────────────────────────────────────────

export async function fetchMcpServers(
  db: D1Database | null,
  query?: string,
  vetted?: boolean,
): Promise<{ servers: McpServer[]; cached: boolean }> {
  // No DB — go straight to fallback
  if (!db) return { servers: filterFallback(query, vetted), cached: false }

  // Try cache first
  const fresh = await isCacheFresh(db)
  if (fresh) {
    const rows = await queryCache(db, query, vetted)
    if (rows.length > 0) return { servers: rows.map(rowToServer), cached: true }
  }

  // Cache miss or stale — refresh from upstream
  try {
    const upstream = await fetchUpstream(query)
    const now = Date.now()
    const rows = upstream.map((s) => upstreamToRow(s, now))
    // Fire-and-forget cache write — don't block the response
    void upsertCache(db, rows).catch(() => {})
    const servers = rows.map(rowToServer)
    return {
      servers: applyFilters(servers, vetted),
      cached: false,
    }
  } catch {
    // Upstream unreachable — try stale cache, then curated fallback
    const stale = await queryCache(db, query, vetted)
    if (stale.length > 0) return { servers: stale.map(rowToServer), cached: true }
    return { servers: filterFallback(query, vetted), cached: false }
  }
}

// ── Helpers ────────────────────────────────────────────────────────────────

async function queryCache(
  db: D1Database,
  query?: string,
  vetted?: boolean,
): Promise<CacheRow[]> {
  let sql = 'SELECT * FROM mcp_servers_cache WHERE 1=1'
  const bindings: (string | number)[] = []

  if (vetted) {
    sql += ' AND vetted = 1'
  }
  if (query) {
    sql += ' AND (name LIKE ? OR description LIKE ? OR tags LIKE ?)'
    const like = `%${query}%`
    bindings.push(like, like, like)
  }
  sql += ' ORDER BY vetted DESC, name ASC'

  const { results } = await db
    .prepare(sql)
    .bind(...bindings)
    .all<CacheRow>()
  return results ?? []
}

function applyFilters(servers: McpServer[], vetted?: boolean): McpServer[] {
  if (vetted) return servers.filter((s) => s.vetted)
  return servers
}

function filterFallback(query?: string, vetted?: boolean): McpServer[] {
  let list = CURATED_FALLBACK
  if (vetted) list = list.filter((s) => s.vetted)
  if (query) {
    const q = query.toLowerCase()
    list = list.filter(
      (s) =>
        s.name.toLowerCase().includes(q) ||
        (s.description ?? '').toLowerCase().includes(q) ||
        s.tags.some((t) => t.includes(q)),
    )
  }
  return list
}
