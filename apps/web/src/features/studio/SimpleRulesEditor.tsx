import { useState, useRef, useEffect } from 'react'
import { Plus } from 'lucide-react'

interface SimpleRulesEditorProps {
  rules: string[]
  onChange: (rules: string[]) => void
}

export function SimpleRulesEditor({ rules, onChange }: SimpleRulesEditorProps) {
  const [editIdx, setEditIdx] = useState<number | null>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (editIdx !== null) inputRef.current?.focus()
  }, [editIdx])

  const add = () => {
    const next = [...rules, '']
    onChange(next)
    setEditIdx(next.length - 1)
  }

  const update = (idx: number, val: string) => {
    onChange(rules.map((r, i) => (i === idx ? val : r)))
  }

  const remove = (idx: number) => {
    onChange(rules.filter((_, i) => i !== idx))
    setEditIdx(null)
  }

  const handleKey = (e: React.KeyboardEvent, idx: number) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      if (rules[idx] === '') {
        remove(idx)
      } else {
        const next = [...rules.slice(0, idx + 1), '', ...rules.slice(idx + 1)]
        onChange(next)
        setEditIdx(idx + 1)
      }
    } else if (e.key === 'Backspace' && rules[idx] === '') {
      e.preventDefault()
      remove(idx)
      setEditIdx(idx > 0 ? idx - 1 : null)
    } else if (e.key === 'Escape') {
      setEditIdx(null)
    }
  }

  return (
    <div className="rounded-lg border border-border/60 bg-muted/20 px-3 py-2 font-mono text-[11px]">
      {rules.length === 0 && editIdx === null && (
        <p className="text-muted-foreground/50 italic py-1">No rules yet — click Add to start</p>
      )}
      {rules.map((rule, idx) => (
        <div key={idx} className="group flex items-baseline gap-1.5 py-0.5">
          <span className="shrink-0 text-muted-foreground/50">-</span>
          {editIdx === idx ? (
            <input
              ref={inputRef}
              value={rule}
              onChange={(e) => update(idx, e.target.value)}
              onBlur={() => {
                if (rule === '') remove(idx)
                else setEditIdx(null)
              }}
              onKeyDown={(e) => handleKey(e, idx)}
              className="flex-1 bg-transparent text-foreground outline-none placeholder-muted-foreground/40"
              placeholder="Enter rule..."
              spellCheck={false}
            />
          ) : (
            <button
              onClick={() => setEditIdx(idx)}
              className="flex-1 text-left text-muted-foreground hover:text-foreground transition"
            >
              {rule || <span className="italic opacity-40">Empty rule</span>}
            </button>
          )}
        </div>
      ))}
      <button
        onClick={add}
        className="mt-1 flex items-center gap-1 text-muted-foreground/50 transition hover:text-muted-foreground"
      >
        <Plus className="size-3" />
        <span>Add rule</span>
      </button>
    </div>
  )
}
