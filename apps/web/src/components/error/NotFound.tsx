import { Link } from '@tanstack/react-router'
import { SearchX, Home, ArrowLeft } from 'lucide-react'

export function NotFound() {
  return (
    <div className="flex min-h-[60vh] w-full items-center justify-center p-6">
      <div className="w-full max-w-md text-center">
        <div className="mx-auto mb-6 flex size-14 items-center justify-center rounded-2xl border border-amber-500/20 bg-amber-500/10">
          <SearchX className="size-6 text-amber-500" />
        </div>

        <h1 className="mb-2 font-display text-xl font-bold text-foreground">
          Page not found
        </h1>
        <p className="mb-6 text-sm text-muted-foreground">
          The page you are looking for does not exist or has been moved.
        </p>

        <div className="flex items-center justify-center gap-3">
          <Link
            to="/"
            className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:opacity-90 no-underline"
          >
            <Home className="size-3.5" />
            Go Home
          </Link>
          <Link
            to="/studio"
            className="inline-flex items-center gap-2 rounded-lg border border-border/60 bg-card px-4 py-2 text-sm font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
          >
            <ArrowLeft className="size-3.5" />
            Open Studio
          </Link>
        </div>
      </div>
    </div>
  )
}
