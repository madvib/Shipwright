import { useState, useEffect, useCallback, useRef } from 'react'
import { Search, Plus, ExternalLink, Loader2, Server, RefreshCw, Wifi, WifiOff } from 'lucide-react'
import type { McpRegistryServer, McpServerConfig } from '../types'

const REGISTRY_API = 'https://registry.modelcontextprotocol.io'

// Curated fallback — shown when the live registry is unreachable.
// Mirrors the CURATED_MCP list in the Studio Library panel.
const CURATED_FALLBACK: McpRegistryServer[] = [
  {
    id: 'github', name: 'GitHub',
    description: 'Search repos, manage PRs and issues, create commits.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/github',
    tags: ['popular', 'dev'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
  {
    id: 'filesystem', name: 'Filesystem',
    description: 'Read and write local files within configurable allowed paths.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem',
    tags: ['popular', 'files'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
  {
    id: 'brave-search', name: 'Brave Search',
    description: 'Web search via the Brave Search API.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/brave-search',
    tags: ['search', 'web'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-brave-search', command: 'npx', args: ['-y', '@modelcontextprotocol/server-brave-search'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
  {
    id: 'memory', name: 'Memory',
    description: 'Persistent knowledge graph — remember facts across sessions.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/memory',
    tags: ['memory', 'context'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-memory', command: 'npx', args: ['-y', '@modelcontextprotocol/server-memory'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
  {
    id: 'slack', name: 'Slack',
    description: 'Read channels, post messages, and manage Slack workspaces.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/slack',
    tags: ['comms', 'popular'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-slack', command: 'npx', args: ['-y', '@modelcontextprotocol/server-slack'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
  {
    id: 'linear', name: 'Linear',
    description: 'Manage issues, projects, and cycles in Linear.',
    homepage: 'https://github.com/linear/linear/tree/master/packages/mcp',
    tags: ['pm', 'issues'],
    package: { registry: 'npm', name: '@linear/mcp-server', command: 'npx', args: ['-y', '@linear/mcp-server'] },
    vendor: { name: 'Linear', url: 'https://linear.app' },
  },
  {
    id: 'playwright', name: 'Playwright',
    description: 'Browser automation — navigate, screenshot, interact with web pages.',
    homepage: 'https://github.com/executeautomation/mcp-playwright',
    tags: ['browser', 'testing'],
    package: { registry: 'npm', name: '@executeautomation/playwright-mcp-server', command: 'npx', args: ['-y', '@executeautomation/playwright-mcp-server'] },
    vendor: { name: 'ExecuteAutomation', url: 'https://executeautomation.github.io' },
  },
  {
    id: 'postgres', name: 'PostgreSQL',
    description: 'Query and inspect a PostgreSQL database with read-only access.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/postgres',
    tags: ['database', 'sql'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-postgres', command: 'npx', args: ['-y', '@modelcontextprotocol/server-postgres'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
  {
    id: 'puppeteer', name: 'Puppeteer',
    description: 'Headless browser control for scraping, screenshots, and automation.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/puppeteer',
    tags: ['browser', 'scraping'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-puppeteer', command: 'npx', args: ['-y', '@modelcontextprotocol/server-puppeteer'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
  {
    id: 'sqlite', name: 'SQLite',
    description: 'Read and write a local SQLite database file.',
    homepage: 'https://github.com/modelcontextprotocol/servers/tree/main/src/sqlite',
    tags: ['database', 'local'],
    package: { registry: 'npm', name: '@modelcontextprotocol/server-sqlite', command: 'npx', args: ['-y', '@modelcontextprotocol/server-sqlite'] },
    vendor: { name: 'Anthropic', url: 'https://modelcontextprotocol.io' },
  },
]

interface RegistryResponse {
  servers: McpRegistryServer[]
  total?: number
  next?: string | null
}

interface Props {
  onAdd: (server: McpServerConfig) => void
  addedIds?: Set<string>
}

type RegistryStatus = 'loading' | 'live' | 'offline'

export function McpRegistryBrowser({ onAdd, addedIds = new Set() }: Props) {
  const [query, setQuery] = useState('')
  const [servers, setServers] = useState<McpRegistryServer[]>([])
  const [status, setStatus] = useState<RegistryStatus>('loading')
  const [hasMore, setHasMore] = useState(false)
  const [nextCursor, setNextCursor] = useState<string | null>(null)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const fetchServers = useCallback(async (q: string, append = false) => {
    setStatus('loading')
    try {
      const params = new URLSearchParams()
      if (q) params.set('q', q)
      params.set('per_page', '20')
      const res = await fetch(`${REGISTRY_API}/api/v0/servers?${params}`)
      if (!res.ok) throw new Error(`${res.status}`)
      const data = (await res.json()) as RegistryResponse
      const list = data.servers ?? []
      setServers(append ? (prev) => [...prev, ...list] : list)
      setHasMore(!!data.next)
      setNextCursor(data.next ?? null)
      setStatus('live')
    } catch {
      // Fall back to curated list
      const filtered = q
        ? CURATED_FALLBACK.filter(
            (s) =>
              s.name.toLowerCase().includes(q.toLowerCase()) ||
              (s.description ?? '').toLowerCase().includes(q.toLowerCase()) ||
              (s.tags ?? []).some((t) => t.includes(q.toLowerCase()))
          )
        : CURATED_FALLBACK
      setServers(filtered)
      setHasMore(false)
      setNextCursor(null)
      setStatus('offline')
    }
  }, [])

  useEffect(() => { void fetchServers('') }, [fetchServers])

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => { void fetchServers(query) }, 300)
    return () => { if (debounceRef.current) clearTimeout(debounceRef.current) }
  }, [query, fetchServers])

  const addToConfig = (s: McpRegistryServer) => {
    const pkg = s.package
    const server: McpServerConfig = {
      name: s.id ?? s.name.toLowerCase().replace(/[^a-z0-9-]/g, '-'),
      command: pkg?.command ?? (pkg?.registry === 'npm' ? 'npx' : pkg?.registry === 'pypi' ? 'uvx' : 'npx'),
      args: pkg?.args ?? (pkg?.name ? ['-y', pkg.name] : []),
      env: {},
      server_type: 'stdio',
      scope: 'project',
      disabled: false,
      url: null,
      timeout_secs: null,
    }
    onAdd(server)
  }

  const isLoading = status === 'loading'

  return (
    <div className="flex flex-col gap-3">
      {/* Search */}
      <div className="relative">
        <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground pointer-events-none" />
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search MCP servers..."
          className="h-8 w-full rounded-lg border border-border bg-background pl-8 pr-8 text-xs focus:outline-none focus:ring-1 focus:ring-primary/40"
        />
        {isLoading && (
          <Loader2 className="absolute right-2.5 top-1/2 -translate-y-1/2 size-3.5 text-muted-foreground animate-spin" />
        )}
      </div>

      {/* Status bar */}
      {!isLoading && (
        <div className={`flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-[10px] ${
          status === 'live'
            ? 'bg-emerald-500/8 text-emerald-600 dark:text-emerald-400'
            : 'bg-muted/60 text-muted-foreground'
        }`}>
          {status === 'live'
            ? <Wifi className="size-3 shrink-0" />
            : <WifiOff className="size-3 shrink-0" />
          }
          {status === 'live'
            ? 'Live registry'
            : 'Registry unavailable — showing curated catalog'
          }
          {status === 'offline' && (
            <button
              onClick={() => void fetchServers(query)}
              className="ml-auto flex items-center gap-1 hover:text-foreground transition"
            >
              <RefreshCw className="size-2.5" /> Retry
            </button>
          )}
        </div>
      )}

      {/* Results */}
      <div className="space-y-1.5">
        {servers.length === 0 && !isLoading && (
          <p className="py-6 text-center text-xs text-muted-foreground">No servers found.</p>
        )}

        {servers.map((s) => {
          const isAdded = addedIds.has(s.id ?? s.name)
          return (
            <div
              key={s.id ?? s.name}
              className="group flex items-start gap-2.5 rounded-xl border border-border/60 bg-card/50 p-3 transition hover:border-border hover:bg-card"
            >
              <div className="flex size-7 shrink-0 items-center justify-center rounded-lg border border-border bg-muted/40 mt-0.5">
                <Server className="size-3.5 text-muted-foreground" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 flex-wrap">
                  <p className="text-xs font-semibold truncate">{s.name}</p>
                  {s.vendor?.name && (
                    <span className="text-[10px] text-muted-foreground">by {s.vendor.name}</span>
                  )}
                  {s.tags?.slice(0, 3).map((tag) => (
                    <span key={tag} className="rounded bg-muted px-1.5 py-0.5 text-[9px] text-muted-foreground">
                      {tag}
                    </span>
                  ))}
                </div>
                {s.description && (
                  <p className="mt-0.5 text-[11px] leading-snug text-muted-foreground line-clamp-2">
                    {s.description}
                  </p>
                )}
                {s.package?.name && (
                  <p className="mt-1 font-mono text-[10px] text-muted-foreground/60">{s.package.name}</p>
                )}
              </div>
              <div className="flex items-center gap-1.5 shrink-0">
                {s.homepage && (
                  <a
                    href={s.homepage}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="flex size-6 items-center justify-center rounded text-muted-foreground/60 transition hover:text-foreground"
                    aria-label="Open homepage"
                  >
                    <ExternalLink className="size-3" />
                  </a>
                )}
                <button
                  onClick={() => addToConfig(s)}
                  disabled={isAdded}
                  className={`flex items-center gap-1 rounded-lg px-2.5 py-1 text-[11px] font-medium transition ${
                    isAdded
                      ? 'bg-muted text-muted-foreground cursor-default'
                      : 'bg-primary/10 text-primary hover:bg-primary/20'
                  }`}
                >
                  {isAdded ? 'Added' : <><Plus className="size-3" /> Add</>}
                </button>
              </div>
            </div>
          )
        })}
      </div>

      {hasMore && (
        <button
          onClick={() => { if (nextCursor) void fetchServers(query, true) }}
          disabled={isLoading}
          className="flex items-center justify-center gap-1.5 rounded-lg border border-border/60 py-2 text-xs text-muted-foreground transition hover:text-foreground disabled:opacity-50"
        >
          {isLoading ? <Loader2 className="size-3.5 animate-spin" /> : null}
          Load more
        </button>
      )}
    </div>
  )
}
