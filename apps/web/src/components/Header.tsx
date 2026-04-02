import { Link, useNavigate, useRouterState } from '@tanstack/react-router'
import { LogOut, Settings, Users, Zap, Server, Upload, Layers, ChevronDown } from 'lucide-react'
import { useEffect, useState, useRef } from 'react'
import { ThemeToggle } from '@ship/primitives'
import { authClient } from '#/lib/auth-client'
import { CliStatusPopover } from '#/features/studio/CliStatusPopover'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'

function NavDropdown({ label, href, items, isActive }: {
  label: string
  href: string
  items: { icon: typeof Users; label: string; href: string; desc?: string }[]
  isActive: boolean
}) {
  const [open, setOpen] = useState(false)
  const timeout = useRef<ReturnType<typeof setTimeout>>(undefined)

  const enter = () => { clearTimeout(timeout.current); setOpen(true) }
  const leave = () => { timeout.current = setTimeout(() => setOpen(false), 150) }

  return (
    <div className="relative" onMouseEnter={enter} onMouseLeave={leave}>
      <Link
        to={href as string}
        onClick={() => setOpen(false)}
        className={`rounded-md px-3 py-2.5 text-sm transition select-none no-underline ${
          isActive ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'
        }`}
      >
        {label}
      </Link>
      {open && (
        <div className="absolute right-0 top-full pt-1 z-50">
          <div className="w-56 rounded-xl border border-border/60 bg-card shadow-lg py-1.5 animate-in fade-in slide-in-from-top-1 duration-150">
            {items.map((item) => {
              const Icon = item.icon
              return (
                <Link
                  key={item.label}
                  to={item.href as string}
                  onClick={() => setOpen(false)}
                  className="flex items-center gap-2.5 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors no-underline"
                >
                  <Icon className="size-3.5 shrink-0" />
                  <div>
                    <div className="font-medium">{item.label}</div>
                    {item.desc && <div className="text-[10px] text-muted-foreground/60 mt-0.5">{item.desc}</div>}
                  </div>
                </Link>
              )
            })}
          </div>
        </div>
      )}
    </div>
  )
}

function UserMenu({ user }: { user: { name: string; email?: string | null; image?: string | null } }) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false)
    }
    const handleEsc = (e: KeyboardEvent) => { if (e.key === 'Escape') setOpen(false) }
    document.addEventListener('mousedown', handleClick)
    document.addEventListener('keydown', handleEsc)
    return () => { document.removeEventListener('mousedown', handleClick); document.removeEventListener('keydown', handleEsc) }
  }, [open])

  const initial = user.name.charAt(0).toUpperCase()

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen((p) => !p)}
        className="flex items-center justify-center size-8 rounded-md border border-border/60 bg-card text-muted-foreground transition hover:text-foreground hover:border-border"
      >
        {user.image ? (
          <img src={user.image} alt="" className="size-6 rounded-md object-cover" />
        ) : (
          <span className="text-xs font-bold text-primary">{initial}</span>
        )}
      </button>
      {open && (
        <div className="absolute right-0 top-full mt-1.5 w-48 rounded-xl border border-border/60 bg-card shadow-lg py-1 z-50 animate-in fade-in slide-in-from-top-1 duration-150">
          <div className="flex items-center gap-2.5 px-3 py-2.5 border-b border-border/40">
            <span className="size-6 rounded-full bg-primary/15 flex items-center justify-center text-[11px] font-bold text-primary">{initial}</span>
            <div className="min-w-0">
              <p className="text-xs font-semibold text-foreground truncate">{user.name}</p>
              {user.email && <p className="text-[10px] text-muted-foreground/70 truncate">{user.email}</p>}
            </div>
          </div>
          <Link to={"/studio/settings" as string} onClick={() => setOpen(false)} className="w-full flex items-center gap-2 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors no-underline">
            <Settings className="size-3.5" /> Settings
          </Link>
          <button onClick={() => { setOpen(false); void authClient.signOut() }} className="w-full flex items-center gap-2 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors">
            <LogOut className="size-3.5" /> Sign out
          </button>
        </div>
      )}
    </div>
  )
}

