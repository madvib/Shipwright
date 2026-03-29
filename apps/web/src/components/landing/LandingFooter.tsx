import { Link } from '@tanstack/react-router'

export function LandingFooter() {
  const year = new Date().getFullYear()

  return (
    <footer className="border-t border-border/40 px-6 py-6 sm:px-10">
      <div className="mx-auto flex max-w-[62rem] flex-col items-center justify-between gap-3 sm:flex-row">
        <span className="text-[11px] text-muted-foreground/30">
          Ship v0.1.0 / getship.dev / {year}
        </span>
        <div className="flex items-center gap-2">
          <Link to="/registry" className="text-[11px] text-muted-foreground/40 transition hover:text-muted-foreground no-underline px-2 py-2 min-h-[44px] flex items-center">
            Registry
          </Link>
          <a href="https://github.com/madvib/Ship" target="_blank" rel="noopener noreferrer" className="text-[11px] text-muted-foreground/40 transition hover:text-muted-foreground px-2 py-2 min-h-[44px] flex items-center">
            GitHub
          </a>
          <a href="https://x.com/themadvib" target="_blank" rel="noopener noreferrer" className="text-[11px] text-muted-foreground/40 transition hover:text-muted-foreground px-2 py-2 min-h-[44px] flex items-center">
            X
          </a>
        </div>
      </div>
    </footer>
  )
}
