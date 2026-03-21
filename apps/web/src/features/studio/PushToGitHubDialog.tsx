import { useState } from 'react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription,
} from '@ship/primitives'
import { Button } from '@ship/primitives'
import { Github, Loader2, CheckCircle2, ExternalLink, AlertCircle } from 'lucide-react'
import { RepoSelector, useGitHubRepos } from '#/components/github/RepoSelector'
import { ConnectGitHub } from '#/components/github/ConnectGitHub'
import type { RepoOption } from '#/components/github/RepoSelector'
import type { CompileResult } from '#/features/compiler/types'

interface PushToGitHubDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  compileOutput: Record<string, CompileResult> | null
  selectedProviders: string[]
}

interface PrResult {
  html_url: string
  number: number
}

type PushPhase =
  | { step: 'idle' }
  | { step: 'pushing' }
  | { step: 'success'; pr: PrResult }
  | { step: 'error'; message: string }

export function PushToGitHubDialog({ open, onOpenChange, compileOutput, selectedProviders }: PushToGitHubDialogProps) {
  const [selected, setSelected] = useState<RepoOption | null>(null)
  const [phase, setPhase] = useState<PushPhase>({ step: 'idle' })

  const { error: reposError, isLoading: reposLoading } = useGitHubRepos()
  const repoStatus = (reposError as { status?: number } | null)?.status
  const isUnauthenticated = !reposLoading && (repoStatus === 401 || repoStatus === 403)

  const hasOutput = compileOutput !== null && selectedProviders.length > 0

  const handleClose = (next: boolean) => {
    if (next) return
    onOpenChange(false)
    setTimeout(() => {
      setPhase({ step: 'idle' })
      setSelected(null)
    }, 200)
  }

  const handlePush = async () => {
    if (!selected || phase.step === 'pushing') return
    setPhase({ step: 'pushing' })

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
        setPhase({ step: 'error', message: msg })
        return
      }

      const pr = (await res.json()) as PrResult
      setPhase({ step: 'success', pr })
    } catch {
      setPhase({ step: 'error', message: 'Network error -- check your connection and try again.' })
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Github className="size-4" />
            Push to GitHub
          </DialogTitle>
          <DialogDescription>
            Create a pull request with your compiled Ship config in the target repository.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-2">
          {isUnauthenticated ? (
            <ConnectGitHub variant="card" />
          ) : phase.step === 'success' ? (
            <SuccessView pr={phase.pr} onReset={() => { setPhase({ step: 'idle' }); setSelected(null) }} onClose={() => handleClose(false)} />
          ) : (
            <PushForm
              selected={selected}
              onSelectRepo={setSelected}
              phase={phase}
              hasOutput={hasOutput}
              onPush={() => void handlePush()}
              onClose={() => handleClose(false)}
            />
          )}
        </div>
      </DialogContent>
    </Dialog>
  )
}

// ── Sub-components ────────────────────────────────────────────────────────────

function PushForm({ selected, onSelectRepo, phase, hasOutput, onPush, onClose }: {
  selected: RepoOption | null
  onSelectRepo: (repo: RepoOption | null) => void
  phase: PushPhase
  hasOutput: boolean
  onPush: () => void
  onClose: () => void
}) {
  return (
    <>
      <RepoSelector selected={selected} onChange={onSelectRepo} />

      {!hasOutput && (
        <p className="text-[11px] text-muted-foreground">
          Compile your agent config first to enable push.
        </p>
      )}

      {phase.step === 'error' && (
        <div className="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2.5">
          <AlertCircle className="size-3.5 text-destructive shrink-0 mt-0.5" />
          <p className="text-xs text-destructive">{phase.message}</p>
        </div>
      )}

      <div className="flex justify-end gap-2 pt-2">
        <Button variant="ghost" onClick={onClose} disabled={phase.step === 'pushing'}>
          Cancel
        </Button>
        <Button
          onClick={onPush}
          disabled={!selected || !hasOutput || phase.step === 'pushing'}
        >
          {phase.step === 'pushing' ? (
            <span className="inline-flex items-center gap-2">
              <Loader2 className="size-3.5 animate-spin" />
              Creating PR...
            </span>
          ) : (
            <span className="inline-flex items-center gap-2">
              <Github className="size-3.5" />
              Create PR
            </span>
          )}
        </Button>
      </div>
    </>
  )
}

function SuccessView({ pr, onReset, onClose }: {
  pr: PrResult
  onReset: () => void
  onClose: () => void
}) {
  return (
    <>
      <div className="flex items-start gap-3 rounded-lg border border-emerald-500/30 bg-emerald-500/5 px-4 py-3">
        <CheckCircle2 className="size-4 text-emerald-500 shrink-0 mt-0.5" />
        <div className="flex-1 min-w-0">
          <p className="text-xs font-semibold text-emerald-600 dark:text-emerald-400 mb-1">
            PR #{pr.number} created
          </p>
          <a
            href={pr.html_url}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-xs text-emerald-600 dark:text-emerald-400 hover:underline break-all"
          >
            {pr.html_url}
            <ExternalLink className="size-3 shrink-0" />
          </a>
        </div>
      </div>

      <div className="flex justify-end gap-2 pt-2">
        <button
          onClick={onReset}
          className="text-xs text-muted-foreground hover:text-foreground transition"
        >
          Push to another repo
        </button>
        <Button onClick={onClose}>Done</Button>
      </div>
    </>
  )
}
