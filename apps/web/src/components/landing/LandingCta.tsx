import { Link } from '@tanstack/react-router'
import { Zap } from 'lucide-react'

export function LandingCta() {
  return (
    <section className="border-t border-border/40 bg-gradient-to-b from-background to-primary/3 px-6 py-16 text-center sm:px-10 sm:py-20">
      <h2 className="mb-2 font-display text-3xl font-extrabold sm:text-4xl">
        Start deploying agents
      </h2>
      <p className="mb-7 text-[15px] text-muted-foreground">
        Open source. Free. No account required.
      </p>
      <div className="mb-5 flex flex-wrap items-center justify-center gap-3">
        <Link
          to="/studio"
          className="inline-flex items-center gap-2 rounded-xl bg-primary px-7 py-3 text-sm font-semibold text-primary-foreground no-underline transition hover:-translate-y-0.5 hover:bg-primary/90"
        >
          <Zap className="size-4" />
          Open Studio
        </Link>
        <a
          href="https://github.com/madvib/Ship"
          target="_blank"
          rel="noopener noreferrer"
          className="inline-flex items-center gap-2 rounded-xl border border-border bg-transparent px-7 py-3 text-sm font-semibold text-muted-foreground no-underline transition hover:-translate-y-0.5 hover:border-border/80 hover:text-foreground"
        >
          <svg
            viewBox="0 0 16 16"
            className="size-4"
            fill="currentColor"
            aria-hidden="true"
          >
            <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.012 8.012 0 0 0 16 8c0-4.42-3.58-8-8-8z" />
          </svg>
          View on GitHub
        </a>
      </div>
      <p className="text-[11px] text-muted-foreground/40">
        Open source on GitHub
      </p>
    </section>
  )
}
