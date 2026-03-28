// Diff tab content: branch info, clean/dirty indicator, diff button, commit log.

import { GitCompareArrows, GitBranch, Circle } from 'lucide-react'
import { useGitStatus, useGitLog } from './useGitInfo'

interface DiffTabContentProps {
  hasDiff: boolean
  isActive: boolean
  onShowDiff: () => void
}

export function DiffTabContent({ hasDiff, isActive, onShowDiff }: DiffTabContentProps) {
  const { data: gitStatus } = useGitStatus()
  const { data: gitLog } = useGitLog(5)

  const modifiedCount = (gitStatus?.modified?.length ?? 0) + (gitStatus?.staged?.length ?? 0)
  const untrackedCount = gitStatus?.untracked?.length ?? 0

  return (
    <div className="space-y-3">
      {/* Branch info */}
      {gitStatus && (
        <div className="px-1.5 py-1.5 rounded-md bg-muted/30">
          <div className="flex items-center gap-1.5">
            <GitBranch className="size-3 text-muted-foreground shrink-0" />
            <span className="text-[11px] font-mono font-medium text-foreground truncate">{gitStatus.branch}</span>
            <Circle className={`size-1.5 shrink-0 ${gitStatus.clean ? 'fill-emerald-500 text-emerald-500' : 'fill-amber-500 text-amber-500'}`} />
          </div>
          {!gitStatus.clean && (
            <div className="flex items-center gap-2 mt-1 px-[18px]">
              {modifiedCount > 0 && (
                <span className="text-[9px] text-muted-foreground">
                  {modifiedCount} modified
                </span>
              )}
              {untrackedCount > 0 && (
                <span className="text-[9px] text-muted-foreground">
                  {untrackedCount} untracked
                </span>
              )}
            </div>
          )}
        </div>
      )}

      {/* View diff button */}
      {hasDiff ? (
        <button
          onClick={onShowDiff}
          className={`flex items-center gap-1.5 w-full px-2 py-1.5 text-[11px] font-medium rounded-md transition ${
            isActive ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
          }`}
        >
          <GitCompareArrows className="size-3" />
          {isActive ? 'Viewing diff' : 'View diff'}
        </button>
      ) : (
        <div className="text-center py-3 px-2">
          <GitCompareArrows className="size-5 mx-auto mb-1.5 text-muted-foreground/40" />
          <p className="text-[10px] text-muted-foreground">
            No diff available. Run <code className="text-[9px] bg-muted px-1 rounded">/diff</code> to generate one.
          </p>
        </div>
      )}

      {/* Mini commit log */}
      {gitLog && gitLog.length > 0 && (
        <div>
          <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">Recent commits</p>
          <div className="space-y-px">
            {gitLog.map((entry) => (
              <div key={entry.hash} className="flex items-start gap-1.5 px-1.5 py-1 rounded hover:bg-muted/30">
                <span className="text-[9px] font-mono text-muted-foreground/60 shrink-0 mt-px">{entry.hash.slice(0, 7)}</span>
                <span className="text-[10px] text-foreground leading-tight truncate">{entry.subject}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
