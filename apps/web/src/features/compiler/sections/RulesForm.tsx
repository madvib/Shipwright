import { useState } from 'react'
import { Plus, Trash2, ScrollText, ChevronDown, ChevronRight } from 'lucide-react'
import { MarkdownEditor, AutocompleteInput } from '@ship/primitives'
import type { Rule } from '@ship/ui'

const RULE_FILENAME_SUGGESTIONS = [
  { value: 'AGENTS.md' },
  { value: 'CLAUDE.md' },
  { value: 'CURSOR.md' },
  { value: 'code-style.md' },
  { value: 'commit-conventions.md' },
  { value: 'project-guidelines.md' },
  { value: 'testing-rules.md' },
  { value: 'architecture.md' },
]

interface Props {
  rules: Rule[]
  onChange: (rules: Rule[]) => void
}

export function RulesForm({ rules, onChange }: Props) {
  const [expanded, setExpanded] = useState<number | null>(null)

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
        <p className="rounded-lg border border-dashed border-border/60 p-4 text-center text-xs text-muted-foreground">
          No rules added yet.
        </p>
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
                <AutocompleteInput
                  value={rule.file_name}
                  options={RULE_FILENAME_SUGGESTIONS}
                  onValueChange={(v) => update(idx, { file_name: v })}
                  placeholder="e.g. code-style.md"
                  allowCustom
                  syncOnInput
                  autoCorrect="off"
                  spellCheck={false}
                  className="h-7 font-mono text-xs"
                />
              </div>
              <MarkdownEditor
                value={rule.content}
                onChange={(v) => update(idx, { content: v })}
                placeholder={'# Code Style\n\nAlways use explicit types...'}
                rows={10}
                showStats={false}
                showFrontmatter={false}
                showAiActions={false}
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
