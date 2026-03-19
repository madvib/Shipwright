import { useState } from 'react'
import { Loader2, Github, CheckCircle2, AlertCircle, ExternalLink } from 'lucide-react'
import { RepoSelector, useGitHubRepos } from './RepoSelector'
import { ConnectGitHub } from './ConnectGitHub'
import type { RepoOption } from './RepoSelector'

// ── Types ─────────────────────────────────────────────────────────────────────

interface PrResult {
  html_url: string
  number: number
}

type PushState =
  | { phase: 'idle' }
  | { phase: 'pushing' }
  | { phase: 'success'; pr: PrResult }
  | { phase: 'error'; message: string }

interface PushToGitHubProps {
  /** Whether the current user has compiled output (guard for push button) */
  hasOutput: boolean
}

// ── Component ─────────────────────────────────────────────────────────────────

export function PushToGitHub({ hasOutput }: PushToGitHubProps) {
  const [selected, setSelected] = useState<RepoOption | null>(null)
  const [pushState, setPushState] = useState<PushState>({ phase: 'idle' })

  const { error: reposError, isLoading: reposLoading } = useGitHubRepos()
  const repoStatus = (reposError as { status?: number } | null)?.status

  const isUnauthenticated = !reposLoading && (repoStatus === 401 || repoStatus === 403)

  const handlePush = async () => {
    if (!selected || pushState.phase === 'pushing') return

    setPushState({ phase: 'pushing' })

    try {
      const res = await fetch('/api/github/create-pr', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          owner: selected.owner,
          repo: selected.name,
          default_branch: selected.default_branch,
        }),
      })

      if (!res.ok) {
        let msg = `HTTP ${res.status}`
        try {
          const body = (await res.json()) as { error?: string }
          msg = body.error ?? msg
        } catch { /* non-JSON body */ }
        setPushState({ phase: 'error', message: msg })
        return
      }

      const pr = (await res.json()) as PrResult
      setPushState({ phase: 'success', pr })
    } catch {
      setPushState({ phase: 'error', message: 'Network error — check your connection and try again.' })
    }
  }

  return (
    <div className="rounded-xl border border-border/60 bg-card overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 border-b border-border/60 px-4 py-3 bg-muted/20">
        <Github className="size-3.5 text-muted-foreground" />
        <h2 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Push to GitHub</h2>
      </div>

      <div className="p-4 space-y-4">
        {/* Unauthenticated: show connect prompt */}
        {isUnauthenticated ? (
          <ConnectGitHub variant="card" />
        ) : (
          <>
            {/* Repo selector */}
            <RepoSelector selected={selected} onChange={setSelected} />

            {/* Success state */}
            {pushState.phase === 'success' && (
              <div className="flex items-start gap-3 rounded-lg border border-emerald-500/30 bg-emerald-500/5 px-3 py-3">
                <CheckCircle2 className="size-4 text-emerald-500 shrink-0 mt-0.5" />
                <div className="flex-1 min-w-0">
                  <p className="text-xs font-semibold text-emerald-600 dark:text-emerald-400 mb-1">
                    PR #{pushState.pr.number} created
                  </p>
                  <a
                    href={pushState.pr.html_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400 hover:underline break-all"
                  >
                    {pushState.pr.html_url}
                    <ExternalLink className="size-3 shrink-0" />
                  </a>
                </div>
              </div>
            )}

            {/* Error state */}
            {pushState.phase === 'error' && (
              <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2.5">
                <AlertCircle className="size-3.5 text-destructive shrink-0 mt-0.5" />
                <p className="text-xs text-destructive">{pushState.message}</p>
              </div>
            )}

            {/* Push button */}
            {pushState.phase !== 'success' && (
              <div className="flex items-center justify-between gap-3">
                {!hasOutput && (
                  <p className="text-[11px] text-muted-foreground">
                    Configure a profile first to generate files.
                  </p>
                )}
                <button
                  onClick={() => void handlePush()}
                  disabled={!selected || !hasOutput || pushState.phase === 'pushing'}
                  className="ml-auto inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-40"
                >
                  {pushState.phase === 'pushing' ? (
                    <>
                      <Loader2 className="size-3.5 animate-spin" />
                      Creating PR…
                    </>
                  ) : (
                    <>
                      <Github className="size-3.5" />
                      Push config to GitHub
                    </>
                  )}
                </button>
              </div>
            )}

            {/* Reset after success */}
            {pushState.phase === 'success' && (
              <div className="flex justify-end">
                <button
                  onClick={() => {
                    setPushState({ phase: 'idle' })
                    setSelected(null)
                  }}
                  className="text-xs text-muted-foreground hover:text-foreground transition"
                >
                  Push to another repo
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}
