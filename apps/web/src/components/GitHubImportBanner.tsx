import { useState } from 'react'
import { Github } from 'lucide-react'
import { Button, Input } from '@ship/primitives'
import { authClient } from '#/lib/auth-client'

interface GitHubImportBannerProps {
  title?: string
  subtitle?: string
  description?: string
  showConnectButton?: boolean
  showPasteInput?: boolean
  onImport?: (repoUrl: string) => void
  className?: string
}

export function GitHubImportBanner({
  title = 'Import from GitHub',
  subtitle = 'Already using CLAUDE.md or .cursor/rules? We\'ll convert it.',
  description = 'Paste a repo URL to extract agent configs, or connect the GitHub App and we\'ll create a PR that adds Ship to your project.',
  showConnectButton = true,
  showPasteInput = true,
  onImport,
  className = '',
}: GitHubImportBannerProps) {
  const [repoUrl, setRepoUrl] = useState('')

  const handleConnectGitHub = () => {
    void authClient.signIn.social({ provider: 'github' })
  }

  const handleImport = () => {
    if (repoUrl.trim() && onImport) {
      onImport(repoUrl.trim())
    }
  }

  return (
    <div
      className={`rounded-xl border border-border/60 bg-card p-5 text-left ${className}`}
      style={{
        background:
          'linear-gradient(135deg, color-mix(in oklch, var(--card) 90%, var(--primary) 10%), var(--card))',
      }}
    >
      <div className="mb-2.5 flex items-center gap-3">
        <div className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-muted/60">
          <Github className="size-5 text-foreground" />
        </div>
        <div>
          <div className="text-sm font-semibold text-foreground">{title}</div>
          <div className="text-[11px] text-muted-foreground">{subtitle}</div>
        </div>
      </div>

      {description && (
        <p className="mb-3.5 text-xs leading-relaxed text-muted-foreground">
          {description}
        </p>
      )}

      <div className="flex items-center gap-2">
        {showConnectButton && (
          <Button
            variant="outline"
            size="sm"
            onClick={handleConnectGitHub}
            className="shrink-0"
          >
            <Github className="size-3.5" />
            Connect GitHub
          </Button>
        )}

        {showPasteInput && (
          <div className="flex flex-1 overflow-hidden rounded-md border border-border/60">
            <Input
              value={repoUrl}
              onChange={(e) => setRepoUrl(e.target.value)}
              placeholder="Paste repo URL..."
              className="flex-1 rounded-none border-0 shadow-none text-[11px] h-8"
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleImport()
              }}
            />
            <button
              onClick={handleImport}
              disabled={!repoUrl.trim()}
              className="shrink-0 border-l border-border/60 bg-muted/40 px-3 text-[11px] font-medium text-muted-foreground transition hover:bg-muted hover:text-foreground disabled:opacity-40"
            >
              Import
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
