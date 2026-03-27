import { useState, useEffect, useRef } from 'react'
import { Pencil, Trash2, Undo2 } from 'lucide-react'
import type { ResolvedAgentProfile } from './types'
import { getAgentIcon, setAgentIcon } from './agent-icons'
import { TechIcon, ICON_CATEGORIES, TECH_STACKS } from '#/features/studio/TechIcon'

interface AgentStickyHeaderProps {
  profile: ResolvedAgentProfile
  onEdit: () => void
  onDelete?: () => void
  onDiscard?: () => void
  isDraft?: boolean
}

export function AgentStickyHeader({ profile, onEdit, onDelete, onDiscard, isDraft }: AgentStickyHeaderProps) {
  const initial = profile.profile.name.charAt(0).toUpperCase()
  const [iconKey, setIconKey] = useState(() => getAgentIcon(profile.profile.id))
  const [pickerOpen, setPickerOpen] = useState(false)
  const [activeCategory, setActiveCategory] = useState<string>(ICON_CATEGORIES[0].id)

  const pickerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!pickerOpen) return
    const handler = (e: MouseEvent) => {
      if (pickerRef.current && !pickerRef.current.contains(e.target as Node)) {
        setPickerOpen(false)
      }
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [pickerOpen])

  const handleIconSelect = (key: string) => {
    setAgentIcon(profile.profile.id, key)
    setIconKey(key)
    setPickerOpen(false)
  }

  const activeCat = ICON_CATEGORIES.find((c) => c.id === activeCategory) ?? ICON_CATEGORIES[0]

  return (
    <div className="flex items-center gap-3 border-b border-border bg-background/80 backdrop-blur-sm px-5 h-12 shrink-0 sticky top-0 z-10">
      {/* Avatar / Icon -- click to pick */}
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
            <div ref={pickerRef} className="absolute top-full left-0 mt-1.5 z-50 rounded-xl border border-border/60 bg-popover shadow-xl p-3 animate-in fade-in slide-in-from-top-1 duration-150 w-[320px]">
              {/* Category tabs */}
              <div className="flex flex-wrap gap-0.5 mb-2">
                {ICON_CATEGORIES.map((cat) => (
                  <button
                    key={cat.id}
                    onClick={() => setActiveCategory(cat.id)}
                    className={`shrink-0 rounded-md px-2 py-1 text-[10px] font-medium transition ${
                      activeCategory === cat.id
                        ? 'bg-primary/10 text-primary'
                        : 'text-muted-foreground hover:text-foreground hover:bg-muted/40'
                    }`}
                  >
                    {cat.label}
                  </button>
                ))}
              </div>

              {/* Icon grid */}
              <div className="grid grid-cols-7 gap-1.5">
                {activeCat.keys.map((key) => (
                  <button
                    key={key}
                    onClick={() => handleIconSelect(key)}
                    className={`rounded-[10px] transition hover:brightness-110 ${
                      iconKey === key ? 'outline outline-2 outline-offset-1 outline-primary' : ''
                    }`}
                  >
                    <TechIcon stack={key} size={38} />
                  </button>
                ))}
              </div>
            </div>
        )}
      </div>

      {/* Name */}
      <button onClick={onEdit} className="group flex items-center gap-1.5 min-w-0">
        <span className="font-display text-sm font-bold text-foreground truncate">
          {profile.profile.name}
        </span>
        <Pencil className="size-3 text-muted-foreground group-hover:text-foreground transition-colors shrink-0" />
      </button>

      {/* Version */}
      <span className="hidden sm:inline text-[10px] text-muted-foreground tabular-nums">
        {profile.profile.version}
      </span>

      {isDraft && (
        <div className="flex items-center gap-1.5">
          <span className="text-[9px] font-medium px-1.5 py-0.5 rounded border border-amber-500/30 bg-amber-500/10 text-amber-500">
            Modified
          </span>
          {onDiscard && (
            <button
              onClick={() => {
                if (confirm('Discard unsaved changes?')) onDiscard()
              }}
              className="inline-flex items-center gap-1 rounded-md px-1.5 py-0.5 text-[9px] font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50 transition"
              title="Discard changes"
            >
              <Undo2 className="size-3" />
              Discard
            </button>
          )}
        </div>
      )}

      <div className="flex-1" />

      {/* Edit metadata button */}
      <button
        onClick={onEdit}
        className="rounded-md p-1.5 text-muted-foreground hover:text-foreground hover:bg-muted/50 transition"
        title="Edit metadata"
      >
        <Pencil className="size-3.5" />
      </button>

      {onDelete && (
        <button
          onClick={onDelete}
          className="rounded-md p-1.5 text-muted-foreground hover:text-red-400 hover:bg-red-500/10 transition"
          title="Delete agent"
        >
          <Trash2 className="size-3.5" />
        </button>
      )}
    </div>
  )
}
