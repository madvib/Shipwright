import { Link, useRouterState } from '@tanstack/react-router'
import { Sun, Moon, LogOut, Settings, Users, Zap, Server, Upload } from 'lucide-react'
import { useEffect, useState, useRef } from 'react'
import { authClient } from '#/lib/auth-client'

type ThemeMode = 'light' | 'dark'

function applyTheme(mode: ThemeMode) {
  document.documentElement.classList.remove('light', 'dark')
  document.documentElement.classList.add(mode)
  document.documentElement.setAttribute('data-theme', mode)
  document.documentElement.style.colorScheme = mode
  window.localStorage.setItem('theme', mode)
}

function ThemeToggle() {
  const [mode, setMode] = useState<ThemeMode>(() => {
    try {
      const stored = localStorage.getItem('theme')
      if (stored === 'light' || stored === 'dark') return stored
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    } catch { return 'dark' }
  })
  useEffect(() => { applyTheme(mode) }, [mode])
  return (
    <button
      onClick={() => setMode((p) => (p === 'dark' ? 'light' : 'dark'))}
      className="flex items-center justify-center size-8 rounded-md border border-border/60 bg-card text-muted-foreground transition hover:text-foreground hover:border-border"
      title={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      {mode === 'dark' ? <Sun className="size-3.5" /> : <Moon className="size-3.5" />}
    </button>
  )
}

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
        className={`rounded-md px-3 py-1.5 text-sm transition select-none no-underline ${
          isActive ? 'text-foreground' : 'text-muted-foreground hover:text-foreground'
        }`}
      >
        {label}
      </Link>
      {open && (
        <div className="absolute left-0 top-full pt-1 z-50">
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
  { icon: Settings, label: 'Settings', href: '/studio/settings', desc: 'Account and defaults' },
]

const REGISTRY_ITEMS = [
  { icon: Zap, label: 'Skills', href: '/registry', desc: 'Browse agent skills' },
  { icon: Users, label: 'Agents', href: '/registry', desc: 'Pre-built agent configs' },
  { icon: Server, label: 'MCP Servers', href: '/registry', desc: 'Tool server integrations' },
  { icon: Upload, label: 'Publish', href: '/registry', desc: 'Share your packages' },
]

export default function Header() {
  const { data: session, isPending } = authClient.useSession()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const user = session?.user

  const segments = pathname.split('/').filter(Boolean)
  const isStudio = pathname.startsWith('/studio')
  const isRegistry = pathname.startsWith('/registry')

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
              const path = '/' + segments.slice(0, i + 1).join('/')
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

        <div className="hidden sm:flex items-center gap-1">
          <NavDropdown label="Studio" href="/studio/agents" items={STUDIO_ITEMS} isActive={isStudio} />
          <NavDropdown label="Registry" href="/registry" items={REGISTRY_ITEMS} isActive={isRegistry} />
        </div>

        <div className="flex items-center gap-2">
          {!isPending && !user && (
            <button
              onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: window.location.href })}
              className="inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              <svg className="size-3.5" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
              Sign in with GitHub
            </button>
          )}
          {user && <UserMenu user={{ name: user.name, email: user.email, image: user.image }} />}
          <ThemeToggle />
        </div>
      </nav>
    </header>
  )
}
