import { createFileRoute, Link, useNavigate } from '@tanstack/react-router'
import { useState, useMemo } from 'react'
import {
  Plus, ArrowRight, Monitor, FolderOpen, Library, PenLine,
  Terminal, ExternalLink,
} from 'lucide-react'
import { useAgents } from '#/features/agents/useAgents'
import { useAgentDrafts } from '#/features/agents/useAgentDrafts'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { getAgentIcon } from '#/features/agents/agent-icons'
import { TechIcon, TECH_STACKS } from '#/features/studio/TechIcon'
import { AgentListSkeleton } from '#/features/studio/StudioSkeleton'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'

type SourceFilter = 'all' | 'project' | 'library'

export const Route = createFileRoute('/studio/agents/')({
  component: AgentsListPage,
  pendingComponent: AgentListSkeleton,
  errorComponent: StudioErrorBoundary,
})

function AgentsListPage() {
  const { agents, isConnected } = useAgents()
  const { hasDraft } = useAgentDrafts()
  const { createAgent } = useAgentStore()
  const navigate = useNavigate()
  const mcp = useLocalMcpContext()
  const localIds = mcp?.localAgentIds ?? new Set<string>()
  const [filter, setFilter] = useState<SourceFilter>('all')

  const hasLibrary = agents.some((a) => a.source === 'library')
  const filtered = useMemo(() => {
    if (filter === 'all') return agents
    return agents.filter((a) => (a.source ?? 'project') === filter)
  }, [agents, filter])

  const handleNewAgent = () => {
    const id = createAgent()
    void navigate({ to: '/studio/agents/$id', params: { id } })
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto px-6 py-8">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="font-display text-2xl font-bold text-foreground">Agents</h1>
            <p className="text-sm text-muted-foreground mt-1">
              {filtered.length} agent{filtered.length !== 1 ? 's' : ''}{filter !== 'all' ? ` (${filter})` : ''}
            </p>
          </div>
          <div className="flex items-center gap-2">
            {hasLibrary && (
              <div className="flex items-center rounded-lg border border-border/40 bg-card/50 p-0.5">
                <FilterPill active={filter === 'all'} onClick={() => setFilter('all')}>All</FilterPill>
                <FilterPill active={filter === 'project'} onClick={() => setFilter('project')}>
                  <FolderOpen className="size-3" /> Project
                </FilterPill>
                <FilterPill active={filter === 'library'} onClick={() => setFilter('library')}>
                  <Library className="size-3" /> Library
                </FilterPill>
              </div>
            )}
            <button
              onClick={handleNewAgent}
              className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
            >
              <Plus className="size-3.5" />
              New agent
            </button>
          </div>
        </div>

        {filtered.length === 0 ? (
          <EmptyState isConnected={isConnected} onNew={handleNewAgent} />
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {filtered.map((a) => (
              <AgentCard key={a.profile.id} agent={a} isDraft={hasDraft(a.profile.id)} isLocal={localIds.has(a.profile.id)} />
            ))}
          </div>
        )}
      </div>
    </div>
  )
}

function EmptyState({ isConnected, onNew }: { isConnected: boolean; onNew: () => void }) {
  if (isConnected) {
    return (
      <div className="flex flex-col items-center justify-center py-16 text-center">
        <p className="text-sm font-medium text-foreground">No agents found</p>
        <p className="mt-1 text-xs text-muted-foreground max-w-xs">
          Your CLI is connected but no agents exist yet. Create one to get started.
        </p>
        <button
          onClick={onNew}
          className="mt-4 inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
        >
          <Plus className="size-3.5" />
          Create agent
        </button>
      </div>
    )
  }

  return (
    <div className="flex flex-col items-center justify-center py-16 text-center max-w-md mx-auto">
      <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40 mb-4">
        <Terminal className="size-5 text-muted-foreground" />
      </div>
      <p className="text-sm font-medium text-foreground">Connect to CLI to see your agents</p>
      <p className="mt-2 text-xs text-muted-foreground leading-relaxed">
        Agents are configured in your local .ship/ directory and synced via the Ship CLI.
        Install the CLI, then click Connect in the dock.
      </p>
      <div className="mt-4 w-full rounded-lg border border-border/40 bg-card/60 px-4 py-3">
        <code className="text-[11px] font-mono text-emerald-400">
          curl -fsSL https://ship.dev/install | sh
        </code>
      </div>
      <div className="mt-4 flex items-center gap-3">
        <a
          href="https://github.com/madvib/Ship#installation"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-1.5 rounded-lg border border-border/40 px-4 py-2 text-xs font-medium text-muted-foreground hover:text-foreground hover:border-primary/30 transition no-underline"
        >
          <ExternalLink className="size-3" />
          Installation docs
        </a>
        <button
          onClick={onNew}
          className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
        >
          <Plus className="size-3.5" />
          Create locally
        </button>
      </div>
    </div>
  )
}

