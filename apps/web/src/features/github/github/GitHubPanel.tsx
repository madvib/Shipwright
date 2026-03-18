import { useState, useEffect, useCallback } from 'react'
import { Github, Lock, Unlock, Loader2, ExternalLink, GitPullRequest, AlertCircle, ChevronRight } from 'lucide-react'
import type { ProjectLibrary } from '#/features/compiler/types'

// ── Types ────────────────────────────────────────────────────────────────────

interface GhUser {
  login: string
  avatar_url: string
}

interface GhRepo {
  full_name: string
  name: string
  owner: string
  private: boolean
  default_branch: string
  description: string | null
}

type GhState =
  | { step: 'disconnected' }
  | { step: 'loading' }
  | { step: 'connected'; user: GhUser; repos: GhRepo[] }
  | { step: 'importing'; repo: GhRepo }
  | { step: 'pr-created'; repo: GhRepo; prUrl: string }
  | { step: 'error'; message: string }

// ── Component ────────────────────────────────────────────────────────────────

interface GitHubPanelProps {
  modeName: string
  onImport: (library: ProjectLibrary) => void
}

export function GitHubPanel({ modeName, onImport }: GitHubPanelProps) {
  const [state, setState] = useState<GhState>({ step: 'disconnected' })
  const [search, setSearch] = useState('')

  // Check connection status on mount (and after OAuth redirect)
  useEffect(() => {
    const params = new URLSearchParams(window.location.search)
    if (params.has('gh_error')) {
      setState({ step: 'error', message: params.get('gh_error')! })
      cleanUrl()
      return
    }
    if (params.has('gh_connected')) {
      cleanUrl()
    }
    // Try fetching repos — if cookie exists, we're connected
    void checkConnection()
  }, [])

  const checkConnection = useCallback(async () => {
    setState({ step: 'loading' })
    try {
      const res = await fetch('/api/github/repos')
      if (res.status === 401) {
        setState({ step: 'disconnected' })
        return
      }
      if (!res.ok) throw new Error('Failed to fetch repos')
      const data = (await res.json()) as { user: GhUser; repos: GhRepo[] }
      setState({ step: 'connected', user: data.user, repos: data.repos })
    } catch {
      setState({ step: 'disconnected' })
    }
  }, [])

  const handleImportAndPr = useCallback(async (repo: GhRepo) => {
    setState({ step: 'importing', repo })
    try {
      const res = await fetch('/api/github/create-pr', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          owner: repo.owner,
          repo: repo.name,
          default_branch: repo.default_branch,
          mode_name: modeName,
        }),
      })
      if (!res.ok) {
        const err = (await res.json()) as { error: string }
        throw new Error(err.error)
      }
      const pr = (await res.json()) as { html_url: string }
      setState({ step: 'pr-created', repo, prUrl: pr.html_url })

      // Also import the config into the Studio editor
      const importRes = await fetch('/api/github/import', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: `https://github.com/${repo.full_name}` }),
      })
      if (importRes.ok) {
        const lib = (await importRes.json()) as ProjectLibrary
        onImport(lib)
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : 'Failed to create PR'
      setState({ step: 'error', message: msg })
    }
  }, [modeName, onImport])

  // ── Render ───────────────────────────────────────────────────────────────

  if (state.step === 'disconnected') {
    return (
      <div className="flex flex-col items-center gap-3 py-4">
        <p className="text-[11px] text-muted-foreground text-center">
          Connect GitHub to import from private repos and create config PRs.
        </p>
        <a
          href="/api/github/oauth"
          className="inline-flex items-center gap-2 rounded-lg bg-[#24292f] px-4 py-2 text-xs font-semibold text-white transition hover:bg-[#32383f]"
        >
          <Github className="size-3.5" />
          Connect GitHub
        </a>
      </div>
    )
  }

  if (state.step === 'loading') {
    return (
      <div className="flex items-center justify-center py-6">
        <Loader2 className="size-4 animate-spin text-muted-foreground" />
      </div>
    )
  }

  if (state.step === 'error') {
    return (
      <div className="flex flex-col items-center gap-3 py-4">
        <div className="flex items-center gap-2 text-destructive">
          <AlertCircle className="size-3.5" />
          <p className="text-[11px] font-medium">{state.message}</p>
        </div>
        <button
          onClick={() => setState({ step: 'disconnected' })}
          className="text-[11px] text-primary hover:underline"
        >
          Try again
        </button>
      </div>
    )
  }

  if (state.step === 'importing') {
    return (
      <div className="flex flex-col items-center gap-2 py-6">
        <Loader2 className="size-4 animate-spin text-muted-foreground" />
        <p className="text-[11px] text-muted-foreground">
          Creating PR on {state.repo.full_name}...
        </p>
      </div>
    )
  }

  if (state.step === 'pr-created') {
    return (
      <div className="flex flex-col items-center gap-3 py-4">
        <div className="flex size-8 items-center justify-center rounded-full bg-emerald-500/10">
          <GitPullRequest className="size-4 text-emerald-600 dark:text-emerald-400" />
        </div>
        <div className="text-center">
          <p className="text-xs font-medium text-foreground">PR created</p>
          <p className="mt-0.5 text-[11px] text-muted-foreground">{state.repo.full_name}</p>
        </div>
        <a
          href={state.prUrl}
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-xs font-semibold text-primary-foreground transition hover:opacity-90"
        >
          View PR
          <ExternalLink className="size-3" />
        </a>
        <button
          onClick={() => void checkConnection()}
          className="text-[11px] text-muted-foreground hover:text-foreground"
        >
          Back to repos
        </button>
      </div>
    )
  }

  // step === 'connected' — repo picker
  const { user, repos } = state
  const filtered = repos.filter(
    (r) =>
      !search ||
      r.full_name.toLowerCase().includes(search.toLowerCase()) ||
      (r.description ?? '').toLowerCase().includes(search.toLowerCase()),
  )

  return (
    <div className="flex flex-col gap-2">
      {/* User badge */}
      <div className="flex items-center gap-2 px-1">
        <img src={user.avatar_url} alt="" className="size-5 rounded-full" />
        <span className="text-[11px] font-medium text-foreground">{user.login}</span>
      </div>

      {/* Search */}
      <input
        value={search}
        onChange={(e) => setSearch(e.target.value)}
        placeholder="Filter repos..."
        className="rounded-md border border-border/60 bg-background/60 px-2.5 py-1.5 text-[11px] text-foreground placeholder:text-muted-foreground focus:outline-none focus:border-border"
      />

      {/* Repo list */}
      <div className="max-h-60 overflow-y-auto -mx-1">
        {filtered.map((repo) => (
          <button
            key={repo.full_name}
            onClick={() => void handleImportAndPr(repo)}
            className="group flex w-full items-center gap-2 rounded-md px-2 py-2 text-left hover:bg-muted/50 transition"
          >
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-1.5">
                {repo.private ? (
                  <Lock className="size-3 text-amber-500 shrink-0" />
                ) : (
                  <Unlock className="size-3 text-muted-foreground shrink-0" />
                )}
                <span className="text-[11px] font-medium text-foreground truncate">{repo.full_name}</span>
              </div>
              {repo.description && (
                <p className="text-[10px] text-muted-foreground truncate mt-0.5 ml-[18px]">{repo.description}</p>
              )}
            </div>
            <ChevronRight className="size-3 text-muted-foreground opacity-0 group-hover:opacity-100 shrink-0" />
          </button>
        ))}
        {filtered.length === 0 && (
          <p className="px-2 py-3 text-[11px] text-muted-foreground text-center">No repos found</p>
        )}
      </div>
    </div>
  )
}

function cleanUrl() {
  const url = new URL(window.location.href)
  url.searchParams.delete('gh_connected')
  url.searchParams.delete('gh_error')
  window.history.replaceState({}, '', url.pathname)
}
