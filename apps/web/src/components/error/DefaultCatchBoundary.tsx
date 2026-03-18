import {
  Link,
  rootRouteId,
  useMatch,
  useRouter,
  type ErrorComponentProps,
} from '@tanstack/react-router'
import { AlertTriangle, RotateCcw, Home } from 'lucide-react'

export function DefaultCatchBoundary({ error }: ErrorComponentProps) {
  const router = useRouter()
  const isRoot = useMatch({
    strict: false,
    select: (state) => state.id === rootRouteId,
  })

  return (
    <div className="flex min-h-[60vh] w-full items-center justify-center p-6">
      <div className="w-full max-w-md text-center">
        <div className="mx-auto mb-6 flex size-14 items-center justify-center rounded-2xl border border-destructive/20 bg-destructive/10">
          <AlertTriangle className="size-6 text-destructive" />
        </div>

        <h1 className="mb-2 font-display text-xl font-bold text-foreground">
          Something went wrong
        </h1>
        <p className="mb-6 text-sm text-muted-foreground">
          {error.message || 'An unexpected error occurred.'}
        </p>

        <div className="flex items-center justify-center gap-3">
          <button
            onClick={() => router.invalidate()}
            className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:opacity-90"
          >
            <RotateCcw className="size-3.5" />
            Try Again
          </button>
          {isRoot ? (
            <Link
              to="/"
              className="inline-flex items-center gap-2 rounded-lg border border-border/60 bg-card px-4 py-2 text-sm font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
            >
              <Home className="size-3.5" />
              Home
            </Link>
          ) : (
            <Link
              to="/"
              className="inline-flex items-center gap-2 rounded-lg border border-border/60 bg-card px-4 py-2 text-sm font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
            >
              <Home className="size-3.5" />
              Go Home
            </Link>
          )}
        </div>
      </div>
    </div>
  )
}
