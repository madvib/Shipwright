// Right panel for Session page: git status, changed files, recent commits, worktrees.
// Collapsible via the parent. Shows at-a-glance session awareness.

import { PanelRightClose, GitBranch, Circle } from 'lucide-react'
import type { GitStatusResult, GitLogEntry } from './useGitInfo'

interface SessionInfoPanelProps {
  gitStatus: GitStatusResult | null | undefined
  gitLog: GitLogEntry[] | null | undefined
  onClose: () => void
  onShowDiff: () => void
  onSelectCommit: (hash: string) => void
}

export function SessionInfoPanel({ gitStatus, gitLog, onClose, onShowDiff, onSelectCommit }: SessionInfoPanelProps) {
  const modifiedCount = (gitStatus?.modified?.length ?? 0) + (gitStatus?.staged?.length ?? 0)
  const untrackedCount = gitStatus?.untracked?.length ?? 0
  const toPath = (f: unknown): string => (typeof f === 'string' ? f : (f as { path?: string })?.path ?? '')
  const allChanged = [
    ...(gitStatus?.staged ?? []).map((f) => ({ path: toPath(f), status: 'S' as const })),
    ...(gitStatus?.modified ?? []).map((f) => ({ path: toPath(f), status: 'M' as const })),
    ...(gitStatus?.untracked ?? []).map((f) => ({ path: toPath(f), status: 'A' as const })),
  ]

  return (
    <aside className="flex w-72 shrink-0 flex-col border-l border-border bg-card/10">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border shrink-0">
        <span className="text-sm font-semibold">Session</span>
        <button
          onClick={onClose}
          className="flex size-6 items-center justify-center rounded text-muted-foreground hover:text-foreground hover:bg-muted/30 transition"
          aria-label="Close panel"
        >
          <PanelRightClose className="size-3.5" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Branch & status */}
        {gitStatus && (
          <div className="px-4 py-3 border-b border-border/40">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
              Branch
            </div>
            <div className="flex items-center gap-2 mb-2">
              <GitBranch className="size-4 text-muted-foreground" />
              <span className="text-sm font-medium font-mono">{gitStatus.branch}</span>
              <span
                className={`text-[9px] px-1.5 py-0.5 rounded font-medium ${
                  gitStatus.clean
                    ? 'bg-emerald-500/10 text-emerald-500'
                    : 'bg-amber-500/10 text-amber-500'
                }`}
              >
                {gitStatus.clean ? 'clean' : 'dirty'}
              </span>
            </div>
            {!gitStatus.clean && (
              <div className="flex gap-3 text-[10px] text-muted-foreground">
                {(gitStatus.staged?.length ?? 0) > 0 && (
                  <span className="flex items-center gap-1">
                    <Circle className="size-1.5 fill-emerald-500 text-emerald-500" />
                    {gitStatus.staged!.length} staged
                  </span>
                )}
                {modifiedCount > 0 && (
                  <span className="flex items-center gap-1">
                    <Circle className="size-1.5 fill-amber-500 text-amber-500" />
                    {modifiedCount} modified
                  </span>
                )}
                {untrackedCount > 0 && (
                  <span className="flex items-center gap-1">
                    <Circle className="size-1.5 fill-red-500 text-red-500" />
                    {untrackedCount} untracked
                  </span>
                )}
              </div>
            )}
          </div>
        )}

        {/* Changed files */}
        {allChanged.length > 0 && (
          <div className="px-4 py-3 border-b border-border/40">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
              Changed Files
            </div>
            <div className="space-y-1">
              {allChanged.slice(0, 8).map(({ path, status }) => (
                <div
                  key={path}
                  className="flex items-center gap-2 py-1 text-xs group cursor-pointer"
                >
                  <span
                    className={`text-[9px] font-mono px-1 rounded font-bold ${
                      status === 'S'
                        ? 'bg-emerald-500/10 text-emerald-500'
                        : status === 'A'
                          ? 'bg-red-500/10 text-red-500'
                          : 'bg-amber-500/10 text-amber-500'
                    }`}
                  >
                    {status}
                  </span>
                  <span className="text-muted-foreground group-hover:text-foreground truncate transition">
                    {path.split('/').pop()}
                  </span>
                </div>
              ))}
              {allChanged.length > 8 && (
                <div className="text-[10px] text-muted-foreground/50 py-1">
                  +{allChanged.length - 8} more
                </div>
              )}
            </div>
            <button
              onClick={onShowDiff}
              className="mt-2 w-full text-center text-[10px] text-primary font-medium py-1.5 rounded hover:bg-primary/5 transition"
            >
              View full diff
            </button>
          </div>
        )}

        {/* Recent commits */}
        {gitLog && gitLog.length > 0 && (
          <div className="px-4 py-3">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
              Recent Commits
            </div>
            <div className="space-y-2">
              {gitLog.slice(0, 5).map((entry) => (
                <div
                  key={entry.hash}
                  onClick={() => onSelectCommit(entry.hash)}
                  className="group cursor-pointer rounded px-1 -mx-1 py-0.5 hover:bg-muted/30 transition"
                >
                  <div className="flex items-center gap-2 text-xs">
                    <span className="font-mono text-[9px] text-primary/60">
                      {entry.hash.slice(0, 7)}
                    </span>
                    <span className="truncate text-muted-foreground group-hover:text-foreground transition">
                      {entry.subject}
                    </span>
                  </div>
                  <div className="text-[9px] text-muted-foreground/40 mt-0.5">
                    {entry.date}
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </aside>
  )
}
