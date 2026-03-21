import { useState, useEffect, useCallback } from 'react'
import { Link2, X, Trash2 } from 'lucide-react'
import type { HookConfig } from '../types'
import { PROVIDERS } from '#/features/compiler/types'

const TRIGGERS = [
  'PreToolUse', 'PostToolUse', 'Stop', 'Notification', 'SubagentStop', 'PreCompact',
] as const

interface HookEditorDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  hook: HookConfig | null
  onSave: (hook: HookConfig) => void
  onDelete?: () => void
}

export function HookEditorDialog({ open, onOpenChange, hook, onSave, onDelete }: HookEditorDialogProps) {
  const [trigger, setTrigger] = useState('')
  const [command, setCommand] = useState('')
  const [providers, setProviders] = useState<string[]>([])
  const [matcher, setMatcher] = useState('')

  const isEditing = hook !== null

  useEffect(() => {
    if (open) {
      setTrigger(hook?.trigger ?? '')
      setCommand(hook?.command ?? '')
      setProviders(hook?.providers ?? [])
      setMatcher(hook?.matcher ?? '')
    }
  }, [open, hook])

  const close = useCallback(() => onOpenChange(false), [onOpenChange])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') close() }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [open, close])

  if (!open) return null

  const valid = trigger !== '' && command.trim() !== '' && providers.length > 0

  const toggleProvider = (id: string) => {
    setProviders((prev) => prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id])
  }

  const handleSave = () => {
    if (!valid) return
    const trimmedMatcher = matcher.trim()
    onSave({ trigger, command: command.trim(), providers, ...(trimmedMatcher ? { matcher: trimmedMatcher } : {}) })
    close()
  }

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={close} />

      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          className="w-full max-w-md rounded-xl border border-border/60 bg-card shadow-2xl"
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <Link2 className="size-4 text-primary" />
              <h2 className="font-display text-sm font-semibold text-foreground">
                {isEditing ? 'Edit Hook' : 'Add Hook'}
              </h2>
            </div>
            <button onClick={close} aria-label="Close" className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition">
              <X className="size-4" />
            </button>
          </div>
          <div className="space-y-4 px-5 py-4">
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Trigger</label>
              <select
                value={trigger}
                onChange={(e) => setTrigger(e.target.value)}
                className="w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm text-foreground outline-none focus:border-primary transition"
              >
                <option value="">Select trigger...</option>
                {TRIGGERS.map((t) => <option key={t} value={t}>{t}</option>)}
              </select>
            </div>
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Command</label>
              <input
                type="text"
                value={command}
                onChange={(e) => setCommand(e.target.value)}
                placeholder="./scripts/my-hook.sh"
                className="w-full rounded-lg border border-border/60 bg-background px-3 py-2 font-mono text-sm text-foreground outline-none focus:border-primary transition placeholder:text-muted-foreground/50"
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Matcher pattern (optional)</label>
              <input
                type="text"
                value={matcher}
                onChange={(e) => setMatcher(e.target.value)}
                placeholder="e.g. Edit|Write — regex to match tool names"
                className="w-full rounded-lg border border-border/60 bg-background px-3 py-2 font-mono text-sm text-foreground outline-none focus:border-primary transition placeholder:text-muted-foreground/50"
              />
              <p className="text-[11px] text-muted-foreground/70">Only trigger this hook when the tool name matches this regex</p>
            </div>
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Providers</label>
              <div className="flex flex-wrap gap-2">
                {PROVIDERS.map((p) => (
                  <label key={p.id} className="flex items-center gap-1.5 cursor-pointer select-none">
                    <input
                      type="checkbox"
                      checked={providers.includes(p.id)}
                      onChange={() => toggleProvider(p.id)}
                      className="size-3.5 rounded border-border/60 accent-primary"
                    />
                    <span className="text-xs text-foreground">{p.name}</span>
                  </label>
                ))}
              </div>
            </div>
          </div>
          <div className="flex items-center justify-between border-t border-border/60 px-5 py-3.5">
            <div>
              {isEditing && onDelete && (
                <button
                  onClick={() => { onDelete(); close() }}
                  className="flex items-center gap-1.5 rounded-lg px-3 py-2 text-xs font-medium text-destructive hover:bg-destructive/10 transition"
                >
                  <Trash2 className="size-3.5" />
                  Delete
                </button>
              )}
            </div>
            <div className="flex items-center gap-2">
              <button onClick={close} className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground">
                Cancel
              </button>
              <button
                onClick={handleSave}
                disabled={!valid}
                className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {isEditing ? 'Save' : 'Add'}
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
