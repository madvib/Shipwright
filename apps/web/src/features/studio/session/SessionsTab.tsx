// Sessions tab content for the Session sidebar.
// Shows current workspace info from git status when connected.

import { Circle, GitBranch, FolderClosed } from 'lucide-react'
import type { GitStatusResult } from './useGitInfo'

interface SessionsTabProps {
  isConnected: boolean
  gitStatus: GitStatusResult | null | undefined
}

export function SessionsTab({ isConnected, gitStatus }: SessionsTabProps) {
  return (
    <div className="px-3 pt-3 text-xs text-muted-foreground">
      {isConnected && gitStatus ? (
        <div className="space-y-3">
          <div className="rounded-md border border-border/40 bg-muted/20 p-2.5">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Current Workspace</div>
            <div className="flex items-center gap-2">
              <GitBranch className="size-3.5 text-primary shrink-0" />
              <span className="text-xs font-mono font-medium text-foreground">{gitStatus.branch}</span>
            </div>
            <div className="flex items-center gap-1.5 mt-1.5 ml-[22px]">
              <Circle className={`size-1.5 shrink-0 ${gitStatus.clean ? 'fill-emerald-500 text-emerald-500' : 'fill-amber-500 text-amber-500'}`} />
              <span className="text-[10px] text-muted-foreground">{gitStatus.clean ? 'Clean working tree' : 'Uncommitted changes'}</span>
            </div>
          </div>
          {gitStatus.workingDirectory && (
            <div className="rounded-md border border-border/40 bg-muted/20 p-2.5">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Working Directory</div>
              <div className="flex items-center gap-2">
                <FolderClosed className="size-3.5 text-muted-foreground shrink-0" />
                <span className="text-[10px] font-mono text-muted-foreground truncate">{gitStatus.workingDirectory}</span>
              </div>
            </div>
          )}
        </div>
      ) : (
        <div className="flex flex-col items-center gap-2 py-6 text-center">
          <GitBranch className="size-5 text-muted-foreground/30" />
          <p className="text-[11px] text-muted-foreground/60">
            {isConnected ? 'No workspace info available.' : 'No active workspaces. Connect CLI to see workspace info.'}
          </p>
        </div>
      )}
    </div>
  )
}
