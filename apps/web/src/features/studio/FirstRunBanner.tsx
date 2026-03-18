import { useState } from 'react'
import { X, Github, Link as LinkIcon, Loader2 } from 'lucide-react'
import { authClient } from '#/lib/auth-client'

const DISMISSED_KEY = 'ship-first-run-banner-dismissed'

function isDismissed() {
  try { return localStorage.getItem(DISMISSED_KEY) === '1' } catch { return false }
}

type ImportTab = 'github-url' | 'github-app'

const TABS: Array<{ id: ImportTab; label: string; icon: React.ElementType }> = [
  { id: 'github-url', label: 'GitHub URL', icon: LinkIcon },
  { id: 'github-app', label: 'GitHub App', icon: Github },
]

export interface PresetTemplate {
  id: string
  label: string
  persona: string
  providers: string[]
  rules: string[]
}

const BLANK_PRESETS: PresetTemplate[] = [
  {
    id: 'web-dev',
    label: 'Web Developer (React \u00b7 TypeScript \u00b7 Tailwind)',
    persona: 'React + TypeScript + Tailwind frontend specialist. Prefer composition, strict types, components under 200 lines.',
    providers: ['claude', 'cursor'],
    rules: ['Use functional components with hooks', 'Keep files under 300 lines', 'Prefer Tailwind utility classes'],
  },
  {
    id: 'rust-eng',
    label: 'Rust Engineer (CLI \u00b7 Systems \u00b7 Cargo)',
    persona: 'Rust systems engineer. CLI tooling, error handling with thiserror/anyhow, workspace-aware Cargo builds.',
    providers: ['claude', 'codex'],
    rules: ['Use thiserror for library errors, anyhow for binaries', 'Prefer zero-copy where practical', 'Run clippy before committing'],
  },
  {
    id: 'fullstack',
    label: 'Full Stack (API \u00b7 DB \u00b7 Frontend)',
    persona: 'Full-stack developer. API design, database migrations, frontend integration. TypeScript throughout.',
    providers: ['claude', 'gemini', 'cursor'],
    rules: ['Keep API routes thin, logic in services', 'Use explicit error types', 'Test happy and failure paths'],
  },
  {
    id: 'commander',
    label: 'Ship Commander (Multi-agent \u00b7 Orchestration)',
    persona: 'Ship Commander orchestrating multi-agent workflows. Route jobs, manage worktrees, review before merge.',
    providers: ['claude'],
    rules: ['Route jobs to the right workspace', 'Write handoff.md at session end', 'Never do work outside your file scope'],
  },
  {
    id: 'blank',
    label: 'Blank Profile',
    persona: '',
    providers: ['claude'],
    rules: [],
  },
]

interface FirstRunBannerProps {
  onDismiss: () => void
  onPresetInit: (preset: PresetTemplate) => void
  onImportUrl: (url: string) => Promise<void>
}

export function FirstRunBanner({ onDismiss, onPresetInit, onImportUrl }: FirstRunBannerProps) {
  const [tab, setTab] = useState<ImportTab>('github-url')
  const [url, setUrl] = useState('')
  const [importing, setImporting] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [initingPreset, setInitingPreset] = useState<string | null>(null)

  const handleImport = async () => {
    const trimmed = url.trim()
    if (!trimmed) return
    setImporting(true)
    setError(null)
    try {
      await onImportUrl(trimmed)
      setUrl('')
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Import failed')
    } finally {
      setImporting(false)
    }
  }

  const handleGitHubConnect = () => {
    void authClient.signIn.social({
      provider: 'github',
      callbackURL: window.location.href,
    })
  }

  const handlePresetInit = (preset: PresetTemplate) => {
    setInitingPreset(preset.id)
    try {
      onPresetInit(preset)
    } finally {
      setInitingPreset(null)
    }
  }

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
      <div className="flex flex-col gap-2">
        {tab === 'github-url' && (
          <>
            <div className="flex items-center gap-2">
              <input
                value={url}
                onChange={(e) => { setUrl(e.target.value); setError(null) }}
                onKeyDown={(e) => { if (e.key === 'Enter' && url.trim()) void handleImport() }}
                placeholder="https://github.com/org/repo"
                disabled={importing}
                className="flex-1 rounded-lg border border-border/60 bg-background px-3 py-2 text-xs text-foreground placeholder:text-muted-foreground focus:border-border focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 transition disabled:opacity-50"
              />
              <button
                onClick={() => void handleImport()}
                disabled={!url.trim() || importing}
                className="shrink-0 rounded-lg bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-40 inline-flex items-center gap-1.5"
              >
                {importing && <Loader2 className="size-3 animate-spin" />}
                {importing ? 'Importing\u2026' : 'Import'}
              </button>
            </div>
            {error && (
              <p className="text-[11px] text-destructive">{error}</p>
            )}
          </>
        )}
        {tab === 'github-app' && (
          <button
            onClick={handleGitHubConnect}
            className="inline-flex items-center gap-1.5 rounded-lg border border-border/60 bg-card px-3 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground w-fit"
          >
            <Github className="size-3" />
            Connect GitHub App
          </button>
        )}
      </div>

      {/* Start blank divider */}
      <div className="mt-4 pt-4 border-t border-border/40 flex flex-wrap items-center gap-2">
        <span className="text-[11px] text-muted-foreground">or start blank:</span>
        {BLANK_PRESETS.map((p) => (
          <button
            key={p.id}
            onClick={() => handlePresetInit(p)}
            disabled={initingPreset === p.id}
            className="rounded-full border border-border/60 bg-muted/30 px-2.5 py-1 text-[10px] text-muted-foreground hover:bg-muted hover:text-foreground transition disabled:opacity-50 inline-flex items-center gap-1"
          >
            {initingPreset === p.id && <Loader2 className="size-2.5 animate-spin" />}
            {p.label}
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
