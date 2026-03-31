// Sessions tab — recent sessions across all workspaces on this machine.
import { useState } from 'react'
import { Circle, GitBranch, ChevronDown, ChevronRight, Bot, Clock } from 'lucide-react'
import { useQuery } from '@tanstack/react-query'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import type { GitStatusResult } from './useGitInfo'
import { sessionKeys } from './query-keys'

interface SessionRecord {
  id: string
  workspace_branch: string
  status: 'active' | 'ended'
  started_at: string
  ended_at: string | null
  agent_id: string | null
  primary_provider: string | null
  goal: string | null
  summary: string | null
}

function useSessions(isConnected: boolean) {
  const mcp = useLocalMcpContext()
  return useQuery({
    queryKey: [...sessionKeys.all, 'list'],
    queryFn: async (): Promise<SessionRecord[]> => {
      if (!mcp) return []
      try {
        const raw = await mcp.callTool('list_sessions')
        const parsed = JSON.parse(raw)
        return Array.isArray(parsed) ? parsed : []
      } catch {
        return []
      }
    },
    enabled: isConnected && mcp?.status === 'connected',
    staleTime: 15_000,
    refetchInterval: 30_000,
  })
}

function relTime(iso: string): string {
  const diff = Date.now() - new Date(iso).getTime()
  if (diff < 60_000) return 'just now'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`
  return `${Math.floor(diff / 86_400_000)}d ago`
}

function sessionDuration(start: string, end: string | null): string {
  const ms = (end ? new Date(end).getTime() : Date.now()) - new Date(start).getTime()
  if (ms < 3_600_000) return `${Math.floor(ms / 60_000)}m`
  return `${(ms / 3_600_000).toFixed(1)}h`
}

interface SessionsTabProps {
  isConnected: boolean
  gitStatus: GitStatusResult | null | undefined
}

export function SessionsTab({ isConnected, gitStatus }: SessionsTabProps) {
  const { data: sessions = [], isLoading } = useSessions(isConnected)
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

  const toggle = (id: string) =>
    setExpanded((prev) => { const n = new Set(prev); n.has(id) ? n.delete(id) : n.add(id); return n })

  if (!isConnected) {
    return (
      <div className="flex flex-col items-center gap-2 py-8 px-3 text-center">
        <GitBranch className="size-5 text-muted-foreground/30" />
        <p className="text-[11px] text-muted-foreground/50">Connect CLI to see session history.</p>
      </div>
    )
  }

  if (isLoading) {
    return <div className="px-3 pt-4 text-[11px] text-muted-foreground/40">Loading…</div>
  }

  if (sessions.length === 0) {
    return (
      <div className="flex flex-col items-center gap-2 py-8 px-3 text-center">
        <Clock className="size-5 text-muted-foreground/30" />
        <p className="text-[11px] text-muted-foreground/50">No sessions yet on this machine.</p>
      </div>
    )
  }

  const active = sessions.filter((s) => s.status === 'active')
  const ended = sessions.filter((s) => s.status === 'ended')

  return (
    <div className="py-1">
      {active.length > 0 && (
        <>
          <SectionLabel>Active</SectionLabel>
          {active.map((s) => (
            <SessionRow key={s.id} session={s} isExpanded={expanded.has(s.id)} onToggle={() => toggle(s.id)} currentBranch={gitStatus?.branch} />
          ))}
        </>
      )}
      {ended.length > 0 && (
        <>
          <SectionLabel className={active.length > 0 ? 'mt-2' : ''}>Recent</SectionLabel>
          {ended.map((s) => (
            <SessionRow key={s.id} session={s} isExpanded={expanded.has(s.id)} onToggle={() => toggle(s.id)} currentBranch={gitStatus?.branch} />
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

function SessionRow({ session: s, isExpanded, onToggle, currentBranch }: {
  session: SessionRecord
  isExpanded: boolean
  onToggle: () => void
  currentBranch?: string
}) {
  const isCurrent = s.workspace_branch === currentBranch
  const isActive = s.status === 'active'

  return (
    <div className="border-b border-border/15 last:border-0">
      <button
        onClick={onToggle}
        className="w-full px-3 py-2 text-left hover:bg-white/[0.02] transition-colors"
      >
        {/* branch + duration + chevron */}
        <div className="flex items-center gap-1.5 mb-0.5">
          <Circle className={`size-1.5 shrink-0 ${isActive ? 'fill-emerald-500 text-emerald-500' : 'fill-muted-foreground/20 text-muted-foreground/20'}`} />
          <span className={`text-[10.5px] font-mono truncate flex-1 ${isCurrent ? 'text-primary font-medium' : 'text-foreground/70'}`}>
            {s.workspace_branch}
          </span>
          <span className="text-[9px] text-muted-foreground/30 tabular-nums shrink-0">
            {sessionDuration(s.started_at, s.ended_at)}
          </span>
          {isExpanded
            ? <ChevronDown className="size-2.5 text-muted-foreground/25 shrink-0" />
            : <ChevronRight className="size-2.5 text-muted-foreground/25 shrink-0" />}
        </div>
        {/* goal */}
        {s.goal && (
          <p className="text-[10px] text-muted-foreground/65 leading-snug truncate mb-1 ml-3.5">{s.goal}</p>
        )}
        {/* agent + time */}
        <div className="flex items-center gap-1.5 ml-3.5">
          {s.agent_id && (
            <span className="flex items-center gap-1 text-[9px] text-muted-foreground/40">
              <Bot className="size-2.5 shrink-0" />
              <span className="truncate max-w-[80px]">{s.agent_id}</span>
            </span>
          )}
          <span className="ml-auto text-[9px] text-muted-foreground/28 tabular-nums shrink-0">
            {relTime(s.started_at)}
          </span>
        </div>
      </button>
      {isExpanded && s.summary && (
        <div className="px-3 pb-2.5 pt-2 bg-black/[0.06] border-t border-border/10">
          <p className="text-[10px] text-muted-foreground/55 leading-relaxed">{s.summary}</p>
        </div>
      )}
    </div>
  )
}