function AgentCard({ agent: a, isDraft, isLocal }: { agent: ReturnType<typeof useAgents>['agents'][number]; isDraft: boolean; isLocal: boolean }) {
  const icon = getAgentIcon(a.profile.id)
  const preset = a.permissions?.preset ?? 'custom'

  return (
    <Link
      to="/studio/agents/$id"
      params={{ id: a.profile.id }}
      className="group rounded-xl border border-border/60 bg-card p-4 hover:border-primary/30 transition-colors no-underline"
    >
      <div className="flex items-start gap-3 mb-3">
        {icon && icon in TECH_STACKS ? (
          <TechIcon stack={icon} size={40} />
        ) : (
          <div
            className="flex size-10 shrink-0 items-center justify-center rounded-xl text-sm font-bold text-white"
            style={{ background: 'linear-gradient(135deg, oklch(0.67 0.16 58), oklch(0.5 0.16 30))' }}
          >
            {a.profile.name.charAt(0).toUpperCase()}
          </div>
        )}
        <div className="flex-1 min-w-0">
          <div className="text-sm font-semibold text-foreground truncate">{a.profile.name}</div>
          {a.profile.description && (
            <p className="text-[11px] text-muted-foreground line-clamp-2 mt-0.5">{a.profile.description}</p>
          )}
        </div>
        <ArrowRight className="size-3.5 text-muted-foreground/20 group-hover:text-muted-foreground transition-colors mt-1" />
      </div>

      <div className="flex items-center gap-3 text-[10px] text-muted-foreground">
        <span>{a.skills.length} skill{a.skills.length !== 1 ? 's' : ''}</span>
        <span className="text-border">·</span>
        <span>{a.mcpServers.length} MCP</span>
        <span className="text-border">·</span>
        <span>{a.rules.length} rule{a.rules.length !== 1 ? 's' : ''}</span>
      </div>

      <div className="mt-2 flex items-center gap-1.5">
        <span className="text-[9px] px-1.5 py-0.5 rounded border border-border/40 text-muted-foreground">
          {preset.replace('ship-', '')}
        </span>
        {a.source === 'library' && (
          <span className="inline-flex items-center gap-1 text-[9px] px-1.5 py-0.5 rounded border border-violet-500/30 bg-violet-500/10 text-violet-500">
            <Library className="size-2.5" />
            Library
          </span>
        )}
        {isLocal && (
          <span className="inline-flex items-center gap-1 text-[9px] px-1.5 py-0.5 rounded border border-emerald-500/30 bg-emerald-500/10 text-emerald-500">
            <Monitor className="size-2.5" />
            Local
          </span>
        )}
        {isDraft && (
          <span className="inline-flex items-center gap-1 text-[9px] px-1.5 py-0.5 rounded border border-amber-500/30 bg-amber-500/10 text-amber-500">
            <PenLine className="size-2.5" />
            Modified
          </span>
        )}
      </div>
    </Link>
  )
}

function FilterPill({ active, onClick, children }: { active: boolean; onClick: () => void; children: React.ReactNode }) {
  return (
    <button
      onClick={onClick}
      className={`inline-flex items-center gap-1 rounded-md px-2.5 py-1 text-[11px] font-medium transition-colors ${
        active
          ? 'bg-primary/10 text-primary'
          : 'text-muted-foreground/60 hover:text-muted-foreground'
      }`}
    >
      {children}
    </button>
  )
}
