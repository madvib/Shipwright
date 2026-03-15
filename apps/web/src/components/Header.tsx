import { Link } from '@tanstack/react-router'
import { ThemeToggle } from '@ship/primitives'

export default function Header() {
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
        </div>

        <div className="ml-auto">
          <ThemeToggle />
        </div>
      </nav>
    </header>
  )
}
