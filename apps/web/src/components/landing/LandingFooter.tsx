import { Link } from '@tanstack/react-router'

export function LandingFooter() {
  const year = new Date().getFullYear()

  return (
    <footer className="border-t border-border/40 px-6 py-6 sm:px-10">
      <div className="mx-auto flex max-w-[62rem] flex-col items-center justify-between gap-3 sm:flex-row">
        <span className="text-[11px] text-muted-foreground/30">
          v0.1.0 / {year}
        </span>
        <div className="flex items-center gap-4">
          <Link to="/registry" className="text-[11px] text-muted-foreground/40 transition hover:text-muted-foreground no-underline">
            Browse packages
          </Link>
          <a href="https://github.com/madvib/Ship" target="_blank" rel="noopener noreferrer" className="text-[11px] text-muted-foreground/40 transition hover:text-muted-foreground">
            GitHub
          </a>
          <a href="https://x.com/themadvib" target="_blank" rel="noopener noreferrer" className="text-[11px] text-muted-foreground/40 transition hover:text-muted-foreground">
            X
          </a>
        </div>
      </div>
    </footer>
  )
}
