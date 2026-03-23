import { Pencil } from 'lucide-react'
import type { AgentProfile } from './types'

interface AgentStickyHeaderProps {
  profile: AgentProfile
  onEdit: () => void
}

export function AgentStickyHeader({ profile, onEdit }: AgentStickyHeaderProps) {
  const initial = profile.name.charAt(0).toUpperCase()

  return (
    <div className="flex items-center gap-3 border-b border-border/30 bg-background/80 backdrop-blur-sm px-5 h-12 shrink-0 sticky top-0 z-10">
      {/* Avatar */}
      <div
        className="flex size-7 shrink-0 items-center justify-center rounded-lg text-xs font-bold text-white"
        style={{
          background:
            'linear-gradient(135deg, oklch(0.67 0.16 58), oklch(0.5 0.16 30))',
        }}
      >
        {initial}
      </div>

      {/* Name */}
      <button
        onClick={onEdit}
        className="group flex items-center gap-1.5 min-w-0"
      >
        <span className="font-display text-sm font-bold text-foreground truncate">
          {profile.name}
        </span>
        <Pencil className="size-3 text-muted-foreground/0 group-hover:text-muted-foreground/60 transition-colors shrink-0" />
      </button>

      {/* Provider badges */}
      <div className="hidden sm:flex items-center gap-1 shrink-0">
        {profile.providers.map((p) => (
          <span
            key={p}
            className="rounded bg-primary/10 px-1.5 py-0.5 text-[9px] font-semibold text-primary leading-none"
          >
            {p}
          </span>
        ))}
      </div>

      {/* Version */}
      <span className="hidden sm:inline text-[10px] text-muted-foreground/50 tabular-nums">
        {profile.version}
      </span>

      {/* Spacer */}
      <div className="flex-1" />
    </div>
  )
}
