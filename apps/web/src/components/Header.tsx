import { Link, useRouterState } from '@tanstack/react-router'
import { Sun, Moon, LogOut, User, Settings } from 'lucide-react'
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
    } catch {
      return 'dark'
    }
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
          <Link to="/studio/settings" onClick={() => setOpen(false)} className="w-full flex items-center gap-2 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors no-underline">
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

export default function Header() {
  const { data: session, isPending } = authClient.useSession()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const user = session?.user

  // Breadcrumb segments from pathname
  const segments = pathname.split('/').filter(Boolean)

  return (
    <header className="sticky top-0 z-50 border-b border-border/60 bg-background/80 backdrop-blur-md">
      <nav className="flex items-center gap-4 px-5 h-12">
        {/* Logo */}
        <Link to="/" className="flex items-center gap-2 no-underline shrink-0">
          <img src="/ship-logos/ship_logo.svg" alt="Ship" className="size-5" />
          <span className="font-display text-base font-bold tracking-[-0.04em] leading-none">SHIP</span>
        </Link>

        {/* Breadcrumbs */}
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
                      <Link to={path} className="hover:text-foreground transition-colors no-underline truncate">{seg}</Link>
                      <span className="text-border">/</span>
                    </>
                  )}
                </span>
              )
            })}
          </div>
        )}

        {/* Spacer */}
        <div className="flex-1" />

        {/* Top-level links */}
        <div className="hidden sm:flex items-center gap-1 text-sm">
          <Link
            to="/studio"
            className="rounded-md px-3 py-1.5 text-muted-foreground transition hover:bg-muted hover:text-foreground [&.active]:text-foreground"
            activeProps={{ className: 'active' }}
          >
            Studio
          </Link>
          <Link
            to="/registry"
            className="rounded-md px-3 py-1.5 text-muted-foreground transition hover:bg-muted hover:text-foreground [&.active]:text-foreground"
            activeProps={{ className: 'active' }}
          >
            Registry
          </Link>
        </div>

        {/* Auth + theme */}
        <div className="flex items-center gap-2">
          {!isPending && !user && (
            <button
              onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: window.location.href })}
              className="inline-flex items-center gap-1.5 rounded-md border border-border/60 bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              Sign in
            </button>
          )}
          {user && <UserMenu user={{ name: user.name, email: user.email, image: user.image }} />}
          <ThemeToggle />
        </div>
      </nav>
    </header>
  )
}
