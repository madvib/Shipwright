import { useState } from 'react'
import { X, Github, FolderOpen, Link } from 'lucide-react'

const DISMISSED_KEY = 'ship-first-run-banner-dismissed'

function isDismissed() {
  try { return localStorage.getItem(DISMISSED_KEY) === '1' } catch { return false }
}

type ImportTab = 'github-url' | 'github-app' | 'local'

const TABS: Array<{ id: ImportTab; label: string; icon: React.ElementType }> = [
  { id: 'github-url', label: 'GitHub URL', icon: Link },
  { id: 'github-app', label: 'GitHub App', icon: Github },
  { id: 'local', label: 'Local folder', icon: FolderOpen },
]

const BLANK_PRESETS = [
  'Web Developer (React · TypeScript · Tailwind)',
  'Rust Engineer (CLI · Systems · Cargo)',
  'Full Stack (API · DB · Frontend)',
  'Ship Commander (Multi-agent · Orchestration)',
  'Blank Profile',
]

export function FirstRunBanner({ onDismiss }: { onDismiss: () => void }) {
  const [tab, setTab] = useState<ImportTab>('github-url')
  const [url, setUrl] = useState('')

  return (
    <div className="relative rounded-xl border border-border/60 bg-card p-5 mb-6">
      <button
        onClick={onDismiss}
        className="absolute right-3 top-3 rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition"
        title="Dismiss"
      >
        <X className="size-3.5" />
      </button>

      <div className="mb-4">
        <h2 className="font-display text-sm font-semibold text-foreground">Import an existing project</h2>
        <p className="mt-0.5 text-xs text-muted-foreground">
          Ship reads your repo — CLAUDE.md, .mcp.json, GEMINI.md, AGENTS.md, .cursor/, .codex/ — and consolidates everything into <code className="text-[10px] bg-muted px-1 py-0.5 rounded">.ship/</code>
        </p>
      </div>

      {/* Tab picker */}
      <div className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5 w-fit mb-4">
        {TABS.map(({ id, label, icon: Icon }) => (
          <button
            key={id}
            onClick={() => setTab(id)}
            className={`flex items-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium transition ${
              tab === id ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Icon className="size-3" />
            {label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      <div className="flex items-center gap-2">
        {tab === 'github-url' && (
          <>
            <input
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder="https://github.com/org/repo"
              className="flex-1 rounded-lg border border-border/60 bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground focus:border-border focus:outline-none transition"
            />
            <button
              disabled={!url.trim()}
              className="shrink-0 rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-40"
            >
              Import
            </button>
          </>
        )}
        {tab === 'github-app' && (
          <button className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-3 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground">
            <Github className="size-3" />
            Connect GitHub App
          </button>
        )}
        {tab === 'local' && (
          <p className="text-xs text-muted-foreground">
            Run <code className="bg-muted px-1.5 py-0.5 rounded text-[10px] font-mono">ship import</code> in your project directory, then refresh.
          </p>
        )}
      </div>

      {/* Start blank divider */}
      <div className="mt-4 pt-4 border-t border-border/40 flex flex-wrap items-center gap-2">
        <span className="text-[11px] text-muted-foreground">or start blank:</span>
        {BLANK_PRESETS.map((p) => (
          <button
            key={p}
            className="rounded-full border border-border/60 bg-muted/30 px-2.5 py-1 text-[10px] text-muted-foreground hover:bg-muted hover:text-foreground transition"
          >
            {p}
          </button>
        ))}
      </div>
    </div>
  )
}

export function useFirstRunBanner() {
  const [dismissed, setDismissed] = useState(isDismissed)

  const dismiss = () => {
    try { localStorage.setItem(DISMISSED_KEY, '1') } catch { /* ignore */ }
    setDismissed(true)
  }

  return { show: !dismissed, dismiss }
}
