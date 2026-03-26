import { useState, useEffect } from 'react'
import { Link } from '@tanstack/react-router'
import { Menu, X, Sun, Moon } from 'lucide-react'
import { authClient } from '#/lib/auth-client'

type ThemeMode = 'light' | 'dark'

function ThemeToggle() {
  const [mode, setMode] = useState<ThemeMode>(() => {
    try {
      const stored = localStorage.getItem('theme')
      if (stored === 'light' || stored === 'dark') return stored
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    } catch { return 'dark' }
  })
  useEffect(() => {
    document.documentElement.classList.remove('light', 'dark')
    document.documentElement.classList.add(mode)
    document.documentElement.setAttribute('data-theme', mode)
    document.documentElement.style.colorScheme = mode
    window.localStorage.setItem('theme', mode)
  }, [mode])
  return (
    <button
      onClick={() => setMode((p) => (p === 'dark' ? 'light' : 'dark'))}
      className="flex items-center justify-center size-8 rounded-md border border-border/60 bg-background/60 text-muted-foreground transition hover:text-foreground hover:border-border"
      title={mode === 'dark' ? 'Switch to light mode' : 'Switch to dark mode'}
    >
      {mode === 'dark' ? <Sun className="size-3.5" /> : <Moon className="size-3.5" />}
    </button>
  )
}

export default function LandingNav() {
  const { data: session } = authClient.useSession()
  const user = session?.user
  const [mobileOpen, setMobileOpen] = useState(false)

  return (
    <nav className="fixed inset-x-0 top-0 z-50 border-b border-border/40 bg-background/85 backdrop-blur-xl">
      <div className="flex h-14 items-center justify-between px-6 sm:px-10">
        <div className="flex items-center gap-8">
          <Link to="/" className="flex items-center gap-2 no-underline">
            <img src="/ship-logos/ship_logo.svg" alt="Ship" className="size-5" />
            <span className="font-display text-base font-extrabold tracking-[-0.04em] text-primary">SHIP</span>
          </Link>
          <div className="hidden items-center gap-1 sm:flex">
            <Link to="/studio" className="rounded-md px-3 py-1.5 text-[13px] text-muted-foreground transition hover:bg-muted hover:text-foreground">
              Studio
            </Link>
            <Link to="/registry" className="rounded-md px-3 py-1.5 text-[13px] text-muted-foreground transition hover:bg-muted hover:text-foreground">
              Registry
            </Link>
          </div>
        </div>
        <div className="flex items-center gap-2.5">
          <Link to="/studio" className="hidden rounded-lg bg-primary px-4 py-1.5 text-xs font-semibold text-primary-foreground no-underline transition hover:bg-primary/90 sm:inline-block">
            {user ? 'Open Studio' : 'Get started'}
          </Link>
          <ThemeToggle />
          <button
            onClick={() => setMobileOpen(!mobileOpen)}
            className="inline-flex items-center justify-center rounded-md p-1.5 text-muted-foreground transition hover:text-foreground sm:hidden"
            aria-label="Toggle menu"
          >
            {mobileOpen ? <X className="size-5" /> : <Menu className="size-5" />}
          </button>
        </div>
      </div>

      {/* Mobile menu */}
      {mobileOpen && (
        <div className="border-t border-border/40 bg-background px-6 pb-4 pt-2 sm:hidden">
          <Link to="/studio" onClick={() => setMobileOpen(false)} className="block rounded-md px-3 py-2 text-sm text-muted-foreground no-underline transition hover:bg-muted hover:text-foreground">
            Studio
          </Link>
          <Link to="/registry" onClick={() => setMobileOpen(false)} className="block rounded-md px-3 py-2 text-sm text-muted-foreground no-underline transition hover:bg-muted hover:text-foreground">
            Registry
          </Link>
          {!user && (
            <Link
              to="/studio"
              onClick={() => setMobileOpen(false)}
              className="mt-2 flex w-full items-center justify-center gap-1.5 rounded-lg bg-primary px-3.5 py-2 text-sm font-semibold text-primary-foreground no-underline transition hover:bg-primary/90"
            >
              Get started
            </Link>
          )}
        </div>
      )}
    </nav>
  )
}
