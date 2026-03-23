import { useState } from 'react'
import { Pencil } from 'lucide-react'
import type { ResolvedAgentProfile } from './types'
import { getAgentIcon, setAgentIcon } from './agent-icons'
import { TechIcon, TECH_STACK_LIST, TECH_STACKS } from '#/features/studio/TechIcon'

interface AgentStickyHeaderProps {
  profile: ResolvedAgentProfile
  onEdit: () => void
}

export function AgentStickyHeader({ profile, onEdit }: AgentStickyHeaderProps) {
  const initial = profile.profile.name.charAt(0).toUpperCase()
  const [iconKey, setIconKey] = useState(() => getAgentIcon(profile.profile.id))
  const [pickerOpen, setPickerOpen] = useState(false)

  const handleIconSelect = (key: string) => {
    setAgentIcon(profile.profile.id, key)
    setIconKey(key)
    setPickerOpen(false)
  }

  return (
    <div className="flex items-center gap-3 border-b border-border/30 bg-background/80 backdrop-blur-sm px-5 h-12 shrink-0 sticky top-0 z-10">
      {/* Avatar / Icon — click to pick */}
      <div className="relative">
        <button
          onClick={() => setPickerOpen(!pickerOpen)}
          className="group relative"
          title="Change icon"
        >
          {iconKey && iconKey in TECH_STACKS ? (
            <TechIcon stack={iconKey} size={28} />
          ) : (
            <div
              className="flex size-7 shrink-0 items-center justify-center rounded-lg text-xs font-bold text-white"
              style={{ background: 'linear-gradient(135deg, oklch(0.67 0.16 58), oklch(0.5 0.16 30))' }}
            >
              {initial}
            </div>
          )}
          <span className="absolute inset-0 rounded-lg bg-black/0 group-hover:bg-black/20 transition-colors" />
        </button>

        {/* Icon picker dropdown */}
        {pickerOpen && (
          <>
            <div className="fixed inset-0 z-40" onClick={() => setPickerOpen(false)} />
            <div className="absolute top-full left-0 mt-1.5 z-50 rounded-xl border border-border/60 bg-popover shadow-lg p-2 animate-in fade-in slide-in-from-top-1 duration-150">
              <div className="grid grid-cols-6 gap-1 w-[180px]">
                {TECH_STACK_LIST.filter(t => t.slug !== null).map((tech) => (
                  <button
                    key={tech.id}
                    onClick={() => handleIconSelect(tech.id)}
                    className={`rounded-lg p-0.5 transition hover:bg-muted ${iconKey === tech.id ? 'ring-1 ring-primary bg-primary/10' : ''}`}
                    title={tech.id}
                  >
                    <TechIcon stack={tech.id} size={24} />
                  </button>
                ))}
              </div>
            </div>
          </>
        )}
      </div>

      {/* Name */}
      <button onClick={onEdit} className="group flex items-center gap-1.5 min-w-0">
        <span className="font-display text-sm font-bold text-foreground truncate">
          {profile.profile.name}
        </span>
        <Pencil className="size-3 text-muted-foreground/0 group-hover:text-muted-foreground/60 transition-colors shrink-0" />
      </button>

      {/* Version */}
      <span className="hidden sm:inline text-[10px] text-muted-foreground/50 tabular-nums">
        {profile.profile.version}
      </span>

      <div className="flex-1" />
    </div>
  )
}
