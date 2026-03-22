import { createFileRoute, Link, useNavigate } from '@tanstack/react-router'
import { useState } from 'react'
import { useAgentStore } from '#/features/agents/useAgentStore'
import { useLibrary } from '#/features/compiler/useLibrary'
import { useAuth } from '#/lib/components/protected-route'
import {
  Plus, Users, Zap, Server, Github, Package, ArrowRight, LayoutTemplate,
} from 'lucide-react'
import { authClient } from '#/lib/auth-client'
import { CreateAgentDialog } from '#/features/agents/dialogs/CreateAgentDialog'
import { DashboardSkeleton } from '#/features/studio/StudioSkeleton'
import { TemplateGrid } from '#/features/studio/TemplateGrid'
import { templateToAgent } from '#/features/agents/agent-templates'
import type { AgentTemplate } from '#/features/agents/agent-templates'

export const Route = createFileRoute('/studio/')({
  component: StudioHome,
  pendingComponent: DashboardSkeleton,
})

function StudioHome() {
  const { agents, createAgent } = useAgentStore()
  const { library } = useLibrary()
  const auth = useAuth()
  const navigate = useNavigate()
  const [createOpen, setCreateOpen] = useState(false)
  const [templateOpen, setTemplateOpen] = useState(false)

  const skillCount = library.skills?.length ?? 0
  const mcpCount = library.mcp_servers?.length ?? 0
  const agentCount = agents.length

  const handleTemplateSelect = (template: AgentTemplate) => {
    const partial = templateToAgent(template, template.name)
    const id = createAgent(partial)
    void navigate({ to: '/studio/agents/$id', params: { id } })
  }

  if (agentCount === 0) {
    return (
      <EmptyState
        isAuthenticated={auth.isAuthenticated}
        onTemplateSelect={handleTemplateSelect}
      />
    )
  }

  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-5xl mx-auto px-6 py-8">
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="font-display text-2xl font-bold text-foreground">Studio</h1>
            <p className="text-sm text-muted-foreground mt-1">
              {agentCount} agent{agentCount !== 1 ? 's' : ''}
              {skillCount > 0 && ` \u00b7 ${skillCount} skill${skillCount !== 1 ? 's' : ''}`}
              {mcpCount > 0 && ` \u00b7 ${mcpCount} MCP`}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => setTemplateOpen(!templateOpen)}
              className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-semibold text-foreground transition hover:bg-muted"
            >
              <LayoutTemplate className="size-3.5" />
              From template
            </button>
            <button
              onClick={() => setCreateOpen(true)}
              className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:bg-primary/90"
            >
              <Plus className="size-3.5" />
              New agent
            </button>
          </div>
        </div>

        {templateOpen && (
          <div className="mb-8">
            <TemplateGrid onSelect={handleTemplateSelect} compact />
          </div>
        )}

        <div className="grid grid-cols-3 gap-3 mb-8">
          <StatCard icon={<Users className="size-4" />} label="Agents" value={agentCount} color="text-primary bg-primary/10" href="/studio/agents" />
          <StatCard icon={<Zap className="size-4" />} label="Skills" value={skillCount} color="text-emerald-500 bg-emerald-500/10" href="/studio/skills" />
          <StatCard icon={<Server className="size-4" />} label="MCP Servers" value={mcpCount} color="text-blue-500 bg-blue-500/10" />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="md:col-span-2 space-y-4">
            <QuickLink to="/studio/agents" icon={<Users className="size-4 text-primary" />} iconBg="bg-primary/10" label="Your agents" sublabel={`${agentCount} configured`} />
            <QuickLink to="/studio/skills" icon={<Zap className="size-4 text-emerald-500" />} iconBg="bg-emerald-500/10" label="Skills IDE" sublabel={`${skillCount} skill${skillCount !== 1 ? 's' : ''}`} />
          </div>

          <div className="space-y-4">
            <div className="rounded-xl border border-border/60 bg-card p-4">
              <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider mb-3">Quick actions</h3>
              <div className="space-y-1.5">
                <SidebarAction icon={<Plus className="size-3.5" />} label="Create agent" onClick={() => setCreateOpen(true)} />
                <SidebarAction icon={<Github className="size-3.5" />} label="Import from GitHub" href="/studio/import" />
                <SidebarAction icon={<Package className="size-3.5" />} label="Browse registry" href="/registry" />
              </div>
            </div>
          </div>
        </div>
      </div>
      <CreateAgentDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  )
}

