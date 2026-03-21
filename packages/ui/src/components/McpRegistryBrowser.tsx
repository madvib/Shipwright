import { useState, useEffect, useCallback, useRef } from 'react'
import { Search, Plus, ExternalLink, Loader2, Server, RefreshCw, Wifi, WifiOff, ShieldCheck } from 'lucide-react'
import type { McpServerConfig } from '../types'
import { CURATED_FALLBACK, type ProxyServer } from './mcp-registry-fallback'

interface ProxyResponse {
  servers: ProxyServer[]
  cached: boolean
}

interface Props {
  onAdd: (server: McpServerConfig) => void
  addedIds?: Set<string>
  /** Base URL for the proxy API. Defaults to '' (relative, works in the web app). */
  apiBaseUrl?: string
}

type RegistryStatus = 'loading' | 'live' | 'offline'

function ServerAvatar({ src, name }: { src: string | null; name: string }) {
  const [failed, setFailed] = useState(false)
  if (!src || failed) {
    return (
      <div className="flex size-8 shrink-0 items-center justify-center rounded-lg border border-border bg-muted/40">
        <Server className="size-3.5 text-muted-foreground" />
      </div>
    )
  }
  return (
    <img
      src={src}
      alt={`${name} logo`}
      loading="lazy"
      onError={() => setFailed(true)}
      className="size-8 shrink-0 rounded-lg border border-border bg-muted/40 object-cover"
    />
  )
}

function filterFallback(q: string, vetted: boolean): ProxyServer[] {
  let list = CURATED_FALLBACK
  if (vetted) list = list.filter((s) => s.vetted)
  if (q) {
    const lower = q.toLowerCase()
    list = list.filter(
      (s) =>
        s.name.toLowerCase().includes(lower) ||
        (s.description ?? '').toLowerCase().includes(lower) ||
        s.tags.some((t) => t.includes(lower)),
    )
  }
  return list
}

export function McpRegistryBrowser({
  onAdd,
  addedIds = new Set(),
  apiBaseUrl = '',
}: Props) {
  const [query, setQuery] = useState('')
  const [vettedOnly, setVettedOnly] = useState(true)
  const [servers, setServers] = useState<ProxyServer[]>([])
  const [status, setStatus] = useState<RegistryStatus>('loading')
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const fetchServers = useCallback(
    async (q: string, vetted: boolean) => {
      setStatus('loading')
      try {
        const params = new URLSearchParams()
        if (q) params.set('q', q)
        params.set('vetted', String(vetted))
        params.set('limit', '20')
        const res = await fetch(`${apiBaseUrl}/api/mcp/servers?${params}`)
        if (!res.ok) throw new Error(`${res.status}`)
        const data = (await res.json()) as ProxyResponse
        setServers(data.servers ?? [])
        setStatus('live')
      } catch {
        setServers(filterFallback(q, vetted))
        setStatus('offline')
      }
    },
    [apiBaseUrl],
  )

  useEffect(() => {
    void fetchServers('', vettedOnly)
  }, [fetchServers, vettedOnly])

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => {
      void fetchServers(query, vettedOnly)
    }, 300)
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current)
    }
  }, [query, vettedOnly, fetchServers])

  const addToConfig = (s: ProxyServer) => {
    const reg = s.packageRegistry
    const server: McpServerConfig = {
      name: s.id,
      command: s.command ?? (reg === 'npm' ? 'npx' : reg === 'pypi' ? 'uvx' : 'npx'),
      args: s.args.length > 0 ? s.args : [],
      env: {},
      server_type: 'stdio',
      scope: 'project',
      disabled: false,
      url: null,
      timeout_secs: null,
      codex_enabled_tools: [],
      codex_disabled_tools: [],
      gemini_include_tools: [],
      gemini_exclude_tools: [],
    }
    onAdd(server)
  }

  const isLoading = status === 'loading'

  return (
    <div className="flex flex-col gap-3">
      {/* Search + vetted toggle */}
      <div className="flex items-center gap-2">
        <div className="relative flex-1">
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
        <button
          onClick={() => setVettedOnly((v) => !v)}
          className={`flex items-center gap-1.5 rounded-lg border px-2.5 h-8 text-[11px] font-medium transition whitespace-nowrap ${
            vettedOnly
              ? 'border-emerald-500/30 bg-emerald-500/8 text-emerald-600 dark:text-emerald-400'
              : 'border-border bg-background text-muted-foreground hover:text-foreground'
          }`}
          title={vettedOnly ? 'Showing trusted, popular servers' : 'Showing all servers'}
          aria-pressed={vettedOnly}
        >
          <ShieldCheck className="size-3.5" />
          Vetted
        </button>
      </div>

      {/* Status bar */}
      {!isLoading && (
        <div
          className={`flex items-center gap-1.5 rounded-md px-2.5 py-1.5 text-[10px] ${
            status === 'live'
              ? 'bg-emerald-500/8 text-emerald-600 dark:text-emerald-400'
              : 'bg-muted/60 text-muted-foreground'
          }`}
        >
          {status === 'live' ? (
            <Wifi className="size-3 shrink-0" />
          ) : (
            <WifiOff className="size-3 shrink-0" />
          )}
          {status === 'live'
            ? vettedOnly
              ? 'Showing trusted, popular servers'
              : 'Live registry'
            : 'Registry unavailable -- showing curated catalog'}
          {status === 'offline' && (
            <button
              onClick={() => void fetchServers(query, vettedOnly)}
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
          <p className="py-6 text-center text-xs text-muted-foreground">
            No servers found.
          </p>
        )}

        {servers.map((s) => {
          const isAdded = addedIds.has(s.id)
          return (
            <div
              key={s.id}
              className="group flex items-start gap-2.5 rounded-xl border border-border/60 bg-card/50 p-3 transition hover:border-border hover:bg-card"
            >
              <div className="mt-0.5">
                <ServerAvatar src={s.imageUrl} name={s.name} />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 flex-wrap">
                  <p className="text-xs font-semibold truncate">{s.name}</p>
                  {s.vendor && (
                    <span className="text-[10px] text-muted-foreground">
                      by {s.vendor}
                    </span>
                  )}
                  {s.vetted && (
                    <ShieldCheck className="size-3 text-emerald-500" aria-label="Vetted server" />
                  )}
                </div>
                {s.description && (
                  <p className="mt-0.5 text-[11px] leading-snug text-muted-foreground line-clamp-2">
                    {s.description}
                  </p>
                )}
                {s.tags.length > 0 && (
                  <div className="mt-1 flex items-center gap-1 flex-wrap">
                    {s.tags.slice(0, 3).map((tag) => (
                      <span
                        key={tag}
                        className="rounded bg-muted px-1.5 py-0.5 text-[9px] text-muted-foreground"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
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
                  {isAdded ? (
                    'Added'
                  ) : (
                    <>
                      <Plus className="size-3" /> Add
                    </>
                  )}
                </button>
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
