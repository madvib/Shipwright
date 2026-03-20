import { createFileRoute, Link } from '@tanstack/react-router'
import { useState } from 'react'
import { useProfiles } from '#/features/studio/useProfiles'
import { useLibrary } from '#/features/compiler/useLibrary'
import { useAuth } from '#/lib/components/protected-route'
import { Plus, Users, Zap, Server, Github, Package, ArrowRight } from 'lucide-react'
import { authClient } from '#/lib/auth-client'
import { CreateAgentDialog } from '#/features/agents/dialogs/CreateAgentDialog'

export const Route = createFileRoute('/studio/')({ component: StudioHome })

function StudioHome() {
  const { profiles } = useProfiles()
  const { library } = useLibrary()
  const auth = useAuth()
  const [createOpen, setCreateOpen] = useState(false)

  const skillCount = library.skills?.length ?? 0
  const mcpCount = library.mcp_servers?.length ?? 0
  const agentCount = profiles.length

  if (agentCount === 0 && !auth.isAuthenticated) {
    return (
      <>
        <EmptyWelcome onCreateAgent={() => setCreateOpen(true)} />
        <CreateAgentDialog open={createOpen} onOpenChange={setCreateOpen} />
      </>
    )
  }

  if (agentCount === 0) {
    return (
      <>
        <EmptyDashboard onCreateAgent={() => setCreateOpen(true)} user={auth.user} />
        <CreateAgentDialog open={createOpen} onOpenChange={setCreateOpen} />
      </>
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
              {skillCount > 0 && ` · ${skillCount} skill${skillCount !== 1 ? 's' : ''}`}
              {mcpCount > 0 && ` · ${mcpCount} MCP`}
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

        <div className="grid grid-cols-3 gap-3 mb-8">
          <StatCard icon={<Users className="size-4" />} label="Agents" value={agentCount} color="text-primary bg-primary/10" href="/studio/agents" />
          <StatCard icon={<Zap className="size-4" />} label="Skills" value={skillCount} color="text-emerald-500 bg-emerald-500/10" href="/studio/skills" />
          <StatCard icon={<Server className="size-4" />} label="MCP Servers" value={mcpCount} color="text-blue-500 bg-blue-500/10" />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          <div className="md:col-span-2 space-y-4">
            {/* Quick links */}
            <Link
              to="/studio/agents"
              className="group flex items-center justify-between rounded-xl border border-border/60 bg-card p-4 hover:border-primary/30 transition-colors no-underline"
            >
              <div className="flex items-center gap-3">
                <div className="size-9 rounded-lg bg-primary/10 flex items-center justify-center">
                  <Users className="size-4 text-primary" />
                </div>
                <div>
                  <span className="text-sm font-semibold text-foreground">Your agents</span>
                  <p className="text-xs text-muted-foreground">{agentCount} configured</p>
                </div>
              </div>
              <ArrowRight className="size-4 text-muted-foreground/20 group-hover:text-muted-foreground transition-colors" />
            </Link>

            <Link
              to="/studio/skills"
              className="group flex items-center justify-between rounded-xl border border-border/60 bg-card p-4 hover:border-primary/30 transition-colors no-underline"
            >
              <div className="flex items-center gap-3">
                <div className="size-9 rounded-lg bg-emerald-500/10 flex items-center justify-center">
                  <Zap className="size-4 text-emerald-500" />
                </div>
                <div>
                  <span className="text-sm font-semibold text-foreground">Skills IDE</span>
                  <p className="text-xs text-muted-foreground">{skillCount} skill{skillCount !== 1 ? 's' : ''}</p>
                </div>
              </div>
              <ArrowRight className="size-4 text-muted-foreground/20 group-hover:text-muted-foreground transition-colors" />
            </Link>
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

function EmptyWelcome({ onCreateAgent }: { onCreateAgent: () => void }) {
  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto px-6 py-16 text-center">
        <h1 className="font-display text-3xl font-extrabold text-foreground mb-3">Welcome to Ship Studio</h1>
        <p className="text-base text-muted-foreground max-w-md mx-auto mb-8">
          Configure AI coding agents visually. Define skills, permissions, and MCP servers — compile to any provider.
        </p>

        <button
          onClick={onCreateAgent}
          className="inline-flex items-center gap-2 rounded-xl bg-primary px-6 py-3 text-sm font-bold text-primary-foreground transition hover:bg-primary/90 mb-10"
        >
          <Plus className="size-4" />
          Create your first agent
        </button>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-10 text-left max-w-lg mx-auto">
          <WelcomeCard icon={<Github className="size-5" />} title="Import existing" desc="Already have CLAUDE.md or .cursor/rules? Import and convert to Ship format." action="Import from GitHub" href="/studio/import" color="bg-muted text-foreground" />
          <WelcomeCard icon={<Package className="size-5" />} title="Browse registry" desc="Install pre-built agents and skills from the community. One-click setup." action="Browse registry" href="/registry" color="bg-violet-500/10 text-violet-500" />
        </div>

        <div className="rounded-xl border border-border/60 bg-card p-6 max-w-md mx-auto">
          <p className="text-sm text-foreground font-medium mb-1">Sign in for the full experience</p>
          <p className="text-xs text-muted-foreground mb-4">Sync configs to GitHub, publish to the registry, and use the CLI.</p>
          <button
            onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: '/studio' })}
            className="inline-flex items-center gap-2 rounded-lg bg-foreground px-4 py-2 text-sm font-medium text-background transition hover:opacity-90"
          >
            <svg className="size-4" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
            Sign in with GitHub
          </button>
          <p className="text-[10px] text-muted-foreground/50 mt-3">No account required to use Studio locally.</p>
        </div>
      </div>
    </div>
  )
}