const STUDIO_ITEMS = [
  { icon: Users, label: 'My Agents', href: '/studio/agents', desc: 'Configure your AI agents' },
  { icon: Zap, label: 'Skills IDE', href: '/studio/skills', desc: 'Create and edit skills' },
  { icon: Layers, label: 'Session', href: '/studio/session', desc: 'Canvas, artifacts, and annotations' },
  { icon: Settings, label: 'Settings', href: '/studio/settings', desc: 'Account and defaults' },
]

const STUDIO_NAV = [
  { to: '/studio/agents', icon: Users, label: 'Agents' },
  { to: '/studio/skills', icon: Zap, label: 'Skills' },
  { to: '/studio/session', icon: Layers, label: 'Session' },
  { to: '/studio/settings', icon: Settings, label: 'Settings' },
] as const

function StudioNav({ pathname }: { pathname: string }) {
  const navigate = useNavigate()
  const [hoverIdx, setHoverIdx] = useState<number | null>(null)
  return (
    <nav aria-label="Studio navigation" className="flex items-center gap-1">
      {STUDIO_NAV.map((item, i) => {
        const isActive = pathname.startsWith(item.to)
        const Icon = item.icon
        return (
          <button
            key={item.to}
            onClick={() => void navigate({ to: item.to as string })}
            onMouseEnter={() => setHoverIdx(i)}
            onMouseLeave={() => setHoverIdx(null)}
            className={`relative flex items-center justify-center size-8 rounded-lg transition-colors ${
              isActive
                ? 'text-primary bg-primary/10'
                : 'text-muted-foreground hover:text-foreground hover:bg-muted/40'
            }`}
          >
            <Icon className="size-4" strokeWidth={isActive ? 2.2 : 1.8} />
            {hoverIdx === i && (
              <span className="absolute top-full mt-1.5 left-1/2 -translate-x-1/2 whitespace-nowrap rounded-md border border-border/50 bg-popover px-2 py-1 text-[11px] font-medium text-foreground shadow-md animate-in fade-in slide-in-from-top-1 duration-150 pointer-events-none z-50">
                {item.label}
              </span>
            )}
          </button>
        )
      })}
    </nav>
  )
}

const REGISTRY_ITEMS = [
  { icon: Zap, label: 'Skills', href: '/registry', desc: 'Browse agent skills' },
  { icon: Users, label: 'Agents', href: '/registry', desc: 'Pre-built agent configs' },
  { icon: Server, label: 'MCP Servers', href: '/registry', desc: 'Tool server integrations' },
  { icon: Upload, label: 'Publish', href: '/registry', desc: 'Share your packages' },
]