/* ---- Empty state with template cards ---- */

function EmptyState({ isAuthenticated, onTemplateSelect }: {
  isAuthenticated: boolean
  onTemplateSelect: (t: AgentTemplate) => void
}) {
  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto px-6 py-14">
        <div className="text-center mb-10">
          <h1 className="font-display text-2xl font-bold text-foreground mb-2">
            Start from a template
          </h1>
          <p className="text-sm text-muted-foreground max-w-md mx-auto">
            Pick a starting point &mdash; customize everything after
          </p>
        </div>

        <TemplateGrid onSelect={onTemplateSelect} />

        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 mt-8 max-w-lg mx-auto">
          <SecondaryOption icon={<Github className="size-4" />} label="Import from GitHub" href="/studio/import" />
          <SecondaryOption icon={<Package className="size-4" />} label="Browse registry" href="/registry" />
        </div>

        {!isAuthenticated && (
          <div className="rounded-xl border border-border/60 bg-card p-5 max-w-sm mx-auto mt-10 text-center">
            <p className="text-sm text-foreground font-medium mb-1">Sign in for the full experience</p>
            <p className="text-xs text-muted-foreground mb-4">Sync to GitHub, publish to the registry, use the CLI.</p>
            <button
              onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: '/studio' })}
              className="inline-flex items-center gap-2 rounded-lg bg-foreground px-4 py-2 text-sm font-medium text-background transition hover:opacity-90"
            >
              <Github className="size-4" />
              Sign in with GitHub
            </button>
            <p className="text-[10px] text-muted-foreground/50 mt-3">No account required to use Studio locally.</p>
          </div>
        )}
      </div>
    </div>
  )
}

/* ---- Shared layout components ---- */

function QuickLink({ to, icon, iconBg, label, sublabel }: {
  to: string; icon: React.ReactNode; iconBg: string; label: string; sublabel: string
}) {
  return (
    <Link
      to={to as string}
      className="group flex items-center justify-between rounded-xl border border-border/60 bg-card p-4 hover:border-primary/30 transition-colors no-underline"
    >
      <div className="flex items-center gap-3">
        <div className={`size-9 rounded-lg ${iconBg} flex items-center justify-center`}>{icon}</div>
        <div>
          <span className="text-sm font-semibold text-foreground">{label}</span>
          <p className="text-xs text-muted-foreground">{sublabel}</p>
        </div>
      </div>
      <ArrowRight className="size-4 text-muted-foreground/20 group-hover:text-muted-foreground transition-colors" />
    </Link>
  )
}

function SecondaryOption({ icon, label, href }: {
  icon: React.ReactNode; label: string; href: string
}) {
  return (
    <Link
      to={href as string}
      className="flex items-center gap-2.5 rounded-xl border border-border/60 bg-card px-4 py-3 text-sm text-muted-foreground hover:text-foreground hover:border-border transition-colors no-underline"
    >
      {icon}
      <span className="font-medium">{label}</span>
      <ArrowRight className="size-3.5 ml-auto opacity-30" />
    </Link>
  )
}

function StatCard({ icon, label, value, color, href }: {
  icon: React.ReactNode; label: string; value: number; color: string; href?: string
}) {
  const content = (
    <div className="rounded-xl border border-border/60 bg-card p-4 hover:border-border transition-colors h-full flex flex-col items-center justify-center text-center">
      <div className={`size-8 rounded-lg ${color} flex items-center justify-center mb-2`}>{icon}</div>
      <div className="text-2xl font-bold text-foreground">{value}</div>
      <div className="text-xs text-muted-foreground">{label}</div>
    </div>
  )
  return href ? <Link to={href as string} className="no-underline h-full">{content}</Link> : content
}

function SidebarAction({ icon, label, onClick, href }: {
  icon: React.ReactNode; label: string; onClick?: () => void; href?: string
}) {
  if (href) {
    return (
      <Link to={href as string} className="flex items-center gap-2 rounded-lg px-2.5 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors no-underline">
        {icon} {label}
      </Link>
    )
  }
  return (
    <button onClick={onClick} className="w-full flex items-center gap-2 rounded-lg px-2.5 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
      {icon} {label}
    </button>
  )
}
