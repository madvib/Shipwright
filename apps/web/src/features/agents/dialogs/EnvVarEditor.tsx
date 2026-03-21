import { Input } from '@ship/primitives'
import { Plus, X } from 'lucide-react'

interface EnvVar {
  key: string
  value: string
}

interface EnvVarEditorProps {
  entries: EnvVar[]
  onChange: (entries: EnvVar[]) => void
}

export function EnvVarEditor({ entries, onChange }: EnvVarEditorProps) {
  const addRow = () => onChange([...entries, { key: '', value: '' }])

  const removeRow = (index: number) =>
    onChange(entries.filter((_, i) => i !== index))

  const updateRow = (index: number, field: 'key' | 'value', val: string) =>
    onChange(entries.map((e, i) => (i === index ? { ...e, [field]: val } : e)))

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between">
        <label className="text-xs font-medium text-foreground">Environment variables</label>
        <button
          type="button"
          onClick={addRow}
          className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition"
        >
          <Plus className="size-3" />
          Add
        </button>
      </div>
      {entries.map((entry, i) => (
        <div key={i} className="flex items-center gap-2">
          <Input
            value={entry.key}
            onChange={(e) => updateRow(i, 'key', e.target.value)}
            placeholder="KEY"
            className="flex-1 font-mono text-xs"
          />
          <span className="text-muted-foreground text-xs">=</span>
          <Input
            value={entry.value}
            onChange={(e) => updateRow(i, 'value', e.target.value)}
            placeholder="value"
            className="flex-1 font-mono text-xs"
          />
          <button
            type="button"
            onClick={() => removeRow(i)}
            className="text-muted-foreground hover:text-destructive transition shrink-0"
          >
            <X className="size-3.5" />
          </button>
        </div>
      ))}
      {entries.length === 0 && (
        <p className="text-xs text-muted-foreground/60 italic">No environment variables</p>
      )}
    </div>
  )
}
