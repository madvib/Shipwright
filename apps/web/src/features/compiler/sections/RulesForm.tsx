import { useState } from 'react'
import { Plus, Trash2, ScrollText, ChevronDown, ChevronRight } from 'lucide-react'
import type { Rule } from '#/features/compiler/types'

const RULE_FILENAME_SUGGESTIONS = [
  'AGENTS.md', 'CLAUDE.md', 'CURSOR.md', 'code-style.md',
  'commit-conventions.md', 'project-guidelines.md', 'testing-rules.md', 'architecture.md',
]

interface Props {
  rules: Rule[]
  onChange: (rules: Rule[]) => void
}

export function RulesForm({ rules, onChange }: Props) {
  const [expanded, setExpanded] = useState<number | null>(null)
  const [showSuggestions, setShowSuggestions] = useState(false)

  const add = () => {
    const next = [...rules, { file_name: `rule-${rules.length + 1}.md`, content: '' }]
    onChange(next)
    setExpanded(next.length - 1)
  }

  const remove = (idx: number) => {
    onChange(rules.filter((_, i) => i !== idx))
    if (expanded === idx) setExpanded(null)
  }

  const update = (idx: number, patch: Partial<Rule>) => {
    onChange(rules.map((r, i) => (i === idx ? { ...r, ...patch } : r)))
  }

  return (
    <div className="space-y-2">
      {rules.length === 0 && (
        <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border/60 py-8 px-4 text-center">
          <ScrollText className="size-5 text-muted-foreground/40" />
          <p className="text-xs text-muted-foreground">No rules added yet.</p>
        </div>
      )}

      {rules.map((rule, idx) => (
        <div
          key={idx}
          className={`overflow-hidden rounded-xl border transition ${expanded === idx ? 'border-border bg-card' : 'border-border/60 bg-card/50'}`}
        >
          <div className="flex items-center gap-2 px-3 py-2.5">
            <button
              onClick={() => setExpanded(expanded === idx ? null : idx)}
              className="flex flex-1 items-center gap-2 text-left min-w-0"
            >
              <ScrollText className="size-3.5 shrink-0 text-amber-500" />
              <span className="min-w-0 flex-1 truncate font-mono text-xs">
                {rule.file_name || <span className="font-sans italic text-muted-foreground">Unnamed rule</span>}
              </span>
              {expanded === idx
                ? <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" />
                : <ChevronRight className="size-3.5 shrink-0 text-muted-foreground" />
              }
            </button>
            <button
              onClick={() => remove(idx)}
              title="Remove rule"
              aria-label="Remove rule"
              className="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground/60 transition hover:bg-destructive/10 hover:text-destructive"
            >
              <Trash2 className="size-3" />
            </button>
          </div>

          {expanded === idx && (
            <div className="border-t border-border/60 bg-muted/20 p-3 space-y-3">
              <div className="space-y-1">
                <label className="block text-[11px] font-medium text-muted-foreground">
                  Filename <span className="font-normal opacity-60">— included in agent context as this path</span>
                </label>
                <div className="relative">
                  <input
                    value={rule.file_name}
                    onChange={(e) => update(idx, { file_name: e.target.value })}
                    onFocus={() => setShowSuggestions(true)}
                    onBlur={() => setTimeout(() => setShowSuggestions(false), 150)}
                    placeholder="e.g. code-style.md"
                    autoCorrect="off"
                    spellCheck={false}
                    className="h-7 w-full rounded-md border border-border/60 bg-background px-2 font-mono text-xs focus:outline-none focus:border-border"
                  />
                  {showSuggestions && expanded === idx && (
                    <div className="absolute left-0 top-full z-10 mt-1 w-full rounded-md border border-border/60 bg-card shadow-lg">
                      {RULE_FILENAME_SUGGESTIONS
                        .filter((s) => s.toLowerCase().includes(rule.file_name.toLowerCase()))
                        .map((s) => (
                          <button
                            key={s}
                            onMouseDown={() => update(idx, { file_name: s })}
                            className="block w-full px-2 py-1.5 text-left font-mono text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
                          >
                            {s}
                          </button>
                        ))
                      }
                    </div>
                  )}
                </div>
              </div>
              <textarea
                value={rule.content}
                onChange={(e) => update(idx, { content: e.target.value })}
                placeholder={'# Code Style\n\nAlways use explicit types...'}
                rows={10}
                spellCheck={false}
                className="w-full resize-y rounded-lg border border-border/60 bg-background p-3 font-mono text-xs text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus:border-border"
              />
            </div>
          )}
        </div>
      ))}

      <button
        onClick={add}
        className="flex w-full items-center justify-center gap-1.5 rounded-xl border border-dashed border-amber-500/30 bg-amber-500/5 py-2.5 text-xs font-medium text-amber-600 dark:text-amber-400 transition hover:bg-amber-500/10"
      >
        <Plus className="size-3.5" />
        Add rule
      </button>
    </div>
  )
}
