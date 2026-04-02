// Sessions tab — live workspaces from the shipd daemon.
import { GitBranch, Bot } from 'lucide-react'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import type { WorkspaceEntry } from '#/features/studio/hooks/useDaemon'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import type { GitStatusResult } from './useGitInfo'

interface SessionsTabProps {
  isConnected: boolean
  gitStatus?: GitStatusResult | null
}

export function SessionsTab({ isConnected: _isConnected, gitStatus: _gitStatus }: SessionsTabProps) {
  const { connected, workspaces } = useDaemon()
  const mcp = useLocalMcpContext()

  const activate = (branch: string) => {
    if (mcp) void mcp.callTool('activate_workspace', { branch })
  }

  if (!connected) {
    return (
      <div className="flex flex-col items-center gap-2 py-8 px-3 text-center">
        <GitBranch className="size-5 text-muted-foreground/30" />
        <p className="text-[11px] text-muted-foreground/50">Daemon offline. Start shipd to see workspaces.</p>
      </div>
    )
  }

  if (workspaces.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-8 px-3 text-center">
        <GitBranch className="size-5 text-muted-foreground/30" />
        <p className="text-[11px] text-muted-foreground/50">No workspaces found.</p>
      </div>
    )
  }

  const active = workspaces.filter((w) => w.status === 'active')
  const others = workspaces.filter((w) => w.status !== 'active')

  return (
    <div className="py-1">
      {active.length > 0 && (
        <>
          <SectionLabel>Active</SectionLabel>
          {active.map((ws) => (
            <WorkspaceRow key={ws.branch} workspace={ws} onActivate={activate} />
          ))}
        </>
      )}
      {others.length > 0 && (
        <>
          <SectionLabel className={active.length > 0 ? 'mt-2' : ''}>Workspaces</SectionLabel>
          {others.map((ws) => (
            <WorkspaceRow key={ws.branch} workspace={ws} onActivate={activate} />
          ))}
        </>
      )}
    </div>
  )
}

function SectionLabel({ children, className = '' }: { children: React.ReactNode; className?: string }) {
  return (
    <div className={`px-3 py-1 text-[9.5px] font-semibold uppercase tracking-wider text-muted-foreground/35 ${className}`}>
      {children}
    </div>
  )
}

function WorkspaceRow({
  workspace: ws,
  onActivate,
}: {
  workspace: WorkspaceEntry
  onActivate: (branch: string) => void
}) {
  const isActive = ws.status === 'active'

  return (
    <button
      onClick={() => onActivate(ws.branch)}
      className="w-full px-3 py-2 text-left hover:bg-white/[0.02] transition-colors border-b border-border/15 last:border-0"
    >
      <div className="flex items-center gap-1.5">
        <span
          className={`w-1.5 h-1.5 rounded-full shrink-0 ${isActive ? 'bg-emerald-500' : 'bg-muted-foreground/20'}`}
        />
        <span className={`text-[10.5px] font-mono truncate flex-1 ${isActive ? 'text-primary font-medium' : 'text-foreground/70'}`}>
          {ws.branch}
        </span>
        <span
          className={`text-[9px] px-1.5 py-0.5 rounded-sm ${isActive ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400' : 'bg-muted text-muted-foreground/50'}`}
        >
          {ws.status}
        </span>
      </div>
      {ws.active_agent && (
        <div className="flex items-center gap-1 ml-3 mt-0.5">
          <Bot className="size-2.5 text-muted-foreground/40 shrink-0" />
          <span className="text-[9px] text-muted-foreground/40 truncate">{ws.active_agent}</span>
        </div>
      )}
    </button>
  )
}
