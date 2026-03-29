// Git tab content for the Session sidebar.
// Shows working tree status, modified files, and recent commits.

import { GitCompareArrows, Circle, MessageSquare, FileText } from 'lucide-react'
import type { GitStatusResult, GitLogEntry } from './useGitInfo'

interface GitTabProps {
  gitStatus: GitStatusResult | null | undefined
  gitLog: GitLogEntry[] | null | undefined
  onShowDiff: () => void
  onSelectCommit: (hash: string) => void
}

function countChangedFiles(gitStatus: GitStatusResult): number {
  return (gitStatus.staged?.length ?? 0) + (gitStatus.modified?.length ?? 0) + (gitStatus.untracked?.length ?? 0)
}

export function GitTab({ gitStatus, gitLog, onShowDiff, onSelectCommit }: GitTabProps) {
  const totalChanged = gitStatus ? countChangedFiles(gitStatus) : 0

  return (
    <div className="px-3 pt-3">
      {gitStatus && (
        <div className="mb-3">
          <div className="flex items-center gap-2 mb-2.5">
            <span className="text-xs font-medium font-mono">{gitStatus.branch}</span>
            <Circle className={`size-1.5 shrink-0 ${gitStatus.clean ? 'fill-emerald-500 text-emerald-500' : 'fill-amber-500 text-amber-500'}`} />
            {totalChanged > 0 && (
              <span className="ml-auto flex items-center gap-1 text-[10px] text-muted-foreground">
                <FileText className="size-3" />
                {totalChanged} file{totalChanged !== 1 ? 's' : ''} changed
              </span>
            )}
          </div>
          {!gitStatus.clean && (
            <>
              <div className="flex gap-3 text-[10px] text-muted-foreground mb-2.5">
                {(gitStatus.staged?.length ?? 0) > 0 && (
                  <span className="flex items-center gap-1"><Circle className="size-1.5 fill-emerald-500 text-emerald-500" />{gitStatus.staged!.length} staged</span>
                )}
                {(gitStatus.modified?.length ?? 0) > 0 && (
                  <span className="flex items-center gap-1"><Circle className="size-1.5 fill-amber-500 text-amber-500" />{gitStatus.modified!.length} modified</span>
                )}
                {(gitStatus.untracked?.length ?? 0) > 0 && (
                  <span className="flex items-center gap-1"><Circle className="size-1.5 fill-red-500 text-red-500" />{gitStatus.untracked!.length} untracked</span>
                )}
              </div>
              <div className="space-y-0.5 mb-2">
                {[
                  ...((gitStatus.staged ?? []) as unknown[]).map((f) => ({ f, badge: 'S', cls: 'bg-emerald-500/10 text-emerald-500' })),
                  ...((gitStatus.modified ?? []) as unknown[]).map((f) => ({ f, badge: 'M', cls: 'bg-amber-500/10 text-amber-500' })),
                  ...((gitStatus.untracked ?? []) as unknown[]).map((f) => ({ f, badge: '?', cls: 'bg-red-500/10 text-red-500' })),
                ].map(({ f, badge, cls }, i) => {
                  const p = typeof f === 'string' ? f : (f as { path?: string })?.path ?? ''
                  return (
                    <button
                      key={p || i}
                      type="button"
                      onClick={() => onShowDiff()}
                      className="flex items-center gap-2 px-2 py-1 rounded text-xs text-muted-foreground hover:text-foreground hover:bg-muted/30 cursor-pointer transition w-full text-left"
                    >
                      <span className={`text-[9px] font-mono px-1 rounded font-bold ${cls}`}>{badge}</span>
                      <span className="truncate">{p.split('/').pop()}</span>
                    </button>
                  )
                })}
              </div>
            </>
          )}
          {/* View Diff button: always visible when there are any changes */}
          {totalChanged > 0 && (
            <button onClick={onShowDiff} className="w-full flex items-center justify-center gap-1.5 text-[10px] text-primary font-medium py-1.5 rounded-md border border-primary/20 hover:bg-primary/5 transition">
              <GitCompareArrows className="size-3" />View diff
            </button>
          )}
        </div>
      )}
      {gitLog && gitLog.length > 0 && (
        <>
          {gitStatus && <div className="border-t border-border/40 mb-3" />}
          <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Recent Commits</div>
          <div className="space-y-1">
            {gitLog.slice(0, 10).map((entry, i) => {
              const msg = entry.message ?? entry.subject ?? ''
              const shortHash = entry.short_hash ?? (entry.hash ?? '').slice(0, 7)
              const fullHash = entry.hash ?? ''
              return (
              <button
                key={fullHash || i}
                type="button"
                onClick={() => fullHash && onSelectCommit(fullHash)}
                className="group cursor-pointer rounded-md px-2 py-2 -mx-1 hover:bg-muted/30 transition w-full text-left"
              >
                <div className="text-sm text-muted-foreground group-hover:text-foreground transition leading-snug">
                  {msg ? (msg.length > 60 ? msg.slice(0, 60) + '...' : msg) : <span className="italic text-muted-foreground/50">No message</span>}
                </div>
                <div className="flex items-center gap-2 mt-1 text-[10px] text-muted-foreground/50">
                  <span className="font-mono">{shortHash}</span>
                  <span>{entry.date ?? ''}</span>
                  <div className="flex-1" />
                  <MessageSquare className="size-3 opacity-0 group-hover:opacity-60 transition-opacity" />
                </div>
              </button>
              )
            })}
          </div>
        </>
      )}
      {!gitStatus && <p className="text-[10px] text-muted-foreground/60">Connect CLI to see git info.</p>}
    </div>
  )
}
