import { useState, useEffect } from 'react'
import { Plus, Trash2, BookOpen, Search } from 'lucide-react'
import type { Skill } from '#/features/compiler/types'

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

function skillSlug(skill: Skill): string {
  if (skill.id) return skill.id
  return skill.name.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '') || 'untitled'
}

export function SkillsForm({ skills, onChange }: Props) {
  const [selectedIdx, setSelectedIdx] = useState<number | null>(skills.length > 0 ? 0 : null)
  const [localContent, setLocalContent] = useState(skills[0]?.content ?? '')
  const [search, setSearch] = useState('')

  useEffect(() => {
    if (selectedIdx !== null) {
      setLocalContent(skills[selectedIdx]?.content ?? '')
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedIdx])

  const add = () => {
    const id = `new-skill-${Date.now()}`
    const next = [...skills, { ...EMPTY, id, name: 'New Skill' }]
    onChange(next)
    setSelectedIdx(next.length - 1)
    setLocalContent('')
  }

  const remove = (idx: number) => {
    const next = skills.filter((_, i) => i !== idx)
    onChange(next)
    const newIdx = next.length > 0 ? Math.min(idx, next.length - 1) : null
    setSelectedIdx(newIdx)
    setLocalContent(newIdx !== null ? (next[newIdx]?.content ?? '') : '')
  }

  const update = (idx: number, patch: Partial<Skill>) => {
    onChange(skills.map((s, i) => (i === idx ? { ...s, ...patch } : s)))
  }

  const handleContentChange = (v: string) => {
    setLocalContent(v)
    if (selectedIdx !== null) update(selectedIdx, { content: v })
  }

  const selected = selectedIdx !== null ? skills[selectedIdx] : null

  const filtered = search
    ? skills.filter((s) => {
        const q = search.toLowerCase()
        return s.name.toLowerCase().includes(q) || skillSlug(s).toLowerCase().includes(q)
      })
    : skills

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="flex flex-1 min-h-0 overflow-hidden">
        {/* Left: skill list */}
        <div className="flex w-56 shrink-0 flex-col border-r border-border/60 bg-muted/20">
          {/* Search */}
          <div className="p-2">
            <div className="flex items-center gap-1.5 rounded border border-border/60 bg-background px-2 py-1.5">
              <Search className="size-3 text-muted-foreground/40" />
              <input
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                placeholder="Search..."
                className="min-w-0 flex-1 bg-transparent text-[11px] text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
              />
            </div>
          </div>

          {/* List */}
          <div className="flex-1 overflow-y-auto px-1.5 pb-1.5">
            <div className="px-2 py-1 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/40">
              Your collection
            </div>
            {filtered.length === 0 && (
              <p className="px-2 py-3 text-[11px] italic text-muted-foreground/40">
                {search ? 'No matches.' : 'No skills yet.'}
              </p>
            )}
            {filtered.map((skill) => {
              const realIdx = skills.indexOf(skill)
              return (
                <div
                  key={skill.id || realIdx}
                  className={`group flex items-center gap-2 rounded-md px-2 py-1.5 cursor-pointer transition-colors ${
                    selectedIdx === realIdx
                      ? 'bg-primary/10 border-l-2 border-primary text-foreground'
                      : 'text-muted-foreground hover:bg-muted/50'
                  }`}
                  onClick={() => setSelectedIdx(realIdx)}
                >
                  <BookOpen className="size-3 shrink-0" />
                  <span className="flex-1 truncate text-[11px]">{skill.name || skillSlug(skill)}</span>
                  <button
                    onClick={(e) => { e.stopPropagation(); remove(realIdx) }}
                    className="hidden size-4 items-center justify-center rounded text-muted-foreground/40 transition hover:text-destructive group-hover:flex"
                    title="Remove"
                    aria-label="Remove skill"
                  >
                    <Trash2 className="size-2.5" />
                  </button>
                </div>
              )
            })}
          </div>

          {/* Bottom add button */}
          <div className="border-t border-border/60 p-2">
            <button
              onClick={add}
              className="flex w-full items-center justify-center gap-1.5 rounded-lg border border-dashed border-primary/30 bg-primary/5 py-2 text-[11px] font-medium text-primary transition hover:bg-primary/10"
            >
              <Plus className="size-3" />
              New skill
            </button>
          </div>
        </div>

        {/* Right: detail panel */}
        <div className="flex flex-1 min-w-0 min-h-0 flex-col">
          {selected && selectedIdx !== null ? (
            <>
              {/* Detail header */}
              <div className="flex items-center justify-between border-b border-border/60 px-4 py-2.5 shrink-0">
                <div className="flex items-center gap-2 min-w-0">
                  <input
                    value={selected.name}
                    onChange={(e) => update(selectedIdx, { name: e.target.value })}
                    placeholder="Skill name"
                    className="min-w-0 bg-transparent text-sm font-semibold text-foreground placeholder:text-muted-foreground/60 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                    spellCheck={false}
                  />
                  <span className="text-[10px] text-muted-foreground/40">/ SKILL.md</span>
                </div>
                <button
                  onClick={() => remove(selectedIdx)}
                  className="h-6 px-2 rounded border border-destructive/30 bg-destructive/5 text-[10px] text-destructive transition hover:bg-destructive/10"
                >
                  Remove
                </button>
              </div>

              {/* Metadata: slug + description */}
              <div className="flex items-center gap-3 border-b border-border/60 px-4 py-2 shrink-0">
                <div className="flex items-center gap-2">
                  <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/40 whitespace-nowrap">ID</span>
                  <input
                    value={selected.id}
                    onChange={(e) => update(selectedIdx, { id: e.target.value })}
                    placeholder="slug"
                    className="w-28 bg-transparent font-mono text-[11px] text-muted-foreground placeholder:text-muted-foreground/40 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                    spellCheck={false}
                  />
                </div>
                <div className="h-4 w-px bg-border/60" />
                <div className="flex items-center gap-2 flex-1 min-w-0">
                  <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/40 whitespace-nowrap">Trigger</span>
                  <input
                    value={selected.description ?? ''}
                    onChange={(e) => update(selectedIdx, { description: e.target.value || null })}
                    placeholder="when to use this skill..."
                    className="flex-1 min-w-0 bg-transparent text-[11px] text-muted-foreground placeholder:text-muted-foreground/40 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                    spellCheck={false}
                  />
                </div>
              </div>

              {/* Content editor */}
              <div className="flex-1 min-h-0 overflow-hidden p-3">
                <textarea
                  key={selected.id || selectedIdx}
                  value={localContent}
                  onChange={(e) => handleContentChange(e.target.value)}
                  placeholder={'# Instructions\n\nDescribe what the agent should do and when to use this skill...'}
                  spellCheck={false}
                  className="h-full w-full resize-none rounded-lg border border-border/60 bg-background p-3 font-mono text-xs text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/50 focus:border-primary/30"
                />
              </div>
            </>
          ) : (
            <div className="flex flex-1 flex-col items-center justify-center gap-3 p-8 text-center text-muted-foreground">
              <BookOpen className="size-8 opacity-20" />
              <div>
                <p className="text-sm font-medium">Select a skill to edit</p>
                <p className="mt-1 text-xs opacity-60">Or create a new one to get started.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
