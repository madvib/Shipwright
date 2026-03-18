import { Link, useRouterState } from '@tanstack/react-router'
import { Sun, Moon } from 'lucide-react'
import { useEffect, useState } from 'react'
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
  const [mode, setMode] = useState<ThemeMode>('dark')
  useEffect(() => {
    const stored = window.localStorage.getItem('theme')
    const initial: ThemeMode = stored === 'light' || stored === 'dark'
      ? stored
      : window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    setMode(initial)
    applyTheme(initial)
  }, [])
  const toggle = () => {
    const next = mode === 'dark' ? 'light' : 'dark'
    setMode(next)
    applyTheme(next)
  }
  return (
    <button
      onClick={toggle}
      className="flex items-center justify-center size-8 rounded-md border border-border/60 bg-card text-muted-foreground transition hover:text-foreground hover:border-border"
      title={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      {mode === 'dark' ? <Sun className="size-3.5" /> : <Moon className="size-3.5" />}
    </button>
  )
}

const STUDIO_TABS = [
  { to: '/studio/profiles', label: 'Profiles' },
  { to: '/studio/skills', label: 'Skills' },
  { to: '/studio/mcp', label: 'MCP' },
  { to: '/studio/export', label: 'Export' },
  { to: '/studio/templates', label: 'Registry' },
] as const

export default function Header() {
  const { data: session, isPending } = authClient.useSession()
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const isStudio = pathname.startsWith('/studio')

  return (
    <header className="sticky top-0 z-50 border-b border-border/60 bg-background/80 backdrop-blur-md">
      <nav className="flex items-center gap-6 px-6 py-3">
        <Link to="/" className="flex items-center gap-2.5 no-underline">
          <img src="/ship-logos/ship_logo.svg" alt="Ship" className="size-6" />
          <span className="font-display text-lg font-bold tracking-[-0.05em] leading-none">SHIP</span>
        </Link>

        <div className="flex items-center gap-1 text-sm">
          <Link
            to="/studio"
            className="rounded-md px-3 py-1.5 text-muted-foreground transition hover:bg-muted hover:text-foreground [&.active]:bg-muted [&.active]:text-foreground"
            activeProps={{ className: 'active' }}
          >
            Studio
          </Link>
          <Link
            to="/canvas"
            className="rounded-md px-3 py-1.5 text-muted-foreground transition hover:bg-muted hover:text-foreground [&.active]:bg-muted [&.active]:text-foreground"
            activeProps={{ className: 'active' }}
          >
            Workflow
          </Link>
          {import.meta.env.DEV && (
            <Link
              to="/dev/jobs"
              className="rounded-md px-3 py-1.5 text-muted-foreground/50 transition hover:bg-muted hover:text-foreground [&.active]:bg-muted [&.active]:text-foreground"
              activeProps={{ className: 'active' }}
            >
              Jobs
            </Link>
          )}
        </div>

        {/* Studio sub-tabs — inline when on /studio/* */}
        {isStudio && (
          <>
            <div className="w-px h-5 bg-border/60" />
            <div className="flex items-center gap-0.5">
              {STUDIO_TABS.map((tab) => {
                const active = pathname === tab.to || (tab.to === '/studio/profiles' && pathname === '/studio')
                return (
                  <Link
                    key={tab.to}
                    to={tab.to as '/'}
                    className={`rounded-md px-2.5 py-1 text-xs font-medium transition ${
                      active
                        ? 'bg-muted text-foreground'
                        : 'text-muted-foreground/60 hover:text-muted-foreground hover:bg-muted/50'
                    }`}
                  >
                    {tab.label}
                  </Link>
                )
              })}
            </div>
          </>
        )}

        <div className="ml-auto flex items-center gap-3">
          {!isPending && !session?.user && (
            <button
              onClick={() => void authClient.signIn.social({ provider: 'github' })}
              className="inline-flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              Sign in with GitHub
            </button>
          )}
          {session?.user && (
            <div className="flex items-center gap-2">
              {session.user.image && (
                <img src={session.user.image} alt="" className="size-6 rounded-full" />
              )}
              <button
                onClick={() => void authClient.signOut()}
                className="text-xs text-muted-foreground transition hover:text-foreground"
              >
                Sign out
              </button>
            </div>
          )}
          <ThemeToggle />
        </div>
      </nav>
    </header>
  )
}
