import { Link } from '@tanstack/react-router'
import { ArrowLeft, ChevronRight } from 'lucide-react'
import type { AgentProfile } from '../types'

interface AgentHeaderProps {
  profile: AgentProfile
}

export function AgentHeader({ profile }: AgentHeaderProps) {
  const initial = profile.name.charAt(0).toUpperCase()
  const skillCount = profile.skills.length
  const mcpCount = profile.mcpServers.length
  const subagentCount = profile.subagents.length

  return (
    <>
      {/* Breadcrumb */}
      <div className="flex items-center gap-2 px-5 py-3 text-xs text-muted-foreground border-b border-border/40">
        <Link
          to="/studio"
          className="flex items-center gap-1 text-muted-foreground hover:text-primary transition-colors"
        >
          <ArrowLeft className="size-3" />
        </Link>
        <Link
          to="/studio"
          className="text-muted-foreground hover:text-primary transition-colors"
        >
          Agents
        </Link>
        <ChevronRight className="size-3 text-muted-foreground/40" />
        <span className="text-foreground">{profile.name}</span>
      </div>

      {/* Agent info */}
      <div className="flex items-start gap-4 px-5 py-5 border-b border-border/40">
        <div
          className="flex size-12 shrink-0 items-center justify-center rounded-xl text-xl font-bold text-white"
          style={{ background: 'linear-gradient(135deg, oklch(0.67 0.16 58), oklch(0.5 0.16 30))' }}
        >
          {initial}
        </div>
        <div className="flex-1 min-w-0">
          <h1 className="font-display text-xl font-bold text-foreground">
            {profile.name}
          </h1>
          <p className="mt-0.5 text-[13px] text-muted-foreground">
            {profile.description}
          </p>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {profile.providers.map((p) => (
              <span
                key={p}
                className="rounded-md bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-primary"
              >
                {p}
              </span>
            ))}
            <span className="rounded-md bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
              {profile.version}
            </span>
            <span className="rounded-md bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
              {skillCount} skills / {mcpCount} MCP / {subagentCount} subagents
            </span>
          </div>
        </div>
      </div>
    </>
  )
}
