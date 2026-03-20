import { createFileRoute, Link } from '@tanstack/react-router'
import { useProfiles } from '#/features/studio/useProfiles'
import { useLibrary } from '#/features/compiler/useLibrary'
import { TechIcon } from '#/features/studio/TechIcon'
import { Plus, Users, Zap, Search } from 'lucide-react'
import { Button } from '@ship/primitives'
import { EmptyState } from '#/components/EmptyState'
import { GitHubImportBanner } from '#/components/GitHubImportBanner'
import { CLIInstallBanner } from '#/components/CLIInstallBanner'

export const Route = createFileRoute('/studio/')({ component: AgentsPage })

function AgentsPage() {
  const { profiles, createProfile } = useProfiles()
  const { library } = useLibrary()

  if (profiles.length === 0) {
    return (
      <div className="flex-1 overflow-auto">
        <div className="max-w-xl mx-auto py-20 px-5 flex flex-col items-center">
          <EmptyState
            icon={<Users className="size-7" />}
            title="No agents yet"
            description="Agents are how you configure AI coding assistants. Define skills, permissions, MCP servers, and compile to Claude, Gemini, Codex, or Cursor."
            primaryAction={{ label: 'Create your first agent', onClick: () => createProfile() }}
            secondaryAction={{ label: 'Browse registry', href: '/studio/registry' }}
          />
          <div className="w-full mt-8">
            <GitHubImportBanner />
          </div>
          <div className="w-full mt-4">
            <CLIInstallBanner />
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto py-8 px-5">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-xl font-bold text-foreground">Agents</h1>
            <p className="text-sm text-muted-foreground mt-0.5">
              {profiles.length} agent{profiles.length !== 1 ? 's' : ''} configured
            </p>
          </div>
          <Button onClick={() => createProfile()} variant="default" size="sm">
            <Plus className="size-4 mr-1.5" />
            New agent
          </Button>
        </div>

        {/* Agent list */}
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
                    <span
                      key={pid}
                      className="text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded"
                    >
                      {pid}
                    </span>
                  ))}
                </div>
                <p className="text-xs text-muted-foreground">
                  {p.skills.length} skill{p.skills.length !== 1 ? 's' : ''} · {p.mcpServers.length} MCP
                </p>
              </div>
              <span className="text-muted-foreground/30 group-hover:text-muted-foreground transition-colors text-sm">
                &#8250;
              </span>
            </Link>
          ))}
        </div>
      </div>
    </div>
  )
}
