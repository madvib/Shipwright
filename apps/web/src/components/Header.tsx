import { Link } from '@tanstack/react-router'
import { Sun, Moon, LogOut } from 'lucide-react'
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

function UserMenu({ user }: { user: { name: string; image?: string | null } }) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    function handleClick(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClick)
    return () => document.removeEventListener('mousedown', handleClick)
  }, [open])

  return (
    <div ref={ref} className="relative">
      <button
        onClick={() => setOpen((prev) => !prev)}
        className="flex items-center gap-2 rounded-lg border border-border/60 bg-card px-2 py-1 transition hover:border-border"
      >
        {user.image ? (
          <img src={user.image} alt="" className="size-5 rounded-full" />
        ) : (
          <span className="size-5 rounded-full bg-primary/20 flex items-center justify-center text-[10px] font-bold text-primary">
            {user.name.charAt(0).toUpperCase()}
          </span>
        )}
        <span className="text-xs font-medium text-foreground max-w-[100px] truncate">
          {user.name}
        </span>
      </button>

      {open && (
        <div className="absolute right-0 top-full mt-1.5 w-44 rounded-lg border border-border/60 bg-card shadow-lg shadow-foreground/[0.04] py-1 animate-in fade-in slide-in-from-top-1 duration-150 z-50">
          <div className="px-3 py-2 border-b border-border/40">
            <p className="text-xs font-medium text-foreground truncate">{user.name}</p>
          </div>
          <button
            onClick={() => {
              setOpen(false)
              void authClient.signOut()
            }}
            className="w-full flex items-center gap-2 px-3 py-2 text-xs text-muted-foreground hover:bg-muted hover:text-foreground transition-colors"
          >
            <LogOut className="size-3.5" />
            Sign out
          </button>
        </div>
      )}
    </div>
  )
}

export default function Header() {
  const { data: session, isPending } = authClient.useSession()

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
            to="/studio/workflow"
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

        <div className="ml-auto flex items-center gap-3">
          {!isPending && !session?.user && (
            <button
              onClick={() => void authClient.signIn.social({
                provider: 'github',
                callbackURL: window.location.href,
              })}
              className="inline-flex items-center gap-2 rounded-lg border border-border/60 bg-card px-3 py-1.5 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              <svg className="size-4" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
              </svg>
              Sign in
            </button>
          )}
          {session?.user && (
            <UserMenu user={{ name: session.user.name, image: session.user.image }} />
          )}
          <ThemeToggle />
        </div>
      </nav>
    </header>
  )
}
