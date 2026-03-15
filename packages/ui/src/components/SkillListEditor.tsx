import { useState } from 'react'
import { Plus, Trash2, BookOpen, ChevronDown, ChevronRight } from 'lucide-react'
import type { Skill } from '../types'

interface Props {
  skills: Skill[]
  onChange: (skills: Skill[]) => void
}

const EMPTY: Skill = {
  id: '',
  name: '',
  content: '',
  description: null,
  source: 'custom',
  author: null,
  version: null,
}

export function SkillListEditor({ skills, onChange }: Props) {
  const [expanded, setExpanded] = useState<number | null>(null)

  const add = () => {
    const next = [...skills, { ...EMPTY, id: `skill-${Date.now()}` }]
    onChange(next)
    setExpanded(next.length - 1)
  }

  const remove = (idx: number) => {
    onChange(skills.filter((_, i) => i !== idx))
    if (expanded === idx) setExpanded(null)
  }

  const update = (idx: number, patch: Partial<Skill>) => {
    onChange(skills.map((s, i) => (i === idx ? { ...s, ...patch } : s)))
  }

  return (
    <div className="space-y-2">
      <p className="text-[11px] text-muted-foreground">
        Skills are slash commands injected into agent context. Describe tools, workflows, or domain knowledge.
      </p>

      {skills.length === 0 && (
        <p className="rounded-lg border border-dashed border-border/60 p-4 text-center text-xs text-muted-foreground">
          No skills added yet.
        </p>
      )}

      {skills.map((skill, idx) => (
        <div
          key={idx}
          className={`overflow-hidden rounded-xl border transition ${expanded === idx ? 'border-border bg-card' : 'border-border/60 bg-card/50'}`}
        >
          <div className="flex items-center gap-2 px-3 py-2.5">
            <button
              onClick={() => setExpanded(expanded === idx ? null : idx)}
              className="flex flex-1 items-center gap-2 text-left min-w-0"
            >
              <BookOpen className="size-3.5 shrink-0 text-cyan-500" />
              <span className="min-w-0 flex-1 truncate text-xs font-medium">
                {skill.name || <span className="text-muted-foreground italic">Unnamed skill</span>}
              </span>
              {skill.source && skill.source !== 'custom' && (
                <span className="rounded bg-cyan-500/10 px-1.5 py-0.5 text-[9px] font-medium text-cyan-600 dark:text-cyan-400">
                  {skill.source}
                </span>
              )}
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
              <div className="grid gap-3 sm:grid-cols-2">
                <div className="space-y-1">
                  <label className="block text-[11px] font-medium text-muted-foreground">Name</label>
                  <input
                    type="text"
                    value={skill.name}
                    onChange={(e) => update(idx, { name: e.target.value })}
                    placeholder="e.g. Ship Workflow"
                    className="h-7 w-full rounded-md border border-border bg-background px-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary/40"
                  />
                </div>
                <div className="space-y-1">
                  <label className="block text-[11px] font-medium text-muted-foreground">
                    ID <span className="font-normal opacity-60">(slash command trigger)</span>
                  </label>
                  <input
                    type="text"
                    value={skill.id}
                    onChange={(e) => update(idx, { id: e.target.value })}
                    placeholder="e.g. ship-workflow"
                    className="h-7 w-full rounded-md border border-border bg-background px-2 font-mono text-xs focus:outline-none focus:ring-1 focus:ring-primary/40"
                  />
                </div>
              </div>
              <div className="space-y-1">
                <label className="block text-[11px] font-medium text-muted-foreground">Description</label>
                <input
                  type="text"
                  value={skill.description ?? ''}
                  onChange={(e) => update(idx, { description: e.target.value || null })}
                  placeholder="Brief description"
                  className="h-7 w-full rounded-md border border-border bg-background px-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary/40"
                />
              </div>
              <div className="space-y-1">
                <label className="block text-[11px] font-medium text-muted-foreground">
                  Content <span className="font-normal opacity-60">(markdown)</span>
                </label>
                <textarea
                  value={skill.content}
                  onChange={(e) => update(idx, { content: e.target.value })}
                  placeholder={'# Skill Name\n\nDescribe what the agent should do when this skill is invoked...'}
                  spellCheck={false}
                  rows={8}
                  className="w-full resize-y rounded-md border border-border bg-background p-2 font-mono text-xs leading-relaxed focus:outline-none focus:ring-1 focus:ring-primary/40"
                />
              </div>
            </div>
          )}
        </div>
      ))}

      <button
        onClick={add}
        className="flex w-full items-center justify-center gap-1.5 rounded-xl border border-dashed border-cyan-500/30 bg-cyan-500/5 py-2.5 text-xs font-medium text-cyan-600 dark:text-cyan-400 transition hover:bg-cyan-500/10"
      >
        <Plus className="size-3.5" />
        Add skill
      </button>
    </div>
  )
}
