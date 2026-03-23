import { useQuery } from '@tanstack/react-query'
import { githubKeys } from '#/lib/query-keys'
import { fetchApi } from '#/lib/api-errors'
import { Github, Lock, Unlock } from 'lucide-react'
import { useState, useCallback } from 'react'
import { authClient } from '#/lib/auth-client'

// ── Types ─────────────────────────────────────────────────────────────────────

export interface RepoOption {
  owner: string
  name: string
  full_name: string
  default_branch: string
  private: boolean
}

interface ReposResponse {
  user: { login: string; avatar_url: string }
  repos: Array<{
    owner: string
    name: string
    full_name: string
    default_branch: string
    private: boolean
    description: string | null
  }>
}

export type GitHubConnectionState =
  | { status: 'loading' }
  | { status: 'unauthenticated' }
  | { status: 'error'; message: string }
  | { status: 'ready'; repos: RepoOption[]; user: { login: string; avatar_url: string } }

// ── Hook ──────────────────────────────────────────────────────────────────────

export function useGitHubRepos() {
  return useQuery<ReposResponse, { status: number; message: string }>({
    queryKey: githubKeys.repos(),
    queryFn: () => fetchApi<ReposResponse>('/api/github/repos', { credentials: 'include' }),
    retry: false,
    staleTime: 60_000,
  })
}

// ── Component ─────────────────────────────────────────────────────────────────

interface RepoSelectorProps {
  selected: RepoOption | null
  onChange: (repo: RepoOption | null) => void
}

export function RepoSelector({ selected, onChange }: RepoSelectorProps) {
  const { data, isLoading, error } = useGitHubRepos()
  const [extraRepos, setExtraRepos] = useState<RepoOption[]>([])
  const [page, setPage] = useState(1)
  const [loadingMore, setLoadingMore] = useState(false)
  const [hasMore, setHasMore] = useState(true)

  const status = (error as { status?: number } | null)?.status

  const handleReconnect = useCallback(() => {
    authClient.signIn.social({
      provider: 'github',
      callbackURL: window.location.pathname,
      scopes: ['repo', 'user:email'],
    })
  }, [])

  const handleLoadMore = useCallback(async () => {
    const nextPage = page + 1
    setLoadingMore(true)
    try {
      const moreData = await fetchApi<ReposResponse>(
        `/api/github/repos?page=${nextPage}`,
        { credentials: 'include' },
      )
      const moreRepos = moreData.repos.map<RepoOption>((r) => ({
        owner: r.owner,
        name: r.name,
        full_name: r.full_name,
        default_branch: r.default_branch,
        private: r.private,
      }))
      setExtraRepos((prev) => [...prev, ...moreRepos])
      setPage(nextPage)
      if (moreData.repos.length < 30) {
        setHasMore(false)
      }
    } catch {
      // silently fail — user can try again
    } finally {
      setLoadingMore(false)
    }
  }, [page])

  if (isLoading) {
    return (
      <div className="space-y-2">
        <div className="h-3.5 w-28 rounded bg-muted animate-pulse" />
        <div className="h-9 w-full rounded-lg bg-muted animate-pulse" />
      </div>
    )
  }

  if (status === 401 || status === 403) {
    return (
      <div className="space-y-2">
        <p className="text-xs text-muted-foreground">GitHub connection expired.</p>
        <button
          onClick={handleReconnect}
          className="inline-flex items-center gap-1.5 rounded-lg bg-foreground px-3 py-1.5 text-[11px] font-semibold text-background transition hover:opacity-80"
        >
          <Github className="size-3" />
          Reconnect GitHub
        </button>
      </div>
    )
  }

  if (error || !data) {
    const msg = (error as { message?: string } | null)?.message ?? 'Failed to load repositories'
    return (
      <p className="text-xs text-destructive">{msg}</p>
    )
  }

  const baseRepos = data.repos.map<RepoOption>((r) => ({
    owner: r.owner,
    name: r.name,
    full_name: r.full_name,
    default_branch: r.default_branch,
    private: r.private,
  }))
  const repos = [...baseRepos, ...extraRepos]

  // Determine if initial page might have more
  const showLoadMore = hasMore && (baseRepos.length === 30 || extraRepos.length > 0)

  if (repos.length === 0) {
    return (
      <div className="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/20 px-3 py-2.5">
        <Github className="size-3.5 text-muted-foreground shrink-0" />
        <p className="text-xs text-muted-foreground">No repositories found on this account.</p>
      </div>
    )
  }

  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const val = e.target.value
    if (!val) {
      onChange(null)
      return
    }
    const repo = repos.find((r) => r.full_name === val) ?? null
    onChange(repo)
  }

  return (
    <div className="space-y-1.5">
      <label className="block text-xs font-medium text-muted-foreground">
        Repository
      </label>
      <div className="relative">
        <select
          value={selected?.full_name ?? ''}
          onChange={handleChange}
          className="w-full appearance-none rounded-lg border border-border/60 bg-background/60 pl-8 pr-8 py-2 text-sm text-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/30 transition"
        >
          <option value="">Select a repository...</option>
          {repos.map((r) => (
            <option key={r.full_name} value={r.full_name}>
              {r.full_name}
            </option>
          ))}
        </select>

        {/* Left icon */}
        <span className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground">
          {selected?.private === true ? (
            <Lock className="size-3.5" />
          ) : selected ? (
            <Unlock className="size-3.5" />
          ) : (
            <Github className="size-3.5" />
          )}
        </span>

        {/* Chevron */}
        <span className="pointer-events-none absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground text-[10px]">
          &#9662;
        </span>
      </div>

      {showLoadMore && (
        <button
          onClick={() => void handleLoadMore()}
          disabled={loadingMore}
          className="text-[11px] text-primary hover:underline disabled:opacity-50"
        >
          {loadingMore ? 'Loading...' : 'Load more repositories'}
        </button>
      )}

      {selected && (
        <p className="text-[11px] text-muted-foreground">
          Default branch: <code className="font-mono">{selected.default_branch}</code>
        </p>
      )}
    </div>
  )
}
