import { Link } from '@tanstack/react-router'
import ThemeToggle from './ThemeToggle'

export default function Header() {
  return (
    <header className="sticky top-0 z-50 border-b border-border/60 bg-background/80 px-6 backdrop-blur-md">
      <nav className="mx-auto flex max-w-7xl items-center gap-4 py-3">
        <Link
          to="/"
          className="flex items-center gap-2 text-sm font-bold tracking-tight no-underline"
        >
          <span className="flex size-6 items-center justify-center rounded-md bg-primary text-[10px] font-black text-primary-foreground">
            S
          </span>
          <span>Ship</span>
          <span className="text-[10px] font-semibold text-muted-foreground tracking-wider uppercase">Studio</span>
        </Link>

        <div className="flex items-center gap-1 text-sm">
          <Link
            to="/"
            className="rounded-md px-3 py-1.5 text-muted-foreground transition hover:bg-muted hover:text-foreground [&.active]:bg-muted [&.active]:text-foreground"
            activeProps={{ className: 'active' }}
          >
            Home
          </Link>
          <Link
            to="/studio"
            className="rounded-md px-3 py-1.5 text-muted-foreground transition hover:bg-muted hover:text-foreground [&.active]:bg-muted [&.active]:text-foreground"
            activeProps={{ className: 'active' }}
          >
            Studio
          </Link>
        </div>

        <div className="ml-auto flex items-center gap-2">
          <ThemeToggle />
        </div>
      </nav>
    </header>
  )
}
