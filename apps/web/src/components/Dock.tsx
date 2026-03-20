import { useState } from 'react'
import { useLocation } from '@tanstack/react-router'
import { Users, Zap, Search, Code2, Loader2 } from 'lucide-react'

// ── Nav item config ─────────────────────────────────────────────────────────

interface NavItem {
  icon: React.ElementType
  label: string
  /** Used for route-based active detection when activeSection prop is not provided */
  match: (pathname: string) => boolean
}

const NAV_ITEMS: NavItem[] = [
  {
    icon: Users,
    label: 'Agents',
    match: (p) => p === '/studio' || p.startsWith('/studio/agents') || p.startsWith('/studio/profiles'),
  },
  {
    icon: Zap,
    label: 'Skills',
    match: (p) => p.startsWith('/studio/skills'),
  },
  {
    icon: Search,
    label: 'Registry',
    match: (p) => p.startsWith('/studio/registry'),
  },
]

// ── Tooltip ─────────────────────────────────────────────────────────────────

function Tooltip({ label, children }: { label: string; children: React.ReactNode }) {
  const [show, setShow] = useState(false)
  return (
    <div
      className="relative"
      onMouseEnter={() => setShow(true)}
      onMouseLeave={() => setShow(false)}
    >
      {children}
      {show && (
        <span className="pointer-events-none absolute -top-9 left-1/2 -translate-x-1/2 whitespace-nowrap rounded-md bg-foreground/90 px-2.5 py-1 text-[10px] font-medium text-background shadow-lg">
          {label}
          <span className="absolute -bottom-1 left-1/2 -translate-x-1/2 size-2 rotate-45 bg-foreground/90" />
        </span>
      )}
    </div>
  )
}

// ── Dock ─────────────────────────────────────────────────────────────────────

export interface DockProps {
  onCompile?: () => void
  isCompiling?: boolean
  /** Override the active section for hash-based nav within studio */
  activeSection?: string
  onNavigate?: (section: string) => void
}

export default function Dock({ onCompile, isCompiling, activeSection, onNavigate }: DockProps) {
  const location = useLocation()
  const pathname = location.pathname

  return (
    <div className="fixed bottom-5 left-1/2 z-50 -translate-x-1/2">
      <nav
        className="flex items-center gap-1 rounded-2xl border border-border/60 bg-card/95 px-2 py-2 shadow-xl shadow-black/10 backdrop-blur-xl"
        role="navigation"
        aria-label="Studio navigation"
      >
        {/* Navigation items */}
        {NAV_ITEMS.map((item) => {
          const Icon = item.icon
          const isActive = activeSection
            ? activeSection === item.label.toLowerCase()
            : item.match(pathname)

          return (
            <Tooltip key={item.label} label={item.label}>
              <button
                onClick={() => onNavigate?.(item.label.toLowerCase())}
                className={`flex size-[38px] items-center justify-center rounded-lg transition-all duration-200 ${
                  isActive
                    ? 'bg-primary/12 text-primary shadow-sm'
                    : 'text-muted-foreground hover:-translate-y-0.5 hover:bg-muted/60 hover:text-foreground'
                }`}
                aria-current={isActive ? 'page' : undefined}
              >
                <Icon className="size-[18px]" strokeWidth={isActive ? 2.2 : 1.8} />
              </button>
            </Tooltip>
          )
        })}

        {/* Separator */}
        <div className="mx-1 h-6 w-px bg-border/60" />

        {/* Compile button */}
        <button
          onClick={onCompile}
          disabled={isCompiling}
          className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground transition-all duration-200 hover:-translate-y-0.5 hover:opacity-90 active:translate-y-0 disabled:opacity-60 disabled:hover:translate-y-0"
        >
          {isCompiling ? (
            <Loader2 className="size-4 animate-spin" />
          ) : (
            <Code2 className="size-4" />
          )}
          <span className="hidden sm:inline">Compile</span>
        </button>
      </nav>
    </div>
  )
}
