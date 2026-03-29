// Sessions tab content for the Session sidebar.
// Shows current workspace info from git status, workspace list, and project info.

import { Circle, GitBranch, FolderClosed, Layers, Info } from 'lucide-react'
import { useQuery } from '@tanstack/react-query'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import type { GitStatusResult, GitLogEntry } from './useGitInfo'
import { sessionKeys } from './query-keys'

interface SessionsTabProps {
  isConnected: boolean
  gitStatus: GitStatusResult | null | undefined
  gitLog: GitLogEntry[] | null | undefined
}

interface WorkspaceInfo {
  id: string
  branch: string
  status: string
  kind?: string
  name?: string
}

interface ProjectInfo {
  name?: string
  path?: string
}

function useWorkspaces(isConnected: boolean) {
  const mcp = useLocalMcpContext()

  return useQuery({
    queryKey: [...sessionKeys.all, 'workspaces'],
    queryFn: async (): Promise<WorkspaceInfo[]> => {
      if (!mcp) return []
      try {
        const raw = await mcp.callTool('list_workspaces')
        const parsed = JSON.parse(raw)
        if (Array.isArray(parsed)) return parsed as WorkspaceInfo[]
        if (parsed?.workspaces) return parsed.workspaces as WorkspaceInfo[]
        return []
      } catch {
        return []
      }
    },
    enabled: isConnected && mcp?.status === 'connected',
    staleTime: 30_000,
    refetchInterval: 60_000,
  })
}

function useProjectInfo(isConnected: boolean) {
  const mcp = useLocalMcpContext()

  return useQuery({
    queryKey: [...sessionKeys.all, 'projectInfo'],
    queryFn: async (): Promise<ProjectInfo | null> => {
      if (!mcp) return null
      try {
        const raw = await mcp.callTool('get_project_info')
        return JSON.parse(raw) as ProjectInfo
      } catch {
        return null
      }
    },
    enabled: isConnected && mcp?.status === 'connected',
    staleTime: 60_000,
  })
}

const STATUS_STYLES: Record<string, string> = {
  active: 'bg-emerald-500/15 text-emerald-600 dark:text-emerald-400',
  idle: 'bg-amber-500/15 text-amber-600 dark:text-amber-400',
  archived: 'bg-muted text-muted-foreground',
  complete: 'bg-sky-500/15 text-sky-600 dark:text-sky-400',
}

export function SessionsTab({ isConnected, gitStatus, gitLog }: SessionsTabProps) {
  const { data: workspaces } = useWorkspaces(isConnected)
  const { data: projectInfo } = useProjectInfo(isConnected)
  const commitsAhead = gitLog?.length ?? 0

  return (
    <div className="px-3 pt-3 text-xs text-muted-foreground">
      {isConnected && gitStatus ? (
        <div className="space-y-3">
          {/* Current workspace */}
          <div className="rounded-md border border-border/40 bg-muted/20 p-2.5">
            <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Current Workspace</div>
            <div className="flex items-center gap-2">
              <GitBranch className="size-3.5 text-primary shrink-0" />
              <span className="text-xs font-mono font-medium text-foreground">{gitStatus.branch}</span>
            </div>
            <div className="flex items-center gap-1.5 mt-1.5 ml-[22px]">
              <Circle className={`size-1.5 shrink-0 ${gitStatus.clean ? 'fill-emerald-500 text-emerald-500' : 'fill-amber-500 text-amber-500'}`} />
              <span className="text-[10px] text-muted-foreground">
                {gitStatus.clean ? 'Clean working tree' : 'Uncommitted changes'}
              </span>
            </div>
            {commitsAhead > 0 && (
              <div className="flex items-center gap-1.5 mt-1 ml-[22px]">
                <Circle className="size-1.5 shrink-0 fill-sky-500 text-sky-500" />
                <span className="text-[10px] text-muted-foreground">{commitsAhead} recent commit{commitsAhead !== 1 ? 's' : ''}</span>
              </div>
            )}
          </div>

          {/* Project info */}
          {projectInfo && (projectInfo.name || projectInfo.path) && (
            <div className="rounded-md border border-border/40 bg-muted/20 p-2.5">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Project</div>
              {projectInfo.name && (
                <div className="flex items-center gap-2">
                  <Info className="size-3.5 text-muted-foreground shrink-0" />
                  <span className="text-xs font-medium text-foreground">{projectInfo.name}</span>
                </div>
              )}
              {projectInfo.path && (
                <div className="flex items-center gap-2 mt-1">
                  <FolderClosed className="size-3.5 text-muted-foreground shrink-0" />
                  <span className="text-[10px] font-mono text-muted-foreground truncate">{projectInfo.path}</span>
                </div>
              )}
            </div>
          )}

          {/* Working directory fallback when no project info */}
          {!projectInfo?.path && gitStatus.workingDirectory && (
            <div className="rounded-md border border-border/40 bg-muted/20 p-2.5">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Working Directory</div>
              <div className="flex items-center gap-2">
                <FolderClosed className="size-3.5 text-muted-foreground shrink-0" />
                <span className="text-[10px] font-mono text-muted-foreground truncate">{gitStatus.workingDirectory}</span>
              </div>
            </div>
          )}

          {/* Workspace list */}
          {workspaces && workspaces.length > 0 && (
            <div className="rounded-md border border-border/40 bg-muted/20 p-2.5">
              <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">
                Workspaces
                <span className="ml-1.5 text-muted-foreground/40">{workspaces.length}</span>
              </div>
              <div className="space-y-1.5">
                {workspaces.map((ws) => (
                  <div key={ws.id || ws.branch} className="flex items-center gap-2">
                    <Layers className="size-3 text-muted-foreground/60 shrink-0" />
                    <span className="text-[10px] font-mono text-foreground/80 truncate flex-1">
                      {ws.name || ws.branch || ws.id}
                    </span>
                    <span className={`text-[9px] font-medium px-1.5 py-0.5 rounded ${STATUS_STYLES[ws.status] ?? STATUS_STYLES.idle}`}>
                      {ws.status}
                    </span>
                  </div>
                ))}
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
