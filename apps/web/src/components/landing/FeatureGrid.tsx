import {
  Package,
  Code2,
  Zap,
  Lock,
} from 'lucide-react'
import type { LucideIcon } from 'lucide-react'

interface FeatureCard {
  icon: LucideIcon
  iconColor: string
  iconBg: string
  title: string
  description: string
}

const FEATURES: FeatureCard[] = [
  {
    icon: Package,
    iconColor: 'text-primary',
    iconBg: 'bg-primary/10',
    title: 'Composable agent configs',
    description:
      'Define skills, MCP servers, and agent configs in a portable format. Compose them like dependencies. Share what you build via Git.',
  },
  {
    icon: Lock,
    iconColor: 'text-blue-400',
    iconBg: 'bg-blue-500/10',
    title: 'Granular permissions',
    description:
      'Control which tools each agent can use — allow, ask, or deny at the individual tool level. Scope MCP servers per-agent with different access for different roles.',
  },
  {
    icon: Zap,
    iconColor: 'text-emerald-400',
    iconBg: 'bg-emerald-500/10',
    title: 'Ship Studio',
    description:
      'Configure agents visually with live preview. See the compiled output for every provider as you edit. Add skills, set permissions, write rules — all in the browser.',
  },
  {
    icon: Code2,
    iconColor: 'text-violet-400',
    iconBg: 'bg-violet-500/10',
    title: 'Every provider, one config',
    description:
      'First-class support for the settings developers use daily across Claude Code, Cursor, Gemini, and Codex. Full schema pass-through for everything else.',
  },
]

const TOOL_ROWS = [
  { name: 'get_file_contents', state: 'allow' as const },
  { name: 'search_code', state: 'allow' as const },
  { name: 'list_pull_requests', state: 'allow' as const },
  { name: 'create_issue', state: 'ask' as const },
  { name: 'merge_pull_request', state: 'deny' as const },
  { name: 'push_files', state: 'deny' as const },
]

const BADGE_STYLES = {
  allow: 'bg-emerald-500/10 text-emerald-400',
  ask: 'bg-primary/10 text-primary',
  deny: 'bg-red-500/10 text-red-400',
} as const

const TOGGLE_STYLES = {
  allow: 'bg-emerald-400',
  ask: 'bg-primary',
  deny: 'bg-muted',
} as const

export function FeatureGrid() {
  return (
    <section className="mx-auto max-w-[62rem] px-6 pb-20 sm:px-10">
      <div className="mb-12 text-center">
        <h2 className="mb-2 font-display text-3xl font-extrabold sm:text-4xl">
          What Ship gives your agents
        </h2>
        <p className="text-[15px] text-muted-foreground">
          Configuration as code. Permissions as policy. Every provider from one source.
        </p>
      </div>

      <div className="grid gap-4 sm:grid-cols-2">
        {FEATURES.map((f) => (
          <div
            key={f.title}
            className="rounded-xl border border-border/60 bg-card/30 p-7 transition hover:border-border/80"
          >
            <div
              className={`mb-3.5 flex size-10 items-center justify-center rounded-lg ${f.iconBg}`}
            >
              <f.icon className={`size-5 ${f.iconColor}`} />
            </div>
            <h3 className="mb-1.5 text-base font-bold">{f.title}</h3>
            <p className="text-[13px] leading-relaxed text-muted-foreground">
              {f.description}
            </p>
          </div>
        ))}

        {/* Wide card: Tool scoping */}
        <div className="rounded-xl border border-border/60 bg-card/30 p-7 transition hover:border-border/80 sm:col-span-2">
          <div className="flex flex-col gap-6 sm:flex-row sm:items-start">
            <div className="flex-1">
              <div className="mb-3.5 flex size-10 items-center justify-center rounded-lg bg-primary/10">
                <Lock className="size-5 text-primary" />
              </div>
              <h3 className="mb-1.5 text-base font-bold">
                Per-tool scoping in action
              </h3>
              <p className="text-[13px] leading-relaxed text-muted-foreground">
                Same MCP server, different tool sets per agent. A reviewer gets
                read access. A developer gets write. Nobody gets delete. Every
                tool individually controlled.
              </p>
            </div>
            <ToolScopeDemo />
          </div>
        </div>
      </div>
    </section>
  )
}

function ToolScopeDemo() {
  return (
    <div className="w-full shrink-0 rounded-lg border border-border/60 bg-background/60 p-3 sm:w-72">
      <div className="mb-2 text-[9px] font-semibold text-muted-foreground/50">
        github MCP — web-lane agent
      </div>
      {TOOL_ROWS.map((row) => (
        <div
          key={row.name}
          className="flex items-center gap-2 py-1 text-[11px]"
        >
          {/* Toggle pill */}
          <div
            className={`relative h-3.5 w-6 shrink-0 rounded-full ${TOGGLE_STYLES[row.state]}`}
          >
            <div
              className={`absolute top-[2px] size-[10px] rounded-full bg-white transition-[left] ${
                row.state === 'deny' ? 'left-[2px]' : 'left-[14px]'
              }`}
            />
          </div>
          {/* Name */}
          <span
            className={`flex-1 font-mono text-[10px] ${
              row.state === 'deny'
                ? 'text-muted-foreground/30 line-through'
                : 'text-muted-foreground'
            }`}
          >
            {row.name}
          </span>
          {/* Badge */}
          <span
            className={`rounded px-1.5 py-0.5 text-[8px] font-medium ${BADGE_STYLES[row.state]}`}
          >
            {row.state}
          </span>
        </div>
      ))}
    </div>
  )
}
