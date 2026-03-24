import { createFileRoute, Link } from '@tanstack/react-router'
import { ArrowLeft, Terminal } from 'lucide-react'

export const Route = createFileRoute('/studio/import')({ component: ImportPage, ssr: false })

/**
 * GitHub import is disabled for v0.1.0 (CLI-first, no accounts).
 * The full implementation is preserved in git history.
 */
function ImportPage() {
  return (
    <div className="mx-auto max-w-[640px] px-5 py-6 pb-24">
      <div className="mb-2">
        <Link to="/studio" className="inline-flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition mb-3">
          <ArrowLeft className="size-3" /> Back to Studio
        </Link>
      </div>

      <div className="flex flex-col items-center gap-4 py-16 text-center">
        <div className="flex size-14 items-center justify-center rounded-2xl border border-border/60 bg-muted/40">
          <Terminal className="size-6 text-muted-foreground" />
        </div>
        <div>
          <p className="text-sm font-semibold text-foreground">
            Import via CLI
          </p>
          <p className="mt-1 text-xs text-muted-foreground max-w-xs">
            Use <code className="text-[11px] bg-muted px-1 py-0.5 rounded font-mono">ship init</code> in your
            project directory to create a .ship/ config, then connect Studio via the CLI sync button in the dock.
          </p>
        </div>
      </div>
    </div>
  )
}
