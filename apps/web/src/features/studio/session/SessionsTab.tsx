// Sessions tab — session history from the shipd daemon.
import { Clock, Bot, GitBranch } from 'lucide-react'
import { useDaemon, type SessionEntry } from '#/features/studio/hooks/useDaemon'
import type { Workspace } from '@ship/ui'

export function SessionsTab() {
  const { connected, workspaces, sessions } = useDaemon()

  if (!connected) {
    return (
      <div className="flex flex-col items-center gap-2 py-8 px-3 text-center">
        <GitBranch className="size-5 text-muted-foreground/30" />
        <p className="text-[11px] text-muted-foreground/50">Daemon offline. Start shipd to see sessions.</p>
      </div>
    )
  }

  const active = workspaces.find((w) => w.status === 'active')

  return (
    <div className="py-1">
      {active && (
        <>
          <SectionLabel>Active Workspace</SectionLabel>
          <WorkspaceInfo workspace={active} />
        </>
      )}
      {sessions.length > 0 ? (
        <>
          <SectionLabel className="mt-2">Session History</SectionLabel>
          {sessions.map((s) => (
            <SessionRow key={s.id} session={s} />
          ))}
        </>
      ) : (
        <div className="px-3 py-4 text-center">
          <p className="text-[10px] text-muted-foreground/40">No sessions recorded yet.</p>
        </div>
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

function WorkspaceInfo({ workspace: ws }: { workspace: Workspace }) {
  return (
    <div className="px-3 py-2 border-b border-border/15">
      <div className="flex items-center gap-1.5">
        <span className="w-1.5 h-1.5 rounded-full shrink-0 bg-emerald-500" />
        <span className="text-[10.5px] font-mono text-primary font-medium truncate flex-1">{ws.branch}</span>
      </div>
      {ws.active_agent && (
        <div className="flex items-center gap-1 ml-3 mt-0.5">
          <Bot className="size-2.5 text-muted-foreground/40 shrink-0" />
          <span className="text-[9px] text-muted-foreground/40 truncate">{ws.active_agent}</span>
        </div>
      )}
    </div>
  )
}

function SessionRow({ session: s }: { session: SessionEntry }) {
  const isActive = s.status === 'active'
  const startedAt = s.started_at ? relativeTime(new Date(s.started_at).getTime()) : null

  return (
    <div className="px-3 py-2 border-b border-border/15 last:border-0">
      <div className="flex items-center gap-1.5">
        <span
          className={`w-1.5 h-1.5 rounded-full shrink-0 ${isActive ? 'bg-emerald-500' : 'bg-muted-foreground/20'}`}
        />
        <span className={`text-[10px] font-mono truncate flex-1 ${isActive ? 'text-foreground' : 'text-foreground/60'}`}>
          {s.agent_id ?? 'unknown agent'}
        </span>
        <span className={`text-[9px] px-1.5 py-0.5 rounded-sm ${isActive ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400' : 'bg-muted text-muted-foreground/50'}`}>
          {s.status}
        </span>
      </div>
      {s.goal && (
        <p className="text-[9.5px] text-muted-foreground/50 leading-snug truncate mt-0.5 ml-3">{s.goal}</p>
      )}
      <div className="flex items-center gap-2 ml-3 mt-0.5">
        {startedAt && (
          <span className="flex items-center gap-0.5 text-[9px] text-muted-foreground/30">
            <Clock className="size-2.5" /> {startedAt}
          </span>
        )}
        {s.tool_call_count > 0 && (
          <span className="text-[9px] text-muted-foreground/30 tabular-nums">{s.tool_call_count} tools</span>
        )}
        {s.primary_provider && (
          <span className="text-[9px] text-muted-foreground/30">{s.primary_provider}</span>
        )}
      </div>
      {s.summary && (
        <p className="text-[9px] text-muted-foreground/40 leading-snug mt-1 ml-3 line-clamp-2">{s.summary}</p>
      )}
    </div>
  )
}

function relativeTime(ms: number): string {
  const diff = Date.now() - ms
  if (diff < 60_000) return 'just now'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`
  return `${Math.floor(diff / 86_400_000)}d ago`
}
