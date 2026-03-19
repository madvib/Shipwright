import { Link } from '@tanstack/react-router'
import { ArrowRight, Lock } from 'lucide-react'

interface CardProps {
  title: string
  description: string
  accent: string        // tailwind color token e.g. 'violet'
  emptyLabel: string
  href: string
  locked?: boolean
  lockReason?: string
  count?: number
  preview?: React.ReactNode
}

function DashboardCard({
  title,
  description,
  accent,
  emptyLabel,
  href,
  locked,
  lockReason,
  count,
  preview,
}: CardProps) {
  const accentMap: Record<string, { bg: string; text: string; dot: string }> = {
    violet:  { bg: 'bg-violet-500/8',  text: 'text-violet-600 dark:text-violet-400',  dot: 'bg-violet-500' },
    amber:   { bg: 'bg-amber-500/8',   text: 'text-amber-600 dark:text-amber-400',    dot: 'bg-amber-500'  },
    emerald: { bg: 'bg-emerald-500/8', text: 'text-emerald-600 dark:text-emerald-400', dot: 'bg-emerald-500' },
    blue:    { bg: 'bg-blue-500/8',    text: 'text-blue-600 dark:text-blue-400',      dot: 'bg-blue-500'   },
    neutral: { bg: 'bg-muted/40',      text: 'text-muted-foreground',                 dot: 'bg-muted-foreground' },
  }

  const colors = accentMap[accent] ?? accentMap.neutral

  const inner = (
    <div className={`group relative flex flex-col rounded-xl border border-border/60 p-5 transition
      ${locked ? 'opacity-50 cursor-not-allowed select-none' : 'hover:border-border hover:shadow-sm cursor-pointer bg-card'}`}
    >
      {/* Accent dot */}
      <div className={`mb-3 flex size-8 items-center justify-center rounded-lg ${colors.bg}`}>
        <span className={`size-2.5 rounded-full ${colors.dot}`} />
      </div>

      <div className="flex items-start justify-between gap-2 mb-1">
        <h3 className="font-display text-sm font-semibold text-foreground leading-tight">{title}</h3>
        {locked ? (
          <Lock className="size-3.5 text-muted-foreground/50 shrink-0 mt-0.5" />
        ) : (
          <ArrowRight className="size-3.5 text-muted-foreground/40 shrink-0 mt-0.5 transition group-hover:translate-x-0.5 group-hover:text-muted-foreground" />
        )}
      </div>

      <p className="text-[11px] text-muted-foreground leading-relaxed mb-3">{description}</p>

      {locked && lockReason ? (
        <p className="text-[10px] text-muted-foreground/60 italic">{lockReason}</p>
      ) : count !== undefined && count > 0 ? (
        <div className="flex items-center gap-1.5">
          <span className={`text-xs font-semibold ${colors.text}`}>{count}</span>
          <span className="text-[10px] text-muted-foreground">{preview}</span>
        </div>
      ) : (
        <span className={`text-xs font-medium ${colors.text}`}>{emptyLabel}</span>
      )}
    </div>
  )

  if (locked) return inner
  return <Link to={href as '/'} className="no-underline">{inner}</Link>
}

interface DashboardCardsProps {
  profileCount: number
  workflowJobCount: number
}

export function DashboardCards({ profileCount, workflowJobCount }: DashboardCardsProps) {
  const hasProfiles = profileCount > 0

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
      <DashboardCard
        title="Profiles"
        description="Choose which AI providers to target. Each selected provider gets its own output file."
        accent="violet"
        emptyLabel="Configure →"
        href="/studio/profiles"
        count={profileCount > 0 ? profileCount : undefined}
        preview={`profile${profileCount !== 1 ? 's' : ''}`}
      />
      <DashboardCard
        title="Workflow"
        description="Wire agents together — roles, routing, and session orchestration on a canvas."
        accent="amber"
        emptyLabel="Open canvas →"
        href="/canvas"
        locked={!hasProfiles}
        lockReason="Create a profile first — canvas nodes are profiles"
        count={workflowJobCount > 0 ? workflowJobCount : undefined}
        preview="active jobs"
      />
      <DashboardCard
        title="Skills"
        description="Instruction files injected into agent context — workflows, domain knowledge, repeatable tasks."
        accent="emerald"
        emptyLabel="Add skills →"
        href="/studio/skills"
      />
      <DashboardCard
        title="MCP Servers"
        description="Tools, APIs, and services your agents can call during a session."
        accent="blue"
        emptyLabel="Add servers →"
        href="/studio/mcp"
      />
      <DashboardCard
        title="Export"
        description="Download provider files or sync via CLI. CLAUDE.md, .mcp.json, GEMINI.md, and more."
        accent="neutral"
        emptyLabel="Export →"
        href="/studio/export"
        locked={!hasProfiles}
        lockReason="Create a profile to unlock export"
      />
      <DashboardCard
        title="Registry"
        description="Browse agent packages — skills, MCP servers, and presets for your AI workflow."
        accent="neutral"
        emptyLabel="Browse →"
        href="/studio/registry"
      />
    </div>
  )
}
