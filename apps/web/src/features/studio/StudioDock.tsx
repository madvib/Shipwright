import { useState } from 'react'
import { useNavigate, useRouterState } from '@tanstack/react-router'
import { Users, Zap, Settings, Eye, EyeOff } from 'lucide-react'

const NAV_ITEMS = [
  { to: '/studio', icon: Users, label: 'Agents', exact: true },
  { to: '/studio/skills', icon: Zap, label: 'Skills', exact: false },
  { to: '/studio/settings', icon: Settings, label: 'Settings', exact: false },
] as const

interface StudioDockProps {
  previewOpen?: boolean
  onTogglePreview?: () => void
}

export function StudioDock({ previewOpen, onTogglePreview }: StudioDockProps) {
  const navigate = useNavigate()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const [hoverIdx, setHoverIdx] = useState<number | null>(null)

  return (
    <div className="fixed bottom-4 left-1/2 -translate-x-1/2 z-50">
      <nav
        aria-label="Studio navigation"
        className="flex items-center gap-1 rounded-2xl border border-border/50 bg-card/80 px-2 py-1.5 shadow-lg shadow-foreground/[0.04] backdrop-blur-xl"
      >
        {NAV_ITEMS.map((item, i) => {
          const isActive = item.exact
            ? pathname === item.to || pathname === item.to + '/'
            : pathname.startsWith(item.to)
          const Icon = item.icon

          let scale = 1
          if (hoverIdx !== null) {
            const dist = Math.abs(i - hoverIdx)
            if (dist === 0) scale = 1.12
            else if (dist === 1) scale = 1.04
          }

          return (
            <button
              key={item.to}
              onClick={() => void navigate({ to: item.to as string })}
              onMouseEnter={() => setHoverIdx(i)}
              onMouseLeave={() => setHoverIdx(null)}
              className={`group relative flex items-center justify-center size-9 rounded-xl outline-none focus-visible:ring-2 focus-visible:ring-primary/50 transition-all duration-200 ease-out ${
                isActive
                  ? 'bg-primary/12 text-primary'
                  : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
              }`}
              style={{ transform: `scale(${scale})` }}
            >
              <Icon className="size-[17px]" strokeWidth={isActive ? 2.2 : 1.8} />
              {isActive && (
                <span className="absolute -bottom-0.5 left-1/2 -translate-x-1/2 w-3 h-[2px] rounded-full bg-primary" />
              )}
              {hoverIdx === i && (
                <span className="absolute bottom-full mb-2 left-1/2 -translate-x-1/2 whitespace-nowrap rounded-md border border-border/50 bg-popover px-2 py-1 text-[11px] font-semibold text-popover-foreground shadow-md animate-in fade-in slide-in-from-bottom-1 duration-150 pointer-events-none">
                  {item.label}
                </span>
              )}
            </button>
          )
        })}

        {/* Separator */}
        <div className="h-6 w-px bg-border/60 mx-1" />

        {/* Output preview toggle */}
        <button
          onClick={onTogglePreview}
          className={`flex items-center gap-1.5 rounded-xl px-3 py-1.5 text-xs font-semibold transition ${
            previewOpen
              ? 'bg-primary text-primary-foreground'
              : 'bg-muted/60 text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
        >
          {previewOpen ? <EyeOff className="size-3.5" /> : <Eye className="size-3.5" />}
          Output
        </button>
      </nav>
    </div>
  )
}
