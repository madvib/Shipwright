import { useState } from 'react'
import { Link } from '@tanstack/react-router'
import { Menu, X } from 'lucide-react'
import { ThemeToggle } from '@ship/primitives'
import { authClient } from '#/lib/auth-client'

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
          {user ? (
            <Link to="/studio" className="rounded-lg bg-primary px-4 py-1.5 text-xs font-semibold text-primary-foreground no-underline transition hover:bg-primary/90">
              Open Studio
            </Link>
          ) : (
            <>
              <button
                onClick={() => void authClient.signIn.social({ provider: 'github', callbackURL: '/studio' })}
                className="hidden items-center gap-1.5 rounded-lg border border-border bg-transparent px-3.5 py-1.5 text-xs text-muted-foreground transition hover:border-border/80 hover:text-foreground sm:inline-flex"
              >
                <svg className="size-3.5" viewBox="0 0 16 16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
                Sign in with GitHub
              </button>
              <Link to="/studio" className="hidden rounded-lg bg-primary px-4 py-1.5 text-xs font-semibold text-primary-foreground no-underline transition hover:bg-primary/90 sm:inline-block">
                Get started
              </Link>
            </>
          )}
          <ThemeToggle variant="icon" />
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
