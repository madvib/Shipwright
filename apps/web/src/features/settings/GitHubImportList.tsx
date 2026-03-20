import {
  Zap, FileText, Check, Upload, Loader2,
} from 'lucide-react'
import { Button } from '@ship/primitives'

// ── Types ────────────────────────────────────────────────────────────────────

export interface GitHubRepo {
  id: number
  full_name: string
  name: string
  owner: { login: string }
  description: string | null
  private: boolean
  detected_configs?: string[]
  imported?: boolean
  import_pr_number?: number | null
}

export type RepoState = 'detected' | 'no-config' | 'imported'

export function getRepoState(repo: GitHubRepo): RepoState {
  if (repo.imported) return 'imported'
  if (repo.detected_configs && repo.detected_configs.length > 0)
    return 'detected'
  return 'no-config'
}

// ── Repo row ─────────────────────────────────────────────────────────────────

export function RepoRow({
  repo,
  isImporting,
  onImportPr,
}: {
  repo: GitHubRepo
  isImporting: boolean
  onImportPr: () => void
}) {
  const state = getRepoState(repo)

  return (
    <div
      className={`flex items-center gap-3 rounded-xl border p-3 transition ${
        state === 'detected'
          ? 'border-primary/30 bg-primary/5'
          : state === 'imported'
            ? 'border-border/40 bg-card opacity-60'
            : 'border-border/60 bg-card'
      }`}
    >
      <RepoIcon state={state} />

      <div className="flex-1 min-w-0">
        <div
          className={`text-[13px] font-medium ${
            state === 'imported'
              ? 'text-muted-foreground'
              : state === 'detected'
                ? 'text-foreground'
                : 'text-foreground/70'
          }`}
        >
          {repo.full_name}
        </div>
        <RepoSubline repo={repo} state={state} />
      </div>

      <RepoAction
        state={state}
        isImporting={isImporting}
        onImportPr={onImportPr}
      />
    </div>
  )
}

function RepoIcon({ state }: { state: RepoState }) {
  if (state === 'detected') {
    return (
      <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/15">
        <Zap className="size-4 text-primary" />
      </div>
    )
  }
  if (state === 'imported') {
    return (
      <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-emerald-500/15">
        <Check className="size-4 text-emerald-500" />
      </div>
    )
  }
  return (
    <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-muted/40">
      <FileText className="size-4 text-muted-foreground" />
    </div>
  )
}

function RepoSubline({
  repo,
  state,
}: {
  repo: GitHubRepo
  state: RepoState
}) {
  if (state === 'detected' && repo.detected_configs) {
    return (
      <div className="mt-0.5 text-[10px] text-primary">
        {repo.detected_configs.join(' + ')} detected
      </div>
    )
  }
  if (state === 'imported') {
    return (
      <div className="mt-0.5 text-[10px] text-emerald-600 dark:text-emerald-400">
        Imported
        {repo.import_pr_number != null &&
          ` · PR #${repo.import_pr_number} merged`}
      </div>
    )
  }
  return (
    <div className="mt-0.5 text-[10px] text-muted-foreground">
      No agent config detected
    </div>
  )
}

function RepoAction({
  state,
  isImporting,
  onImportPr,
}: {
  state: RepoState
  isImporting: boolean
  onImportPr: () => void
}) {
  if (state === 'imported') {
    return (
      <span className="text-[10px] text-emerald-600 dark:text-emerald-400">
        Done
      </span>
    )
  }

  if (state === 'detected') {
    return (
      <Button size="sm" onClick={onImportPr} disabled={isImporting}>
        {isImporting ? (
          <Loader2 className="size-3 animate-spin" />
        ) : (
          <Upload className="size-3" />
        )}
        Import & PR
      </Button>
    )
  }

  return (
    <Button variant="outline" size="xs" onClick={onImportPr}>
      Add Ship <span className="size-2 rounded-full bg-primary" />
    </Button>
  )
}

// ── Registry CTA ─────────────────────────────────────────────────────────────

export function RegistryCta() {
  return (
    <div className="mt-5 rounded-xl border border-border/60 bg-card p-4 text-center">
      <div className="text-[13px] font-medium text-foreground/80">
        Built something great?
      </div>
      <div className="mt-1 text-[11px] text-muted-foreground">
        Submit your agents and skills to the Ship Registry for the community to
        use.
      </div>
      <div className="mt-3 flex justify-center">
        <Button variant="outline" size="sm">
          <Upload className="size-3" />
          Publish to registry
        </Button>
      </div>
    </div>
  )
}
