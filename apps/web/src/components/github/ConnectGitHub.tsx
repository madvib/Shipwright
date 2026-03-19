import { Github } from 'lucide-react'
import { useState } from 'react'

interface ConnectGitHubProps {
  /** Whether to show as a compact inline card (default) or a prominent prompt */
  variant?: 'card' | 'inline'
}

/**
 * Renders a prompt to connect GitHub.
 * Clicking "Connect GitHub" navigates to /api/github/oauth which redirects to GitHub.
 */
export function ConnectGitHub({ variant = 'card' }: ConnectGitHubProps) {
  const [loading, setLoading] = useState(false)

  const handleConnect = () => {
    setLoading(true)
    // The OAuth endpoint redirects; let the browser follow it
    window.location.href = '/api/github/oauth'
  }

  if (variant === 'inline') {
    return (
      <div className="flex items-center justify-between rounded-lg border border-border/40 bg-muted/20 px-3 py-2.5">
        <div className="flex items-center gap-2">
          <Github className="size-3.5 text-muted-foreground shrink-0" />
          <p className="text-xs text-muted-foreground">Connect GitHub to push your config directly to a repo</p>
        </div>
        <button
          onClick={handleConnect}
          disabled={loading}
          className="ml-3 inline-flex items-center gap-1.5 rounded-lg bg-foreground px-3 py-1.5 text-[11px] font-semibold text-background transition hover:opacity-80 disabled:opacity-50 shrink-0"
        >
          <Github className="size-3" />
          {loading ? 'Connecting…' : 'Connect GitHub'}
        </button>
      </div>
    )
  }

  return (
    <div className="rounded-xl border border-border/60 bg-card p-4">
      <div className="flex items-start gap-3">
        <div className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-foreground/5 mt-0.5">
          <Github className="size-4 text-foreground" />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-semibold text-foreground mb-0.5">Connect GitHub</p>
          <p className="text-[11px] text-muted-foreground mb-3">
            Push your Ship config directly to a GitHub repo and open a ready-to-merge PR.
          </p>
          <button
            onClick={handleConnect}
            disabled={loading}
            className="inline-flex items-center gap-1.5 rounded-lg bg-foreground px-3.5 py-2 text-xs font-semibold text-background transition hover:opacity-80 disabled:opacity-50"
          >
            <Github className="size-3.5" />
            {loading ? 'Connecting…' : 'Connect GitHub'}
          </button>
        </div>
      </div>
    </div>
  )
}
