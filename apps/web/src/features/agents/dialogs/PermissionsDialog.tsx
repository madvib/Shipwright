import { useState, useEffect, useCallback, useRef } from 'react'
import { Lock, Wrench, X } from 'lucide-react'
import type { ProfilePermissions } from '@ship/ui'

interface PermissionsDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  permissions: ProfilePermissions
  onSave: (permissions: ProfilePermissions) => void
}

export function PermissionsDialog({ open, onOpenChange, permissions, onSave }: PermissionsDialogProps) {
  const [local, setLocal] = useState<ProfilePermissions>({})

  useEffect(() => {
    if (open) setLocal(structuredClone(permissions))
  }, [open, permissions])

  const handleEscape = useCallback(
    (e: KeyboardEvent) => { if (e.key === 'Escape') onOpenChange(false) },
    [onOpenChange],
  )

  useEffect(() => {
    if (!open) return
    document.addEventListener('keydown', handleEscape)
    return () => document.removeEventListener('keydown', handleEscape)
  }, [open, handleEscape])

  if (!open) return null

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={() => onOpenChange(false)} />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          className="w-full max-w-lg rounded-xl border border-border/60 bg-card shadow-2xl flex flex-col max-h-[80vh]"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <Lock className="size-4 text-primary" />
              <h2 className="font-display text-sm font-semibold text-foreground">Edit Permissions</h2>
            </div>
            <button
              onClick={() => onOpenChange(false)}
              aria-label="Close"
              className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition"
            >
              <X className="size-4" />
            </button>
          </div>

          {/* Scrollable body */}
          <div className="flex-1 overflow-y-auto px-5 py-4 space-y-5">
            <DimensionSection icon={<Wrench className="size-3.5" />} label="Tools">
              <TagInput
                label="Allow"
                variant="allow"
                values={local.tools_allow ?? []}
                onChange={(v) => setLocal((prev) => ({ ...prev, tools_allow: v }))}
              />
              <TagInput
                label="Ask"
                variant="neutral"
                values={local.tools_ask ?? []}
                onChange={(v) => setLocal((prev) => ({ ...prev, tools_ask: v }))}
              />
              <TagInput
                label="Deny"
                variant="deny"
                values={local.tools_deny ?? []}
                onChange={(v) => setLocal((prev) => ({ ...prev, tools_deny: v }))}
              />
            </DimensionSection>
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-2 border-t border-border/60 px-5 py-3.5">
            <button
              onClick={() => onOpenChange(false)}
              className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              Cancel
            </button>
            <button
              onClick={() => { onSave(local); onOpenChange(false) }}
              className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90"
            >
              Save
            </button>
          </div>
        </div>
      </div>
    </>
  )
}

function DimensionSection({ icon, label, children }: { icon: React.ReactNode; label: string; children: React.ReactNode }) {
  return (
    <div className="space-y-2.5">
      <div className="flex items-center gap-1.5 text-xs font-medium text-foreground">
        {icon}
        {label}
      </div>
      <div className="space-y-2 pl-5">{children}</div>
    </div>
  )
}

function TagInput({
  label,
  variant,
  values,
  onChange,
}: {
  label: string
  variant: 'allow' | 'deny' | 'neutral'
  values: string[]
  onChange: (values: string[]) => void
}) {
  const inputRef = useRef<HTMLInputElement>(null)

  const tagColor =
    variant === 'allow' ? 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20'
    : variant === 'deny' ? 'bg-destructive/10 text-destructive border-destructive/20'
    : 'bg-muted text-muted-foreground border-border/40'

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key !== 'Enter') return
    e.preventDefault()
    const v = (e.currentTarget.value ?? '').trim()
    if (v && !values.includes(v)) {
      onChange([...values, v])
    }
    e.currentTarget.value = ''
  }

  const remove = (idx: number) => onChange(values.filter((_, i) => i !== idx))

  return (
    <div className="space-y-1.5">
      <span className="text-[11px] text-muted-foreground">{label}</span>
      {values.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {values.map((v, i) => (
            <span key={`${v}-${i}`} className={`inline-flex items-center gap-1 rounded-md border px-1.5 py-0.5 text-[11px] ${tagColor}`}>
              {v}
              <button type="button" onClick={() => remove(i)} className="opacity-60 hover:opacity-100 transition-opacity">
                <X className="size-2.5" />
              </button>
            </span>
          ))}
        </div>
      )}
      <input
        ref={inputRef}
        type="text"
        placeholder={`Add ${label.toLowerCase()}...`}
        onKeyDown={handleKeyDown}
        className="w-full rounded-md border border-border/60 bg-background px-2.5 py-1.5 text-xs text-foreground placeholder:text-muted-foreground/40 outline-none focus:border-primary"
      />
    </div>
  )
}
