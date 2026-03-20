import { Link } from '@tanstack/react-router'
import { authClient } from '#/lib/auth-client'

export function LandingNav() {
  const { data: session } = authClient.useSession()
  const user = session?.user

  return (
    <nav className="fixed inset-x-0 top-0 z-50 flex h-14 items-center justify-between border-b border-border/40 bg-background/85 px-6 backdrop-blur-xl sm:px-10">
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
              className="rounded-lg border border-border bg-transparent px-3.5 py-1.5 text-xs text-muted-foreground transition hover:border-border/80 hover:text-foreground"
            >
              Sign in
            </button>
            <Link to="/studio" className="rounded-lg bg-primary px-4 py-1.5 text-xs font-semibold text-primary-foreground no-underline transition hover:bg-primary/90">
              Get started
            </Link>
          </>
        )}
      </div>
    </nav>
  )
}
