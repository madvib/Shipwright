import { createFileRoute, Link } from '@tanstack/react-router'
import { useProfiles } from '#/features/studio/useProfiles'
import { TechIcon } from '#/features/studio/TechIcon'
import { Plus, Users, Github } from 'lucide-react'

export const Route = createFileRoute('/studio/')({ component: AgentsPage })

function AgentsPage() {
  const { profiles, addProfile } = useProfiles()

  if (profiles.length === 0) {
    return (
      <div className="flex-1 overflow-auto">
        <div className="max-w-lg mx-auto py-20 px-5 flex flex-col items-center text-center">
          <div className="size-14 rounded-2xl border border-border/60 bg-muted/30 flex items-center justify-center mb-5">
            <Users className="size-6 text-muted-foreground" />
          </div>
          <h2 className="text-xl font-bold text-foreground mb-2">No agents yet</h2>
          <p className="text-sm text-muted-foreground mb-6 max-w-sm">
            Agents define how your AI coding assistants work — skills, permissions, MCP servers. Configure once, compile to any provider.
          </p>
          <div className="flex gap-3 mb-8">
            <button
              onClick={() => addProfile()}
              className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:bg-primary/90"
            >
              <Plus className="size-4" />
              Create your first agent
            </button>
            <Link
              to="/registry"
              className="inline-flex items-center gap-1.5 rounded-lg border border-primary text-primary px-4 py-2 text-sm font-medium transition hover:bg-primary/10 no-underline"
            >
              Browse registry
            </Link>
          </div>

          {/* GitHub import */}
          <div className="w-full rounded-xl border border-border/60 bg-card p-5 text-left">
            <div className="flex items-center gap-3 mb-3">
              <div className="size-9 rounded-lg bg-muted/50 flex items-center justify-center">
                <Github className="size-4 text-muted-foreground" />
              </div>
              <div>
                <p className="text-sm font-semibold text-foreground">Import from GitHub</p>
                <p className="text-xs text-muted-foreground">Already using CLAUDE.md or .cursor/rules? We'll convert it.</p>
              </div>
            </div>
            <div className="flex gap-2">
              <input
                placeholder="Paste repo URL..."
                className="flex-1 rounded-lg border border-border/60 bg-background px-3 py-1.5 text-sm text-foreground placeholder:text-muted-foreground/50 outline-none focus:border-primary/50"
              />
              <Link
                to={"/studio/import" as string}
                className="rounded-lg bg-muted px-3 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground transition no-underline"
              >
                Import
              </Link>
            </div>
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto py-8 px-5">
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-xl font-bold text-foreground">Agents</h1>
            <p className="text-sm text-muted-foreground mt-0.5">
              {profiles.length} agent{profiles.length !== 1 ? 's' : ''} configured
            </p>
          </div>
          <button
            onClick={() => addProfile()}
            className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-3 py-1.5 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
          >
            <Plus className="size-3.5" />
            New agent
          </button>
        </div>

        <div className="flex flex-col gap-3">
          {profiles.map((p) => (
            <Link
              key={p.id}
              to="/studio/agents/$id"
              params={{ id: p.id }}
              className="group flex items-start gap-4 rounded-xl border border-border/60 bg-card p-4 hover:border-primary/30 transition-colors no-underline"
            >
              <TechIcon stack={p.icon} size={40} style={{ borderRadius: 10 }} />
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-sm font-semibold text-foreground">{p.name}</span>
                  {p.selectedProviders.map((pid) => (
                    <span key={pid} className="text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded">{pid}</span>
                  ))}
                </div>
                <p className="text-xs text-muted-foreground">
                  {p.skills.length} skill{p.skills.length !== 1 ? 's' : ''} · {p.mcpServers.length} MCP
                </p>
              </div>
              <span className="text-muted-foreground/30 group-hover:text-muted-foreground transition-colors text-sm">&#8250;</span>
            </Link>
          ))}
        </div>
      </div>
    </div>
  )
}
