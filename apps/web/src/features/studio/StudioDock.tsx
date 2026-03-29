import { useNavigate, useRouterState } from '@tanstack/react-router'
import { Users, Zap, Layers, Settings, Radio, PanelRightOpen, WifiOff } from 'lucide-react'
import { CliStatusPopover } from '#/features/studio/CliStatusPopover'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import type { Skill } from '@ship/ui'

const NAV_ITEMS = [
  { to: '/studio/agents', icon: Users, label: 'Agents', exact: false },
  { to: '/studio/skills', icon: Zap, label: 'Skills', exact: false },
  { to: '/studio/session', icon: Layers, label: 'Session', exact: false },
  { to: '/studio/settings', icon: Settings, label: 'Settings', exact: false },
] as const

interface StudioDockProps {
  previewOpen?: boolean
  showPreviewToggle?: boolean
  onTogglePreview?: () => void
  onAddSkill: (skill: Skill) => void
}

export function StudioDock({ previewOpen, showPreviewToggle = true, onTogglePreview, onAddSkill }: StudioDockProps) {
  const navigate = useNavigate()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'

  return (
    <div className="flex items-center border-b border-border bg-card/30 px-4 shrink-0">
      <nav aria-label="Studio navigation" className="flex items-center gap-0.5">
        {NAV_ITEMS.map((item) => {
          const isActive = item.exact
            ? pathname === item.to || pathname === item.to + '/'
            : pathname.startsWith(item.to)
          const Icon = item.icon

          return (
            <button
              key={item.to}
              aria-label={item.label}
              onClick={() => void navigate({ to: item.to as string })}
              className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium transition-colors ${
                isActive
                  ? 'text-primary border-b-2 border-primary'
                  : 'text-muted-foreground hover:text-foreground border-b-2 border-transparent'
              }`}
            >
              <Icon className="size-3.5" strokeWidth={isActive ? 2.2 : 1.8} />
              {item.label}
            </button>
          )
        })}
      </nav>

      <div className="flex-1" />

      {/* Offline indicator */}
      {!isConnected && (
        <div className="flex items-center gap-1 px-2 py-1 text-[10px] text-amber-500 font-medium">
          <WifiOff className="size-3" />
          <span className="hidden sm:inline">Offline</span>
        </div>
      )}

      {/* CLI status */}
      <CliStatusPopover onAddSkill={onAddSkill} />

      {/* Compiler output toggle */}
      {showPreviewToggle && (
        <button
          onClick={onTogglePreview}
          className={`hidden md:flex items-center gap-1.5 rounded-md px-3 py-1.5 ml-2 text-xs font-semibold transition ${
            previewOpen
              ? 'bg-primary text-primary-foreground'
              : 'bg-primary/10 text-primary hover:bg-primary/20'
          }`}
        >
          {previewOpen ? (
            <PanelRightOpen className="size-3.5" />
          ) : (
            <>
              <Radio className="size-3 animate-pulse" />
              <span>Preview</span>
            </>
          )}
          {previewOpen && 'Preview'}
        </button>
      )}
    </div>
  )
}
