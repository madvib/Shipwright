import { Pencil } from 'lucide-react'
import type { ResolvedAgentProfile } from '../types'

interface AgentHeaderProps {
  profile: ResolvedAgentProfile
  onEdit?: () => void
}

export function AgentHeader({ profile, onEdit }: AgentHeaderProps) {
  const initial = profile.profile.name.charAt(0).toUpperCase()
  const skillCount = profile.skills.length
  const mcpCount = profile.mcpServers.length

  return (
    <>
      <div className="flex items-start gap-4 px-5 py-5 border-b border-border/40">
        <div
          className="flex size-12 shrink-0 items-center justify-center rounded-xl text-xl font-bold text-white"
          style={{ background: 'linear-gradient(135deg, oklch(0.67 0.16 58), oklch(0.5 0.16 30))' }}
        >
          {initial}
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h1 className="font-display text-xl font-bold text-foreground">
              {profile.profile.name}
            </h1>
            {onEdit && (
              <button onClick={onEdit} className="rounded-md p-1 text-muted-foreground/40 hover:text-primary transition-colors">
                <Pencil className="size-3.5" />
              </button>
            )}
          </div>
          <p className="mt-0.5 text-[13px] text-muted-foreground">
            {profile.profile.description || 'No description'}
          </p>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {(profile.profile.providers ?? []).map((p) => (
              <span
                key={p}
                className="rounded-md bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-primary"
              >
                {p}
              </span>
            ))}
            <span className="rounded-md bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
              {profile.profile.version}
            </span>
            <span className="rounded-md bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground">
              {skillCount} skills / {mcpCount} MCP
            </span>
          </div>
        </div>
      </div>
    </>
  )
}