function EmptyDashboard({ onCreateAgent, user }: { onCreateAgent: () => void; user: { name: string } | null }) {
  return (
    <div className="flex-1 overflow-auto">
      <div className="max-w-3xl mx-auto px-6 py-16 text-center">
        <h1 className="font-display text-3xl font-extrabold text-foreground mb-2">
          {user ? `Welcome, ${user.name}` : 'Welcome to Studio'}
        </h1>
        <p className="text-base text-muted-foreground mb-8 max-w-md mx-auto">
          You have no agents yet. Create your first one to get started with skills, MCP servers, and permissions.
        </p>

        <button
          onClick={onCreateAgent}
          className="inline-flex items-center gap-2 rounded-xl bg-primary px-6 py-3 text-sm font-bold text-primary-foreground transition hover:bg-primary/90 mb-10"
        >
          <Plus className="size-4" />
          Create your first agent
        </button>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 max-w-lg mx-auto text-left">
          <WelcomeCard icon={<Github className="size-5" />} title="Import from GitHub" desc="We'll scan your repos for agent configs and convert them to Ship format." action="Import" href="/studio/import" color="bg-muted text-foreground" />
          <WelcomeCard icon={<Package className="size-5" />} title="Install from registry" desc="Pre-built agents for fullstack dev, Rust, QA, and more." action="Browse" href="/registry" color="bg-violet-500/10 text-violet-500" />
        </div>
      </div>
    </div>
  )
}

function WelcomeCard({ icon, title, desc, action, onClick, href, color }: {
  icon: React.ReactNode; title: string; desc: string; action: string
  onClick?: () => void; href?: string; color: string
}) {
  return (
    <div className="rounded-xl border border-border/60 bg-card p-5 flex flex-col">
      <div className={`size-10 rounded-xl ${color} flex items-center justify-center mb-3`}>{icon}</div>
      <h3 className="text-sm font-semibold text-foreground mb-1">{title}</h3>
      <p className="text-xs text-muted-foreground flex-1 mb-4">{desc}</p>
      {href ? (
        <Link to={href as string} className="text-xs font-medium text-primary hover:underline no-underline">{action} →</Link>
      ) : (
        <button onClick={onClick} className="text-xs font-medium text-primary hover:underline text-left">{action} →</button>
      )}
    </div>
  )
}

function StatCard({ icon, label, value, color, href, subtitle }: {
  icon: React.ReactNode; label: string; value: number; color: string; href?: string; subtitle?: string
}) {
  const content = (
    <div className="rounded-xl border border-border/60 bg-card p-4 hover:border-border transition-colors h-full flex flex-col items-center justify-center text-center">
      <div className={`size-8 rounded-lg ${color} flex items-center justify-center mb-2`}>{icon}</div>
      <div className="text-2xl font-bold text-foreground">{value}</div>
      <div className="text-xs text-muted-foreground">{label}</div>
      {subtitle && <div className="text-[10px] text-muted-foreground/50 mt-0.5 truncate max-w-full">{subtitle}</div>}
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
