import { useState, useCallback } from 'react'
import { Loader2, Upload, X, Server, BookOpen, ScrollText, AlertCircle, ArrowRight, Github } from 'lucide-react'
import type { ProjectLibrary } from '../features/compiler/types'

interface ImportDialogProps {
  open: boolean
  onClose: () => void
  onImport: (library: ProjectLibrary) => void
}

type ImportState =
  | { step: 'input' }
  | { step: 'loading' }
  | { step: 'preview'; library: ProjectLibrary; repoUrl: string }
  | { step: 'error'; message: string }

export function ImportDialog({ open, onClose, onImport }: ImportDialogProps) {
  const [url, setUrl] = useState('')
  const [state, setState] = useState<ImportState>({ step: 'input' })

  const reset = useCallback(() => {
    setUrl('')
    setState({ step: 'input' })
  }, [])

  const handleClose = useCallback(() => {
    reset()
    onClose()
  }, [onClose, reset])

  const handleFetch = useCallback(async () => {
    const trimmed = url.trim()
    if (!trimmed) return

    setState({ step: 'loading' })

    try {
      const res = await fetch('/api/github/import', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: trimmed }),
      })

      if (!res.ok) {
        const data = await res.json().catch(() => ({ error: 'Request failed' })) as { error?: string }
        setState({ step: 'error', message: data.error ?? `HTTP ${res.status}` })
        return
      }

      const library = await res.json() as ProjectLibrary
      setState({ step: 'preview', library, repoUrl: trimmed })
    } catch {
      setState({ step: 'error', message: 'Network error — check your connection' })
    }
  }, [url])

  const handleLoad = useCallback(() => {
    if (state.step !== 'preview') return
    onImport(state.library)
    handleClose()
  }, [state, onImport, handleClose])

  if (!open) return null

  return (
    <>
      {/* Backdrop */}
      <div className="fixed inset-0 z-40 bg-black/50 backdrop-blur-sm" onClick={handleClose} />

      {/* Dialog */}
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          className="w-full max-w-lg rounded-xl border border-border/60 bg-card shadow-2xl"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <Github className="size-4 text-muted-foreground" />
              <h2 className="font-display text-sm font-semibold">Import from GitHub</h2>
            </div>
            <button
              onClick={handleClose}
              className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition"
            >
              <X className="size-4" />
            </button>
          </div>

          {/* Body */}
          <div className="p-5">
            {/* URL input — always visible except in preview */}
            {state.step !== 'preview' && (
              <div>
                <label className="block text-xs font-medium text-muted-foreground mb-1.5">
                  Repository URL
                </label>
                <div className="flex gap-2">
                  <input
                    value={url}
                    onChange={(e) => {
                      setUrl(e.target.value)
                      if (state.step === 'error') setState({ step: 'input' })
                    }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && url.trim()) void handleFetch()
                    }}
                    placeholder="https://github.com/owner/repo"
                    className="flex-1 rounded-lg border border-border/60 bg-background/60 px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/30 transition"
                    disabled={state.step === 'loading'}
                    autoFocus
                  />
                  <button
                    onClick={() => void handleFetch()}
                    disabled={!url.trim() || state.step === 'loading'}
                    className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-40"
                  >
                    {state.step === 'loading' ? (
                      <Loader2 className="size-3.5 animate-spin" />
                    ) : (
                      <Upload className="size-3.5" />
                    )}
                    {state.step === 'loading' ? 'Scanning…' : 'Scan'}
                  </button>
                </div>

                {state.step === 'error' && (
                  <div className="mt-3 flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2.5">
                    <AlertCircle className="size-3.5 text-destructive shrink-0 mt-0.5" />
                    <p className="text-xs text-destructive">{state.message}</p>
                  </div>
                )}

                {state.step === 'loading' && (
                  <p className="mt-3 text-xs text-muted-foreground">
                    Fetching repository tree and extracting agent config…
                  </p>
                )}
              </div>
            )}

            {/* Preview */}
            {state.step === 'preview' && (
              <div>
                <div className="mb-4">
                  <p className="text-xs text-muted-foreground mb-1">Extracted from</p>
                  <p className="text-sm font-medium text-foreground truncate">{state.repoUrl}</p>
                </div>

                <PreviewSection
                  icon={<ScrollText className="size-3.5" />}
                  label="Rules"
                  items={state.library.rules.map((r) => r.file_name)}
                />
                <PreviewSection
                  icon={<BookOpen className="size-3.5" />}
                  label="Skills"
                  items={state.library.skills.map((s) => s.name)}
                />
                <PreviewSection
                  icon={<Server className="size-3.5" />}
                  label="MCP Servers"
                  items={state.library.mcp_servers.map((s) => s.name)}
                />

                {state.library.rules.length === 0 &&
                  state.library.skills.length === 0 &&
                  state.library.mcp_servers.length === 0 && (
                    <p className="text-xs text-muted-foreground py-4 text-center">
                      No agent configuration found in this repository.
                    </p>
                  )}
              </div>
            )}
          </div>

          {/* Footer */}
          {state.step === 'preview' && (
            <div className="flex items-center justify-between border-t border-border/60 px-5 py-3.5">
              <button
                onClick={reset}
                className="text-xs text-muted-foreground hover:text-foreground transition"
              >
                Try another repo
              </button>
              <button
                onClick={handleLoad}
                className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground transition hover:opacity-90"
              >
                Load into composer
                <ArrowRight className="size-3.5" />
              </button>
            </div>
          )}
        </div>
      </div>
    </>
  )
}

function PreviewSection({ icon, label, items }: { icon: React.ReactNode; label: string; items: string[] }) {
  if (items.length === 0) return null

  return (
    <div className="mb-3 last:mb-0">
      <div className="flex items-center gap-1.5 mb-1.5">
        <span className="text-muted-foreground">{icon}</span>
        <span className="text-xs font-semibold text-foreground">
          {label}
          <span className="ml-1.5 rounded-full bg-primary/10 px-1.5 py-0.5 text-[10px] font-bold text-primary">
            {items.length}
          </span>
        </span>
      </div>
      <div className="flex flex-wrap gap-1.5 pl-5">
        {items.map((name) => (
          <span
            key={name}
            className="rounded-md border border-border/60 bg-muted/40 px-2 py-1 text-[11px] font-medium text-foreground"
          >
            {name}
          </span>
        ))}
      </div>
    </div>
  )
}
