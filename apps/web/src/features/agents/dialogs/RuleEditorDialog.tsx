import { useState, useEffect, useCallback, useRef } from 'react'
import { FileText, X, Trash2 } from 'lucide-react'
import type { Rule } from '@ship/ui'

type RuleData = Pick<Rule, 'file_name' | 'content' | 'always_apply' | 'globs'>

interface RuleEditorDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  rule: RuleData | null
  onSave: (rule: RuleData) => void
  onDelete?: () => void
}

export function RuleEditorDialog({ open, onOpenChange, rule, onSave, onDelete }: RuleEditorDialogProps) {
  const [fileName, setFileName] = useState('')
  const [content, setContent] = useState('')
  const [alwaysApply, setAlwaysApply] = useState(true)
  const [globs, setGlobs] = useState('')
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (open) {
      setFileName(rule?.file_name ?? '')
      setContent(rule?.content ?? '')
      setAlwaysApply(rule?.always_apply ?? true)
      setGlobs(rule?.globs?.join(', ') ?? '')
      setTimeout(() => inputRef.current?.focus(), 0)
    }
  }, [open, rule])

  const close = useCallback(() => onOpenChange(false), [onOpenChange])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') close() }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [open, close])

  if (!open) return null

  const isEditing = rule !== null
  const trimmed = fileName.trim()
  const valid = trimmed.length > 0 && trimmed.endsWith('.md') && content.trim().length > 0

  const handleSave = () => {
    if (!valid) return
    const parsedGlobs = globs.split(',').map((g) => g.trim()).filter(Boolean)
    onSave({
      file_name: trimmed,
      content: content.trim(),
      always_apply: alwaysApply,
      ...(parsedGlobs.length > 0 && !alwaysApply ? { globs: parsedGlobs } : {}),
    })
    close()
  }

  const inputCls = 'w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground/50 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/25 transition'

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={close} />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          className="w-full max-w-lg rounded-xl border border-border/60 bg-card shadow-2xl"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <FileText className="size-4 text-primary" />
              <h2 className="font-display text-sm font-semibold text-foreground">
                {isEditing ? 'Edit Rule' : 'Create Rule'}
              </h2>
            </div>
            <button onClick={close} aria-label="Close" className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition">
              <X className="size-4" />
            </button>
          </div>
          {/* Body */}
          <div className="space-y-4 px-5 py-4">
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Filename</label>
              <input ref={inputRef} type="text" value={fileName} onChange={(e) => setFileName(e.target.value)} placeholder="my-rule.md" className={inputCls} />
            </div>
            <div className="space-y-1.5">
              <label className="text-xs font-medium text-muted-foreground">Content</label>
              <textarea value={content} onChange={(e) => setContent(e.target.value)} rows={6} placeholder="Write rule content in markdown..." className={`${inputCls} font-mono resize-y`} />
            </div>
            <div className="flex items-center gap-2">
              <label className="flex items-center gap-2 cursor-pointer select-none">
                <input
                  type="checkbox"
                  checked={alwaysApply}
                  onChange={(e) => setAlwaysApply(e.target.checked)}
                  className="size-3.5 rounded border-border/60 accent-primary"
                />
                <span className="text-xs font-medium text-muted-foreground">Always apply this rule</span>
              </label>
            </div>
            {!alwaysApply && (
              <div className="space-y-1.5">
                <label className="text-xs font-medium text-muted-foreground">File patterns (comma-separated)</label>
                <input type="text" value={globs} onChange={(e) => setGlobs(e.target.value)} placeholder="e.g. src/**/*.ts, tests/**" className={inputCls} />
                <p className="text-[11px] text-muted-foreground/70">When always_apply is off, this rule only applies to files matching these patterns</p>
              </div>
            )}
          </div>
          {/* Footer */}
          <div className="flex items-center justify-between border-t border-border/60 px-5 py-3.5">
            <div>
              {isEditing && onDelete && (
                <button onClick={() => { onDelete(); close() }} className="flex items-center gap-1.5 rounded-lg px-3 py-2 text-xs font-medium text-destructive hover:bg-destructive/10 transition">
                  <Trash2 className="size-3.5" />Delete
                </button>
              )}
            </div>
            <div className="flex items-center gap-2">
              <button onClick={close} className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground">
                Cancel
              </button>
              <button onClick={handleSave} disabled={!valid} className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed">
                Save
              </button>
            </div>
          </div>
        </div>
      </div>
    </>
  )
}
