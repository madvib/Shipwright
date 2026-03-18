import { useState } from 'react'
import { useNavigate, useRouterState } from '@tanstack/react-router'
import {
  User, Puzzle, Server, Download, Package, LayoutGrid,
} from 'lucide-react'

const DOCK_ITEMS = [
  { to: '/studio', icon: LayoutGrid, label: 'Overview', exact: true as const },
  { to: '/studio/profiles', icon: User, label: 'Profiles', exact: false as const },
  { to: '/studio/skills', icon: Puzzle, label: 'Skills', exact: false as const },
  { to: '/studio/mcp', icon: Server, label: 'MCP', exact: false as const },
  { to: '/studio/export', icon: Download, label: 'Export', exact: false as const },
  { to: '/studio/templates', icon: Package, label: 'Registry', exact: false as const },
] as const

export function StudioDock() {
  const navigate = useNavigate()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const [hoverIdx, setHoverIdx] = useState<number | null>(null)

  return (
    <div className="fixed bottom-4 left-1/2 -translate-x-1/2 z-50">
      <nav className="flex items-end gap-1 rounded-2xl border border-border/50 bg-card/80 px-2 py-1.5 shadow-lg shadow-foreground/[0.04] backdrop-blur-xl">
        {DOCK_ITEMS.map((item, i) => {
          const isActive = item.exact
            ? pathname === item.to
            : pathname.startsWith(item.to)
          const Icon = item.icon

          // Subtle proximity: hovered = 1.12, neighbor = 1.04
          let scale = 1
          if (hoverIdx !== null) {
            const dist = Math.abs(i - hoverIdx)
            if (dist === 0) scale = 1.12
            else if (dist === 1) scale = 1.04
          }

          return (
            <button
              key={item.to}
              onClick={() => void navigate({ to: item.to })}
              onMouseEnter={() => setHoverIdx(i)}
              onMouseLeave={() => setHoverIdx(null)}
              className={`group relative flex items-center justify-center size-9 rounded-xl outline-none transition-all duration-200 ease-out ${
                isActive
                  ? 'bg-primary/12 text-primary'
                  : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
              }`}
              style={{ transform: `scale(${scale})` }}
            >
              <Icon className="size-[17px]" strokeWidth={isActive ? 2.2 : 1.8} />

              {/* Active indicator — thin bar */}
              {isActive && (
                <span className="absolute -bottom-0.5 left-1/2 -translate-x-1/2 w-3 h-[2px] rounded-full bg-primary" />
              )}

              {/* Tooltip */}
              {hoverIdx === i && (
                <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 whitespace-nowrap rounded-md border border-border/50 bg-popover px-2 py-1 text-[10px] font-semibold text-popover-foreground shadow-md animate-in fade-in slide-in-from-bottom-1 duration-150 pointer-events-none">
                  {item.label}
                </span>
              )}
            </button>
          )
        })}
      </nav>
    </div>
  )
}
