import { createFileRoute, Link } from '@tanstack/react-router'
import { useState, useMemo } from 'react'
import { Github, Search, ArrowLeft, Loader2 } from 'lucide-react'
import { Button } from '@ship/primitives'
import { authClient } from '#/lib/auth-client'
import {
  RepoRow, RegistryCta, getRepoState,
} from '#/features/settings/GitHubImportList'
import type { GitHubRepo } from '#/features/settings/GitHubImportList'

export const Route = createFileRoute('/studio/import')({ component: ImportPage, ssr: false })

function ImportPage() {
  const { data: session } = authClient.useSession()
  const isConnected = !!session?.user
  const [filter, setFilter] = useState('')
  const [repos, setRepos] = useState<GitHubRepo[]>([])
  const [loading, setLoading] = useState(false)
  const [fetched, setFetched] = useState(false)
  const [importingId, setImportingId] = useState<number | null>(null)

  const fetchRepos = async () => {
    setLoading(true)
    try {
      const res = await fetch('/api/github/repos')
      if (res.ok) {
        const data = (await res.json()) as GitHubRepo[]
        setRepos(data)
      }
    } catch {
      /* ignore */
    } finally {
      setLoading(false)
      setFetched(true)
    }
  }

  const handleImportPr = async (repo: GitHubRepo) => {
    setImportingId(repo.id)
    try {
      const res = await fetch('/api/github/create-pr', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ owner: repo.owner.login, repo: repo.name }),
      })
      if (res.ok) {
        const result = (await res.json()) as { pr_number?: number }
        setRepos((prev) =>
          prev.map((r) =>
            r.id === repo.id
              ? { ...r, imported: true, import_pr_number: result.pr_number ?? null }
              : r,
          ),
        )
      }
    } catch {
      /* ignore */
    } finally {
      setImportingId(null)
    }
  }

  const sortedRepos = useMemo(() => {
    const order = { detected: 0, 'no-config': 1, imported: 2 } as const
    const filtered = filter.trim()
      ? repos.filter((r) =>
          r.full_name.toLowerCase().includes(filter.toLowerCase()),
        )
      : repos
    return [...filtered].sort(
      (a, b) => order[getRepoState(a)] - order[getRepoState(b)],
    )
  }, [repos, filter])

  return (
    <div className="mx-auto max-w-[640px] px-5 py-6 pb-24">
      <div className="mb-2">
        <Link
          to="/"
          className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition mb-3"
        >
          <ArrowLeft className="size-3" />
          Back
        </Link>
      </div>

      <div className="mb-5">
        <h1 className="font-display text-xl font-bold text-foreground">
          Import from GitHub
        </h1>
        <p className="text-[13px] text-muted-foreground">
          Select repos to convert. Ship will create a PR adding .ship/ config.
        </p>
      </div>

      {!isConnected ? (
        <NotConnectedState />
      ) : (
        <>
          <ConnectedHeader
            userName={session?.user?.name || 'Connected'}
            filter={filter}
            onFilterChange={setFilter}
          />

          {!fetched && !loading && (
            <div className="mt-6 text-center">
              <Button onClick={() => void fetchRepos()}>
                Load repositories
              </Button>
            </div>
          )}

          {loading && (
            <div className="mt-10 flex items-center justify-center gap-2 text-sm text-muted-foreground">
              <Loader2 className="size-4 animate-spin" />
              Loading repositories...
            </div>
          )}

          {fetched && !loading && (
            <div className="mt-4 flex flex-col gap-1.5">
              {sortedRepos.length === 0 ? (
                <p className="py-10 text-center text-sm text-muted-foreground">
                  {filter ? 'No repos match your filter.' : 'No repositories found.'}
                </p>
              ) : (
                sortedRepos.map((repo) => (
                  <RepoRow
                    key={repo.id}
                    repo={repo}
                    isImporting={importingId === repo.id}
                    onImportPr={() => void handleImportPr(repo)}
                  />
                ))
              )}
            </div>
          )}

          <RegistryCta />
        </>
      )}
    </div>
  )
}

// ── Sub-components ───────────────────────────────────────────────────────────

function NotConnectedState() {
  return (
    <div className="flex flex-col items-center gap-4 py-16 text-center">
      <div className="flex size-14 items-center justify-center rounded-2xl border border-border/60 bg-muted/40">
        <Github className="size-6 text-muted-foreground" />
      </div>
      <div>
        <p className="text-sm font-semibold text-foreground">
          Connect GitHub to get started
        </p>
        <p className="mt-1 text-xs text-muted-foreground max-w-xs">
          We'll scan your repos for existing agent configs and help you convert
          them to Ship format.
        </p>
      </div>
      <Button
        onClick={() => void authClient.signIn.social({ provider: 'github' })}
        className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground"
      >
        <Github className="size-4" />
        Connect GitHub
      </Button>
    </div>
  )
}

function ConnectedHeader({
  userName,
  filter,
  onFilterChange,
}: {
  userName: string
  filter: string
  onFilterChange: (v: string) => void
}) {
  return (
    <div
      className="flex items-center gap-2.5 rounded-xl border border-border/60 bg-card p-3"
      style={{
        background:
          'linear-gradient(135deg, color-mix(in oklch, var(--card) 90%, var(--primary) 10%), var(--card))',
      }}
    >
      <div className="flex size-7 shrink-0 items-center justify-center rounded-lg bg-muted/60">
        <Github className="size-3.5 text-foreground" />
      </div>
      <div className="flex-1 min-w-0">
        <div className="text-xs font-medium text-foreground">{userName}</div>
        <div className="flex items-center gap-1 text-[10px] text-emerald-600 dark:text-emerald-400">
          <span className="size-1.5 rounded-full bg-emerald-500" />
          Connected
        </div>
      </div>
      <div className="flex flex-1 items-center gap-1.5 rounded-md border border-border/60 bg-background/60 px-2.5 py-1.5">
        <Search className="size-3 shrink-0 text-muted-foreground" />
        <input
          value={filter}
          onChange={(e) => onFilterChange(e.target.value)}
          placeholder="Filter repos..."
          className="flex-1 bg-transparent text-[11px] text-foreground placeholder:text-muted-foreground focus:outline-none min-w-0"
        />
      </div>
    </div>
  )
}
