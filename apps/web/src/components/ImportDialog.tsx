import { useState, useCallback } from 'react'
import { Loader2, Upload, X, Server, BookOpen, ScrollText, AlertCircle, ArrowRight, Github } from 'lucide-react'
import { toast } from 'sonner'
import { PreviewSection } from './ImportPreview'
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

interface Selection {
  skills: Set<string>
  rules: Set<string>
  mcp_servers: Set<string>
}

/** Filter a library to only include items present in the selection sets. */
export function filterLibraryBySelection(library: ProjectLibrary, selection: Selection): ProjectLibrary {
  return {
    ...library,
    skills: (library.skills ?? []).filter((s) => selection.skills.has(s.id)),
    rules: (library.rules ?? []).filter((r) => selection.rules.has(r.file_name)),
    mcp_servers: (library.mcp_servers ?? []).filter((s) => selection.mcp_servers.has(s.name)),
  }
}

function selectionFromLibrary(library: ProjectLibrary): Selection {
  return {
    skills: new Set((library.skills ?? []).map((s) => s.id)),
    rules: new Set((library.rules ?? []).map((r) => r.file_name)),
    mcp_servers: new Set((library.mcp_servers ?? []).map((s) => s.name)),
  }
}

export function ImportDialog({ open, onClose, onImport }: ImportDialogProps) {
  const [url, setUrl] = useState('')
  const [state, setState] = useState<ImportState>({ step: 'input' })
  const [selection, setSelection] = useState<Selection>({ skills: new Set(), rules: new Set(), mcp_servers: new Set() })

  const reset = useCallback(() => {
    setUrl('')
    setState({ step: 'input' })
    setSelection({ skills: new Set(), rules: new Set(), mcp_servers: new Set() })
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
        const msg = data.error ?? `HTTP ${res.status}`
        setState({ step: 'error', message: msg })
        toast.error(`Import failed: ${msg}`)
        return
      }

      const library = await res.json() as ProjectLibrary
      setState({ step: 'preview', library, repoUrl: trimmed })
      setSelection(selectionFromLibrary(library))
    } catch {
      setState({ step: 'error', message: 'Network error — check your connection' })
      toast.error('Network error — check your connection')
    }
  }, [url])

  const handleLoad = useCallback(() => {
    if (state.step !== 'preview') return
    const filtered = filterLibraryBySelection(state.library, selection)
    const hasAny = (filtered.skills ?? []).length > 0 || (filtered.rules ?? []).length > 0 || (filtered.mcp_servers ?? []).length > 0
    if (!hasAny) return
    onImport(filtered)
    const total = (filtered.skills ?? []).length + (filtered.rules ?? []).length + (filtered.mcp_servers ?? []).length
    toast.success(`Imported ${total} item${total !== 1 ? 's' : ''} from GitHub`)
    handleClose()
  }, [state, selection, onImport, handleClose])

  const toggleItem = useCallback((category: keyof Selection, key: string) => {
    setSelection((prev) => {
      const next = new Set(prev[category])
      if (next.has(key)) next.delete(key)
      else next.add(key)
      return { ...prev, [category]: next }
    })
  }, [])

  const toggleAll = useCallback((category: keyof Selection, keys: string[]) => {
    setSelection((prev) => {
      const allSelected = keys.every((k) => prev[category].has(k))
      const next = allSelected ? new Set<string>() : new Set(keys)
      return { ...prev, [category]: next }
    })
  }, [])

  const selectedCount =
    selection.skills.size + selection.rules.size + selection.mcp_servers.size

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
              aria-label="Close import dialog"
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
                  <p className="mt-3 text-xs text-muted-foreground" aria-busy="true" aria-label="Scanning repository">
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
                  items={(state.library.rules ?? []).map((r) => r.file_name)}
                  selected={selection.rules}
                  onToggle={(key) => toggleItem('rules', key)}
                  onToggleAll={(keys) => toggleAll('rules', keys)}
                />
                <PreviewSection
                  icon={<BookOpen className="size-3.5" />}
                  label="Skills"
                  items={(state.library.skills ?? []).map((s) => s.id)}
                  labels={(state.library.skills ?? []).map((s) => s.name)}
                  selected={selection.skills}
                  onToggle={(key) => toggleItem('skills', key)}
                  onToggleAll={(keys) => toggleAll('skills', keys)}
                />
                <PreviewSection
                  icon={<Server className="size-3.5" />}
                  label="MCP Servers"
                  items={(state.library.mcp_servers ?? []).map((s) => s.name)}
                  selected={selection.mcp_servers}
                  onToggle={(key) => toggleItem('mcp_servers', key)}
                  onToggleAll={(keys) => toggleAll('mcp_servers', keys)}
                />

                {(state.library.rules ?? []).length === 0 &&
                  (state.library.skills ?? []).length === 0 &&
                  (state.library.mcp_servers ?? []).length === 0 && (
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
                disabled={selectedCount === 0}
                className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-sm font-semibold text-primary-foreground transition hover:opacity-90 disabled:opacity-40"
              >
                Load into composer
                {selectedCount > 0 && (
                  <span className="rounded-full bg-white/20 px-1.5 py-0.5 text-[10px] font-bold">
                    {selectedCount}
                  </span>
                )}
                <ArrowRight className="size-3.5" />
              </button>
            </div>
          )}
        </div>
      </div>
    </>
  )
}

