import { useState } from 'react'
import { Plus } from 'lucide-react'
import type { HookConfig } from '@ship/ui'
import { HookEditorDialog } from './HookEditorDialog'

interface ProviderHooksTabProps {
  hooks: HookConfig[]
  onChange: (hooks: HookConfig[]) => void
}

export function ProviderHooksTab({ hooks, onChange }: ProviderHooksTabProps) {
  const [editorOpen, setEditorOpen] = useState(false)
  const [editTarget, setEditTarget] = useState<{ index: number; hook: HookConfig } | null>(null)

  const handleAdd = () => {
    setEditTarget(null)
    setEditorOpen(true)
  }

  const handleEdit = (index: number) => {
    setEditTarget({ index, hook: hooks[index] })
    setEditorOpen(true)
  }

  const handleSave = (hook: HookConfig) => {
    if (editTarget) {
      onChange(hooks.map((h, i) => (i === editTarget.index ? hook : h)))
    } else {
      onChange([...hooks, hook])
    }
  }

  const handleDelete = () => {
    if (editTarget) {
      onChange(hooks.filter((_, i) => i !== editTarget.index))
    }
  }

  const handleRemove = (index: number) => {
    onChange(hooks.filter((_, i) => i !== index))
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center justify-between">
        <p className="text-xs text-muted-foreground/60">
          Hooks run shell commands at specific lifecycle events.
        </p>
        <button
          onClick={handleAdd}
          className="flex items-center gap-1 rounded-md border border-border/60 px-2.5 py-1 text-[11px] text-muted-foreground hover:border-primary hover:text-primary transition-colors"
        >
          <Plus className="size-3" />
          Add hook
        </button>
      </div>

      <div className="flex flex-col gap-1.5">
        {hooks.map((hook, i) => (
          <div
            key={i}
            onClick={() => handleEdit(i)}
            className="flex items-center gap-2 rounded-lg border border-border/40 bg-card/30 px-3 py-2 cursor-pointer hover:border-border transition-colors"
          >
            <span className="shrink-0 rounded bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-primary">
              {hook.trigger}
            </span>
            <span className="flex-1 truncate font-mono text-[11px] text-muted-foreground/60">
              {hook.command}
            </span>
            {hook.matcher && (
              <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 text-[9px] font-mono text-muted-foreground/50">
                {hook.matcher}
              </span>
            )}
            <button
              onClick={(e) => { e.stopPropagation(); handleRemove(i) }}
              className="shrink-0 text-muted-foreground/30 hover:text-destructive transition-colors text-sm"
            >
              x
            </button>
          </div>
        ))}
        {hooks.length === 0 && (
          <p className="text-xs text-muted-foreground/40 italic py-2">No hooks configured</p>
        )}
      </div>

      <HookEditorDialog
        open={editorOpen}
        onOpenChange={setEditorOpen}
        hook={editTarget?.hook ?? null}
        onSave={handleSave}
        onDelete={editTarget ? handleDelete : undefined}
      />
    </div>
  )
}