function WorkspacePicker() {
  const { connected, workspaces, agents } = useDaemon()
  const mcp = useLocalMcpContext()
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  const activeWorkspace = workspaces.find((w) => w.status === 'active')
  const activeAgentCount = agents.length

  useEffect(() => {
    if (!open) return
    const handleClick = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false)
    }
    const handleEsc = (e: KeyboardEvent) => { if (e.key === 'Escape') setOpen(false) }
    document.addEventListener('mousedown', handleClick)
    document.addEventListener('keydown', handleEsc)
    return () => {
      document.removeEventListener('mousedown', handleClick)
      document.removeEventListener('keydown', handleEsc)
    }
  }, [open])

  const activate = (branch: string) => {
    setOpen(false)
    if (mcp) void mcp.callTool('activate_workspace', { branch })
  }

  return (
    <div ref={ref} className="flex items-center gap-2">
      <span
        className={`w-2 h-2 rounded-full shrink-0 ${connected ? 'bg-emerald-500' : 'bg-red-500'}`}
        title={connected ? 'Daemon connected' : 'Daemon offline'}
      />
      <div className="relative">
        <button
          onClick={() => setOpen((p) => !p)}
          className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors font-mono"
        >
          <span>{activeWorkspace?.branch ?? '--'}</span>
          <ChevronDown className="size-3 opacity-50" />
        </button>
        {open && workspaces.length > 0 && (
          <div className="absolute left-1/2 -translate-x-1/2 top-full mt-1.5 w-56 rounded-xl border border-border/60 bg-card shadow-lg py-1 z-50 animate-in fade-in slide-in-from-top-1 duration-150">
            {workspaces.map((ws) => (
              <button
                key={ws.branch}
                onClick={() => activate(ws.branch)}
                className="w-full flex items-center gap-2 px-3 py-2 text-left text-xs hover:bg-muted transition-colors"
              >
                <span
                  className={`w-1.5 h-1.5 rounded-full shrink-0 ${ws.status === 'active' ? 'bg-emerald-500' : 'bg-muted-foreground/30'}`}
                />
                <span className="font-mono truncate flex-1 text-foreground/80">{ws.branch}</span>
                {ws.active_agent && (
                  <span className="text-[9px] text-muted-foreground/50 truncate max-w-[60px]">{ws.active_agent}</span>
                )}
              </button>
            ))}
          </div>
        )}
      </div>
      {connected && activeAgentCount > 0 && (
        <span className="text-[10px] text-muted-foreground/50 tabular-nums">{activeAgentCount} active</span>
      )}
    </div>
  )
}

export default function Header() {
  const { data: session } = authClient.useSession()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const user = session?.user

  const rawSegments = pathname.split('/').filter(Boolean)
  const isStudio = pathname.startsWith('/studio')
  const isRegistry = pathname.startsWith('/registry')
  // Breadcrumb segments — agent ID stays as-is (no MCP context outside studio)
  const segments = rawSegments

  return (
    <header className="sticky top-0 z-50 border-b border-border/60 bg-background/80 backdrop-blur-md">
      <nav className="flex items-center gap-4 px-5 h-12">
        <Link to="/" className="flex items-center gap-2 no-underline shrink-0">
          <img src="/ship-logos/ship_logo.svg" alt="Ship" className="size-5" />
          <span className="font-display text-base font-bold tracking-[-0.04em] leading-none">SHIP</span>
        </Link>

        {segments.length > 0 && (
          <div className="flex items-center gap-1 text-sm text-muted-foreground min-w-0">
            <span className="text-border">/</span>
            {segments.map((seg, i) => {
              const path = '/' + rawSegments.slice(0, i + 1).join('/')
              const isLast = i === segments.length - 1
              return (
                <span key={path} className="flex items-center gap-1 min-w-0">
                  {isLast ? (
                    <span className="text-foreground font-medium truncate">{seg}</span>
                  ) : (
                    <>
                      <Link to={path as string} className="hover:text-foreground transition-colors no-underline truncate">{seg}</Link>
                      <span className="text-border">/</span>
                    </>
                  )}
                </span>
              )
            })}
          </div>
        )}

        <div className="flex-1" />

        {isStudio && (
          <div className="hidden sm:flex items-center gap-4">
            <WorkspacePicker />
            <StudioNav pathname={pathname} />
          </div>
        )}

        <div className="flex-1" />

        <div className="hidden sm:flex items-center gap-1">
          <NavDropdown label="Studio" href="/studio/agents" items={STUDIO_ITEMS} isActive={isStudio} />
          <NavDropdown label="Registry" href="/registry" items={REGISTRY_ITEMS} isActive={isRegistry} />
        </div>

        <div className="flex items-center gap-2">
          {isStudio && <CliStatusPopover onAddSkill={() => {}} />}
          {user && <UserMenu user={{ name: user.name, email: user.email, image: user.image }} />}
          <ThemeToggle variant="icon" />
        </div>
      </nav>
    </header>
  )
}
