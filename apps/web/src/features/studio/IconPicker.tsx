import { useState } from 'react'
import { TechIcon, TECH_STACK_LIST } from './TechIcon'

const ACCENT_COLORS = [
  '#61dafb', '#ce422b', '#3178c6', '#f05028',
  '#7c3aed', '#22c55e', '#f59e0b', '#f43f5e', '#94a3b8',
]

interface IconPickerProps {
  icon: string
  accentColor: string
  name: string
  onChange: (icon: string, accentColor: string) => void
}

export function IconPicker({ icon, accentColor, name, onChange }: IconPickerProps) {
  const [open, setOpen] = useState(false)
  const [localStack, setLocalStack] = useState<string>(icon)
  const [localColor, setLocalColor] = useState(accentColor)
  const [search, setSearch] = useState('')

  const filtered = search.trim()
    ? TECH_STACK_LIST.filter((t) => t.id.includes(search.toLowerCase()))
    : TECH_STACK_LIST

  const apply = () => {
    onChange(localStack, localColor)
    setOpen(false)
  }

  const cancel = () => {
    setLocalStack(icon)
    setLocalColor(accentColor)
    setOpen(false)
  }

  return (
    <div className="relative">
      {/* Trigger tile */}
      <button
        onClick={() => setOpen((v) => !v)}
        className="cursor-pointer transition-opacity hover:opacity-80"
        title="Change icon"
      >
        <TechIcon stack={icon} size={40} />
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-10" onClick={cancel} />
          <div className="absolute z-20 mt-2 left-0 w-72 rounded-xl border border-border bg-card shadow-xl overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between px-3 py-2 border-b border-border">
              <span className="text-xs font-semibold text-foreground">Choose icon</span>
              <button
                onClick={cancel}
                className="text-sm text-muted-foreground hover:text-foreground transition-colors leading-none"
              >
                ×
              </button>
            </div>

            {/* Search */}
            <div className="border-b border-border px-3 py-2">
              <input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search icons... (react, rust, python...)"
                className="w-full bg-muted/30 border border-border/60 rounded px-2 py-1 text-xs text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus-visible:ring-2 focus-visible:ring-violet-500/50 focus:border-violet-500/50 transition-colors"
              />
            </div>

            {/* Body */}
            <div className="px-3 pt-3 pb-2">
              {/* Color accent */}
              <div className="text-[11px] font-semibold uppercase tracking-widest text-muted-foreground mb-2">
                COLOR ACCENT
              </div>
              <div className="flex gap-1.5 mb-3">
                {ACCENT_COLORS.map((c) => (
                  <button
                    key={c}
                    onClick={() => setLocalColor(c)}
                    className="size-5 rounded-full shrink-0 transition-[outline]"
                    style={{
                      background: c,
                      outline: localColor === c ? `2px solid ${c}` : '2px solid transparent',
                      outlineOffset: 2,
                    }}
                  />
                ))}
              </div>

              {/* Tech icons */}
              <div className="text-[11px] font-semibold uppercase tracking-widest text-muted-foreground mb-2">
                TECH ICONS
              </div>
              <div className="grid grid-cols-8 gap-1 mb-2 max-h-36 overflow-y-auto">
                {filtered.map((tech) => {
                  const selected = localStack === tech.id
                  return (
                    <button
                      key={tech.id}
                      onClick={() => setLocalStack(tech.id)}
                      title={tech.id}
                      className="flex items-center justify-center rounded-md transition-colors"
                      style={{
                        width: 34,
                        height: 34,
                        border: `1px solid ${selected ? tech.border : 'var(--color-border)'}`,
                        background: selected ? tech.bg : 'var(--color-muted)',
                      }}
                    >
                      <TechIcon stack={tech.id} size={26} style={{ border: 'none', background: 'transparent', borderRadius: 0 }} />
                    </button>
                  )
                })}
              </div>
            </div>

            {/* Footer: preview + apply */}
            <div className="flex items-center gap-2 px-3 py-2 border-t border-border bg-card/80">
              <TechIcon stack={localStack} size={32} />
              <span className="text-xs text-muted-foreground flex-1 overflow-hidden text-ellipsis whitespace-nowrap">
                {name || 'Profile'}
              </span>
              <button
                onClick={apply}
                className="h-7 px-3 bg-violet-600 dark:bg-violet-500 hover:bg-violet-500 dark:hover:bg-violet-400 transition-colors rounded text-[10px] text-primary-foreground font-medium"
              >
                Apply
              </button>
            </div>
          </div>
        </>
      )}
    </div>
  )
}
