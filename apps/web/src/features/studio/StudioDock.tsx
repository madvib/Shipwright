import { useRef, useState } from 'react'
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
  const dockRef = useRef<HTMLDivElement>(null)
  const [hoverIdx, setHoverIdx] = useState<number | null>(null)

  return (
    <>
      <style>{DOCK_CSS}</style>
      <div className="studio-dock-wrapper">
        <div ref={dockRef} className="studio-dock">
          {DOCK_ITEMS.map((item, i) => {
            const isActive = item.exact
              ? pathname === item.to
              : pathname.startsWith(item.to)
            const Icon = item.icon

            // Proximity scale: hovered = 1.35, neighbor = 1.15, else = 1
            let scale = 1
            if (hoverIdx !== null) {
              const dist = Math.abs(i - hoverIdx)
              if (dist === 0) scale = 1.35
              else if (dist === 1) scale = 1.15
              else if (dist === 2) scale = 1.05
            }

            return (
              <button
                key={item.to}
                onClick={() => void navigate({ to: item.to })}
                onMouseEnter={() => setHoverIdx(i)}
                onMouseLeave={() => setHoverIdx(null)}
                className={`studio-dock-item ${isActive ? 'active' : ''}`}
                style={{ transform: `scale(${scale})` }}
                title={item.label}
              >
                <Icon className="studio-dock-icon" />
                {hoverIdx === i && (
                  <span className="studio-dock-label">{item.label}</span>
                )}
                {isActive && <span className="studio-dock-dot" />}
              </button>
            )
          })}
        </div>
      </div>
    </>
  )
}

const DOCK_CSS = `
  .studio-dock-wrapper {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 50;
    pointer-events: none;
  }

  .studio-dock {
    display: flex;
    align-items: flex-end;
    gap: 4px;
    padding: 6px 8px;
    border-radius: 18px;
    border: 1px solid hsl(var(--border) / 0.5);
    background: hsl(var(--card) / 0.85);
    backdrop-filter: blur(16px) saturate(1.5);
    box-shadow:
      0 8px 32px hsl(var(--foreground) / 0.08),
      0 0 0 1px hsl(var(--foreground) / 0.03);
    pointer-events: auto;
  }

  .studio-dock-item {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 40px;
    height: 40px;
    border-radius: 12px;
    border: 1px solid transparent;
    background: transparent;
    color: hsl(var(--muted-foreground));
    cursor: pointer;
    transition: transform 0.2s cubic-bezier(0.25, 0.46, 0.45, 0.94),
                background 0.15s, color 0.15s, border-color 0.15s;
    outline: none;
  }

  .studio-dock-item:hover {
    background: hsl(var(--muted) / 0.6);
    color: hsl(var(--foreground));
    border-color: hsl(var(--border) / 0.4);
  }

  .studio-dock-item.active {
    background: hsl(var(--primary) / 0.1);
    color: hsl(var(--primary));
    border-color: hsl(var(--primary) / 0.2);
  }

  .studio-dock-icon {
    width: 18px;
    height: 18px;
  }

  .studio-dock-label {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    white-space: nowrap;
    font-size: 11px;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 6px;
    background: hsl(var(--popover));
    color: hsl(var(--popover-foreground));
    border: 1px solid hsl(var(--border) / 0.5);
    box-shadow: 0 4px 12px hsl(var(--foreground) / 0.1);
    pointer-events: none;
    animation: dock-label-in 0.15s ease;
  }

  @keyframes dock-label-in {
    from { opacity: 0; transform: translateX(-50%) translateY(4px); }
    to   { opacity: 1; transform: translateX(-50%) translateY(0); }
  }

  .studio-dock-dot {
    position: absolute;
    bottom: 2px;
    left: 50%;
    transform: translateX(-50%);
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: hsl(var(--primary));
  }
`
