import { useState, useCallback } from 'react'
import { RotateCcw, X, Plus } from 'lucide-react'
import type { JsonValue } from '@ship/ui'

interface VarEditorProps {
  currentValue: unknown
  defaultValue: JsonValue | undefined
  onSave: (valueJson: string) => void
  isPending: boolean
}

function hasChanged(current: unknown, defaultVal: JsonValue | undefined): boolean {
  if (current === undefined) return false
  return JSON.stringify(current) !== JSON.stringify(defaultVal)
}

function ResetButton({
  current,
  defaultVal,
  onReset,
  isPending,
}: {
  current: unknown
  defaultVal: JsonValue | undefined
  onReset: () => void
  isPending: boolean
}) {
  if (!hasChanged(current, defaultVal)) return null
  return (
    <button
      onClick={onReset}
      disabled={isPending}
      title="Reset to default"
      className="p-0.5 text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
    >
      <RotateCcw className="size-3" />
    </button>
  )
}

export function BoolEditor({ currentValue, defaultValue, onSave, isPending }: VarEditorProps) {
  const active = typeof currentValue === 'boolean' ? currentValue : (defaultValue === true)
  return (
    <div className="flex items-center gap-2">
      <button
        onClick={() => onSave(JSON.stringify(!active))}
        disabled={isPending}
        className="relative w-7 h-4 rounded-full transition-colors disabled:opacity-50 cursor-pointer"
        style={{ backgroundColor: active ? 'var(--color-primary)' : 'var(--color-muted)' }}
      >
        <div
          className={`size-3 rounded-full bg-white absolute top-0.5 transition-transform ${
            active ? 'translate-x-3.5' : 'translate-x-0.5'
          }`}
        />
      </button>
      <span className="text-[11px] text-foreground font-mono">{active ? 'true' : 'false'}</span>
      <ResetButton
        current={currentValue}
        defaultVal={defaultValue}
        onReset={() => onSave(JSON.stringify(defaultValue ?? false))}
        isPending={isPending}
      />
    </div>
  )
}

export function EnumEditor({
  currentValue,
  defaultValue,
  values,
  onSave,
  isPending,
}: VarEditorProps & { values: string[] }) {
  const active = typeof currentValue === 'string' ? currentValue : (typeof defaultValue === 'string' ? defaultValue : '')
  return (
    <div className="flex items-center gap-1 flex-wrap">
      {values.map((val) => (
        <button
          key={val}
          onClick={() => onSave(JSON.stringify(val))}
          disabled={isPending}
          className={`text-[10px] px-1.5 py-0.5 rounded border transition-colors cursor-pointer disabled:opacity-50 ${
            val === active
              ? 'border-primary/50 bg-primary/10 text-primary font-medium'
              : 'border-border bg-muted text-muted-foreground hover:border-foreground/30'
          }`}
        >
          {val}
        </button>
      ))}
      <ResetButton
        current={currentValue}
        defaultVal={defaultValue}
        onReset={() => onSave(JSON.stringify(defaultValue ?? ''))}
        isPending={isPending}
      />
    </div>
  )
}

export function StringEditor({ currentValue, defaultValue, onSave, isPending }: VarEditorProps) {
  const resolved = typeof currentValue === 'string' ? currentValue : (typeof defaultValue === 'string' ? defaultValue : '')
  const [draft, setDraft] = useState(resolved)
  const [focused, setFocused] = useState(false)

  // Sync draft when external value changes (and user is not editing)
  const displayValue = focused ? draft : resolved

  const commit = useCallback(() => {
    if (draft !== resolved) {
      onSave(JSON.stringify(draft))
    }
    setFocused(false)
  }, [draft, resolved, onSave])

  return (
    <div className="flex items-center gap-1.5">
      <input
        type="text"
        value={displayValue}
        onChange={(e) => { setDraft(e.target.value); setFocused(true) }}
        onFocus={() => { setDraft(resolved); setFocused(true) }}
        onBlur={commit}
        onKeyDown={(e) => { if (e.key === 'Enter') commit() }}
        disabled={isPending}
        placeholder={typeof defaultValue === 'string' ? defaultValue : ''}
        className="flex-1 min-w-0 bg-muted/60 border border-border rounded px-2 py-1 text-[11px] text-foreground font-mono placeholder:text-muted-foreground/50 focus:outline-none focus:border-primary/50 disabled:opacity-50"
      />
      <ResetButton
        current={currentValue}
        defaultVal={defaultValue}
        onReset={() => { setDraft(typeof defaultValue === 'string' ? defaultValue : ''); onSave(JSON.stringify(defaultValue ?? '')) }}
        isPending={isPending}
      />
    </div>
  )
}

export function ArrayEditor({ currentValue, defaultValue, onSave, isPending }: VarEditorProps) {
  const currentArr = Array.isArray(currentValue) ? currentValue as string[] : (Array.isArray(defaultValue) ? defaultValue as string[] : [])
  const [addInput, setAddInput] = useState('')

  const removeItem = useCallback((idx: number) => {
    const next = currentArr.filter((_, i) => i !== idx)
    onSave(JSON.stringify(next))
  }, [currentArr, onSave])

  const addItem = useCallback(() => {
    const trimmed = addInput.trim()
    if (!trimmed) return
    const next = [...currentArr, trimmed]
    onSave(JSON.stringify(next))
    setAddInput('')
  }, [addInput, currentArr, onSave])

  return (
    <div className="space-y-1.5">
      <div className="flex flex-wrap gap-1">
        {currentArr.map((item, idx) => (
          <span
            key={`${item}-${idx}`}
            className="inline-flex items-center gap-0.5 text-[10px] px-1.5 py-0.5 rounded border border-border bg-muted text-foreground/80"
          >
            {String(item)}
            <button
              onClick={() => removeItem(idx)}
              disabled={isPending}
              className="text-muted-foreground hover:text-destructive transition-colors disabled:opacity-50"
            >
              <X className="size-2.5" />
            </button>
          </span>
        ))}
        {currentArr.length === 0 && (
          <span className="text-[10px] text-muted-foreground">empty</span>
        )}
      </div>
      <div className="flex items-center gap-1">
        <input
          type="text"
          value={addInput}
          onChange={(e) => setAddInput(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter') addItem() }}
          disabled={isPending}
          placeholder="Add item..."
          className="flex-1 min-w-0 bg-muted/60 border border-border rounded px-2 py-1 text-[10px] text-foreground font-mono placeholder:text-muted-foreground/50 focus:outline-none focus:border-primary/50 disabled:opacity-50"
        />
        <button
          onClick={addItem}
          disabled={isPending || !addInput.trim()}
          className="p-1 text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
        >
          <Plus className="size-3" />
        </button>
        <ResetButton
          current={currentValue}
          defaultVal={defaultValue}
          onReset={() => onSave(JSON.stringify(defaultValue ?? []))}
          isPending={isPending}
        />
      </div>
    </div>
  )
}
