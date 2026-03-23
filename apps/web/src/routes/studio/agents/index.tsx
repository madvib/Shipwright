import { createFileRoute, Link, useNavigate } from '@tanstack/react-router'
import { Plus, ArrowRight } from 'lucide-react'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { getAgentIcon } from '#/features/agents/agent-icons'
import { TechIcon, TECH_STACKS } from '#/features/studio/TechIcon'
import { AgentListSkeleton } from '#/features/studio/StudioSkeleton'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'

export const Route = createFileRoute('/studio/agents/')({
  component: AgentsListPage,
  pendingComponent: AgentListSkeleton,
  errorComponent: StudioErrorBoundary,
})

function AgentsListPage() {
  const { agents, createAgent } = useAgentStore()
  const navigate = useNavigate()

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
              {agents.length} agent{agents.length !== 1 ? 's' : ''} configured
            </p>
          </div>
          <button
            onClick={handleNewAgent}
            className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
          >
            <Plus className="size-3.5" />
            New agent
          </button>
        </div>

        {agents.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <p className="text-sm font-medium text-foreground">No agents yet</p>
            <p className="mt-1 text-xs text-muted-foreground max-w-xs">
              Create your first agent to get started.
            </p>
            <button
              onClick={handleNewAgent}
              className="mt-4 inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
            >
              <Plus className="size-3.5" />
              Create agent
            </button>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            {agents.map((a) => {
              const icon = getAgentIcon(a.profile.id)
              const preset = a.permissions?.preset ?? 'custom'

              return (
                <Link
                  key={a.profile.id}
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

                  {/* Stats row */}
                  <div className="flex items-center gap-3 text-[10px] text-muted-foreground">
                    <span>{a.skills.length} skill{a.skills.length !== 1 ? 's' : ''}</span>
                    <span className="text-border">·</span>
                    <span>{a.mcpServers.length} MCP</span>
                    <span className="text-border">·</span>
                    <span>{a.rules.length} rule{a.rules.length !== 1 ? 's' : ''}</span>
                  </div>

                  {/* Preset badge */}
                  <div className="mt-2">
                    <span className="text-[9px] px-1.5 py-0.5 rounded border border-border/40 text-muted-foreground">
                      {preset.replace('ship-', '')}
                    </span>
                  </div>
                </Link>
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}
