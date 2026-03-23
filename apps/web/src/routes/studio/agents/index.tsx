import { createFileRoute, Link } from '@tanstack/react-router'
import { useState } from 'react'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { Plus, ArrowRight } from 'lucide-react'
import { CreateAgentDialog } from '#/features/agents/dialogs/CreateAgentDialog'
import { AgentListSkeleton } from '#/features/studio/StudioSkeleton'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'

export const Route = createFileRoute('/studio/agents/')({
  component: AgentsListPage,
  pendingComponent: AgentListSkeleton,
  errorComponent: StudioErrorBoundary,
})

function AgentsListPage() {
  const { agents } = useAgentStore()
  const [createOpen, setCreateOpen] = useState(false)

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
            onClick={() => setCreateOpen(true)}
            className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
          >
            <Plus className="size-3.5" />
            New agent
          </button>
        </div>

        {agents.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-center">
            <div className="flex size-12 items-center justify-center rounded-2xl border border-border/60 bg-muted/40 text-muted-foreground/40 mb-3">
              <Plus className="size-5" />
            </div>
            <p className="text-sm font-medium text-foreground">No agents yet</p>
            <p className="mt-1 text-xs text-muted-foreground max-w-xs">
              Create your first agent to get started with AI-assisted development.
            </p>
            <button
              onClick={() => setCreateOpen(true)}
              className="mt-4 inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
            >
              <Plus className="size-3.5" />
              Create agent
            </button>
          </div>
        ) : (
          <div className="space-y-2">
            {agents.map((a) => (
              <Link
                key={a.profile.id}
                to="/studio/agents/$id"
                params={{ id: a.profile.id }}
                className="group flex items-center gap-4 rounded-xl border border-border/60 bg-card p-4 hover:border-primary/30 transition-colors no-underline"
              >
                <div
                  className="flex size-10 shrink-0 items-center justify-center rounded-xl text-sm font-bold text-white"
                  style={{ background: 'linear-gradient(135deg, oklch(0.67 0.16 58), oklch(0.5 0.16 30))' }}
                >
                  {a.profile.name.charAt(0).toUpperCase()}
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2 mb-0.5">
                    <span className="text-sm font-semibold text-foreground">{a.profile.name}</span>
                    {(a.profile.providers ?? []).map((pid: string) => (
                      <span key={pid} className="text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded">{pid}</span>
                    ))}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {a.skills.length} skill{a.skills.length !== 1 ? 's' : ''} · {a.mcpServers.length} MCP
                  </p>
                </div>
                <ArrowRight className="size-4 text-muted-foreground/20 group-hover:text-muted-foreground transition-colors" />
              </Link>
            ))}
          </div>
        )}
      </div>
      <CreateAgentDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  )
}
