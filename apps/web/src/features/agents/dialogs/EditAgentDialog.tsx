import { useState, useEffect, useCallback, useRef, useMemo } from 'react'
import { Pencil, X } from 'lucide-react'
import type { ResolvedAgentProfile } from '../types'
import { getFieldEnum } from '#/features/agents/schema-hints'
import { validateAgentProfile } from '#/features/agents/schema-validation'

interface EditAgentDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  profile: ResolvedAgentProfile
  onSave: (patch: Partial<ResolvedAgentProfile>) => void
}

const inputCls = 'flex w-full rounded-lg border border-border/60 bg-transparent px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none transition'

export function EditAgentDialog({ open, onOpenChange, profile, onSave }: EditAgentDialogProps) {
  const [name, setName] = useState(profile.profile.name)
  const [description, setDescription] = useState(profile.profile.description ?? '')
  const [selectedProviders, setSelectedProviders] = useState<string[]>(profile.profile.providers ?? [])
  const [validationErrors, setValidationErrors] = useState<string[]>([])
  const nameRef = useRef<HTMLInputElement>(null)
  const schemaProviders = useMemo(() => getFieldEnum('agent.providers'), [])

  useEffect(() => {
    if (!open) return
    setName(profile.profile.name)
    setDescription(profile.profile.description ?? '')
    setSelectedProviders([...(profile.profile.providers ?? [])])
    requestAnimationFrame(() => nameRef.current?.focus())
  }, [open, profile])

  const close = useCallback(() => onOpenChange(false), [onOpenChange])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') close() }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [open, close])

  const toggleProvider = (id: string) => {
    setSelectedProviders((prev) =>
      prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id],
    )
  }

  const canSave = name.trim().length > 0 && selectedProviders.length > 0
  const handleSave = () => {
    if (!canSave) return
    // Validate providers against schema
    const draft: ResolvedAgentProfile = {
      ...profile,
      profile: {
        ...profile.profile,
        name: name.trim(),
        description: description.trim(),
        providers: selectedProviders,
      },
    }
    const result = validateAgentProfile(draft)
    if (!result.valid) {
      setValidationErrors(result.errors.map((e) => e.message))
      return
    }
    setValidationErrors([])
    onSave({ profile: { ...profile.profile, name: name.trim(), description: description.trim(), providers: selectedProviders } })
    close()
  }

  if (!open) return null

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={close} />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div role="dialog" aria-modal="true" className="w-full max-w-sm rounded-xl border border-border/60 bg-card shadow-2xl" onClick={(e) => e.stopPropagation()}>
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <Pencil className="size-4 text-primary" />
              <h2 className="font-display text-sm font-semibold text-foreground">Edit agent</h2>
            </div>
            <button onClick={close} aria-label="Close" className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition">
              <X className="size-4" />
            </button>
          </div>

          <div className="space-y-4 px-5 py-4">
            <div>
              <label className="text-xs font-medium text-foreground mb-1.5 block">Name</label>
              <input ref={nameRef} value={name} onChange={(e) => setName(e.target.value)} className={inputCls} />
            </div>
            <div>
              <label className="text-xs font-medium text-foreground mb-1.5 block">Description</label>
              <textarea value={description} onChange={(e) => setDescription(e.target.value)} rows={3} className={`${inputCls} resize-none`} />
            </div>
            <div>
              <label className="text-xs font-medium text-foreground mb-1.5 block">Target providers</label>
              <div className="flex flex-wrap gap-2">
                {schemaProviders.map((id) => (
                  <button
                    key={id}
                    type="button"
                    onClick={() => toggleProvider(id)}
                    className={`inline-flex items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs font-medium transition capitalize ${
                      selectedProviders.includes(id)
                        ? 'border-primary/30 bg-primary/10 text-primary'
                        : 'border-border/60 text-muted-foreground hover:border-border'
                    }`}
                  >
                    {id}
                  </button>
                ))}
              </div>
            </div>

            {validationErrors.length > 0 && (
              <div className="rounded-md border border-destructive/30 bg-destructive/5 px-3 py-2">
                {validationErrors.map((err, i) => (
                  <p key={i} className="text-xs text-destructive">{err}</p>
                ))}
              </div>
            )}
          </div>

          <div className="flex items-center justify-end gap-2 border-t border-border/60 px-5 py-3.5">
            <button onClick={close} className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground">
              Cancel
            </button>
            <button onClick={handleSave} disabled={!canSave} className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90 disabled:opacity-50">
              Save
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
