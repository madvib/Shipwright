import { Link } from '@tanstack/react-router'
import { ThemeToggle } from '@ship/primitives'
import { authClient } from '#/lib/auth-client'

export default function Header() {
  const { data: session, isPending } = authClient.useSession()

  return (
    <header className="sticky top-0 z-50 border-b border-border/60 bg-background/80 px-6 backdrop-blur-md">
      <nav className="mx-auto flex max-w-7xl items-center gap-6 py-3">
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
