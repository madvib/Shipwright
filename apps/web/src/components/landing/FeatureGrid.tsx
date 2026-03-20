import {
  Package,
  Code2,
  Zap,
  Search,
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
    title: 'Package manager for agents',
    description:
      'Install agents, skills, and MCP servers like npm packages. Version them. Share them. Compose them into exactly the agent you need.',
  },
  {
    icon: Code2,
    iconColor: 'text-blue-400',
    iconBg: 'bg-blue-500/10',
    title: 'Write once, compile everywhere',
    description:
      'One .ship/ directory compiles to CLAUDE.md, GEMINI.md, AGENTS.md, .cursor/rules, and .mcp.json simultaneously. Switch providers without reconfiguring.',
  },
  {
    icon: Zap,
    iconColor: 'text-emerald-400',
    iconBg: 'bg-emerald-500/10',
    title: 'Visual skill editor',
    description:
      'Create and edit skills in an IDE-lite environment. Syntax highlighting, live preview, and one-click publish to the registry.',
  },
  {
    icon: Search,
    iconColor: 'text-violet-400',
    iconBg: 'bg-violet-500/10',
    title: 'Community registry',
    description:
      'Browse hundreds of skills, agents, and MCP servers. Install with one click. Publish your own for the community.',
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
          Everything agents need
        </h2>
        <p className="text-[15px] text-muted-foreground">
          A complete platform for configuring, scoping, and distributing AI
          coding agents.
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
                Scope agents to exactly what they need
              </h3>
              <p className="text-[13px] leading-relaxed text-muted-foreground">
                Same MCP server, different tool sets per agent. Allow read
                operations, require confirmation for writes, deny destructive
                actions. Five permission dimensions: tools, filesystem, commands,
                network, and agent limits.
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
        github MCP -- web-lane agent
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
