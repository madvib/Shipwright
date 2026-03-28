import { useEffect } from 'react'
import { Link, type ErrorComponentProps } from '@tanstack/react-router'
import { AlertTriangle, RotateCcw, LayoutDashboard, Users } from 'lucide-react'
import { Button } from '@ship/primitives'
import { usePanicSave } from '#/features/agents/PanicSaveContext'

/**
 * Checks whether the error represents a "not found" scenario.
 * Covers thrown Response objects (404 status) and Error instances
 * whose message includes common not-found phrases.
 */
function isNotFoundError(error: unknown): boolean {
  if (error instanceof Response && error.status === 404) return true
  if (error instanceof Error) {
    const msg = error.message.toLowerCase()
    return msg.includes('not found') || msg.includes('404')
  }
  return false
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return 'An unexpected error occurred.'
}

export function StudioErrorBoundary({ error, reset }: ErrorComponentProps) {
  // Panic save any unsaved agent edits when an error boundary fires
  const { saveAll } = usePanicSave()
  useEffect(() => { saveAll() }, [saveAll])

  if (isNotFoundError(error)) {
    return (
      <div className="flex min-h-[60vh] w-full items-center justify-center p-6">
        <div className="w-full max-w-md text-center">
          <div className="mx-auto mb-6 flex size-14 items-center justify-center rounded-2xl border border-border/60 bg-muted/40">
            <Users className="size-6 text-muted-foreground" />
          </div>

          <h1 className="mb-2 font-display text-xl font-bold text-foreground">
            Agent not found
          </h1>
          <p className="mb-6 text-sm text-muted-foreground">
            The agent you are looking for does not exist or may have been removed.
          </p>

          <div className="flex items-center justify-center gap-3">
            <Link
              to="/studio/agents"
              className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition hover:opacity-90 no-underline"
            >
              <Users className="size-3.5" />
              View agents
            </Link>
            <Link
              to="/studio"
              className="inline-flex items-center gap-2 rounded-lg border border-border/60 bg-card px-4 py-2 text-sm font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
            >
              <LayoutDashboard className="size-3.5" />
              Dashboard
            </Link>
          </div>
        </div>
      </div>
    )
  }

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
          {errorMessage(error)}
        </p>

        <div className="flex items-center justify-center gap-3">
          <Button variant="default" size="default" onClick={() => reset()}>
            <RotateCcw className="size-3.5" data-icon="inline-start" />
            Try again
          </Button>
          <Link
            to="/studio"
            className="inline-flex items-center gap-2 rounded-lg border border-border/60 bg-card px-4 py-2 text-sm font-medium text-muted-foreground transition hover:border-border hover:text-foreground no-underline"
          >
            <LayoutDashboard className="size-3.5" />
            Dashboard
          </Link>
        </div>
      </div>
    </div>
  )
}
