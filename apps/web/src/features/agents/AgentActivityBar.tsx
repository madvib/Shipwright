import { useState } from 'react'
import {
  Zap,
  Grid3X3,
  Shield,
  FileText,
  SlidersHorizontal,
} from 'lucide-react'

export const SECTION_DEFS = [
  { id: 'skills', label: 'Skills', icon: Zap, countable: true },
  { id: 'mcp', label: 'MCP Servers', icon: Grid3X3, countable: true },
  { id: 'permissions', label: 'Permissions', icon: Shield, countable: false },
  { id: 'rules', label: 'Rules', icon: FileText, countable: true },
  { id: 'providers', label: 'Providers', icon: SlidersHorizontal, countable: true },
] as const

interface AgentActivityBarProps {
  activeSection: string
  onSectionClick: (sectionId: string) => void
  counts: Record<string, number>
}

export function AgentActivityBar({
  activeSection,
  onSectionClick,
  counts,
}: AgentActivityBarProps) {
  const [hoverIdx, setHoverIdx] = useState<number | null>(null)

  return (
    <nav
      aria-label="Agent sections"
      className="hidden md:flex w-12 shrink-0 flex-col items-center gap-0.5 border-r border-border/30 bg-card/50 py-3"
    >
      {SECTION_DEFS.map((section, i) => {
        const Icon = section.icon
        const isActive = activeSection === section.id
        const count = section.countable ? counts[section.id] : undefined

        return (
          <button
            key={section.id}
            aria-label={section.label}
            onClick={() => onSectionClick(section.id)}
            onMouseEnter={() => setHoverIdx(i)}
            onMouseLeave={() => setHoverIdx(null)}
            className={`relative flex items-center justify-center size-9 rounded-lg outline-none transition-all duration-150 ${
              isActive
                ? 'text-primary'
                : 'text-muted-foreground/40 hover:text-muted-foreground/80'
            }`}
          >
            {/* Active indicator — left edge bar */}
            {isActive && (
              <span className="absolute left-0 top-1.5 bottom-1.5 w-[2px] rounded-r-full bg-primary" />
            )}

            <Icon className="size-[16px]" strokeWidth={isActive ? 2.2 : 1.6} />

            {/* Count badge */}
            {count !== undefined && count > 0 && (
              <span className="absolute -top-0.5 -right-0.5 flex items-center justify-center min-w-[14px] h-[14px] rounded-full bg-primary/15 text-[8px] font-bold text-primary px-0.5 tabular-nums">
                {count}
              </span>
            )}

            {/* Tooltip */}
            {hoverIdx === i && (
              <span className="absolute left-full ml-2 whitespace-nowrap rounded-md border border-border/50 bg-popover px-2 py-1 text-[11px] font-semibold text-popover-foreground shadow-md animate-in fade-in slide-in-from-left-1 duration-150 pointer-events-none z-50">
                {section.label}
              </span>
            )}
          </button>
        )
      })}
    </nav>
  )
}
