import { useState, useEffect } from 'react'
import { Plus, Trash2, BookOpen } from 'lucide-react'
import {
  CustomMilkdownEditor,
  FileTree, FileTreeFile, FileTreeFolder, FileTreeActions, FileTreeName,
} from '@ship/primitives'
import type { Skill } from '@ship/ui'

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

/** Folder name = skill slug (id). Falls back to slugified name. */
function skillSlug(skill: Skill): string {
  if (skill.id) return skill.id
  return skill.name.toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '') || 'untitled'
}

export function SkillsForm({ skills, onChange }: Props) {
  const [selectedIdx, setSelectedIdx] = useState<number | null>(skills.length > 0 ? 0 : null)
  // Local content buffer — decouples CustomMilkdownEditor from parent re-renders
  const [localContent, setLocalContent] = useState(skills[0]?.content ?? '')

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
  const selectedFolder = selected ? skillSlug(selected) : undefined

  return (
    <div className="flex h-full min-h-0 overflow-hidden rounded-xl border border-border/60">
      {/* ── Left: file explorer ─────────────────────────────────────── */}
      <div className="flex w-56 shrink-0 flex-col border-r border-border/60 bg-muted/20">
        <div className="flex items-center justify-between border-b border-border/60 px-3 py-2">
          <span className="text-[10px] font-semibold uppercase tracking-wide text-muted-foreground">Explorer</span>
          <button
            onClick={add}
            className="flex size-5 items-center justify-center rounded text-muted-foreground transition hover:bg-primary/10 hover:text-primary"
            title="New skill"
          >
            <Plus className="size-3.5" />
          </button>
        </div>

        <div className="flex-1 overflow-y-auto p-1.5">
          <FileTree
            selectedPath={selectedFolder ? `${selectedFolder}/SKILL.md` : undefined}
            onSelect={(path) => {
              // Clicking SKILL.md inside a folder selects that skill
              const folder = path.replace('/SKILL.md', '')
              const idx = skills.findIndex((s) => skillSlug(s) === folder)
              if (idx !== -1) setSelectedIdx(idx)
            }}
            defaultExpanded={new Set(skills.map(skillSlug))}
            className="border-0 rounded-none bg-transparent text-xs"
          >
            {skills.length === 0 && (
              <p className="px-2 py-3 text-[11px] italic text-muted-foreground">No skills yet.</p>
            )}
            {skills.map((skill, idx) => {
              const slug = skillSlug(skill)
              return (
                <FileTreeFolder key={skill.id || idx} path={slug} name={slug}>
                  <FileTreeFile path={`${slug}/SKILL.md`} name="SKILL.md">
                    <span className="size-4 shrink-0" />
                    {/* file icon from FileTreeFile default */}
                    <FileTreeName className="flex-1 text-[11px]">SKILL.md</FileTreeName>
                    <FileTreeActions>
                      <button
                        onClick={() => remove(idx)}
                        className="flex size-4 items-center justify-center rounded text-muted-foreground/40 transition hover:text-destructive"
                        title="Delete skill"
                      >
                        <Trash2 className="size-2.5" />
                      </button>
                    </FileTreeActions>
                  </FileTreeFile>
                </FileTreeFolder>
              )
            })}
          </FileTree>
        </div>

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

      {/* ── Right: editor ───────────────────────────────────────────── */}
      <div className="flex flex-1 min-w-0 min-h-0 flex-col">
        {selected && selectedIdx !== null ? (
          <>
            {/* Metadata row — maps to SKILL.md frontmatter fields */}
            <div className="flex items-center gap-3 border-b border-border/60 bg-card/50 px-4 py-2.5 shrink-0">
              <BookOpen className="size-3.5 shrink-0 text-muted-foreground" />
              <input
                value={selected.name}
                onChange={(e) => update(selectedIdx, { name: e.target.value })}
                placeholder="Skill name"
                className="min-w-0 flex-1 bg-transparent text-sm font-semibold placeholder:text-muted-foreground/60 focus:outline-none"
                spellCheck={false}
              />
              <input
                value={selected.id}
                onChange={(e) => update(selectedIdx, { id: e.target.value })}
                placeholder="slug"
                className="w-28 bg-transparent font-mono text-[11px] text-muted-foreground placeholder:text-muted-foreground/40 focus:outline-none"
                spellCheck={false}
              />
              <input
                value={selected.description ?? ''}
                onChange={(e) => update(selectedIdx, { description: e.target.value || null })}
                placeholder="description (when to use…)"
                className="w-64 bg-transparent text-[11px] text-muted-foreground placeholder:text-muted-foreground/40 focus:outline-none"
                spellCheck={false}
              />
            </div>

            {/* SKILL.md body editor — keyed by id so it remounts on skill switch */}
            <div className="flex-1 min-h-0 overflow-hidden p-3">
              <CustomMilkdownEditor
                key={selected.id || selectedIdx}
                value={localContent}
                onChange={handleContentChange}
                placeholder={'# Instructions\n\nDescribe what the agent should do and when to use this skill...'}
                fillHeight
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
  )
}
