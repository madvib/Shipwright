/** Filter bar for the Skills IDE file explorer. */

import { useState, useMemo } from 'react'
import type { LibrarySkill } from './useSkillsLibrary'

export type SkillFilter = 'all' | 'smart' | 'documented'

interface FilterCounts {
  all: number
  smart: number
  documented: number
  tags: Map<string, number>
}

export function computeFilterCounts(skills: LibrarySkill[]): FilterCounts {
  let smart = 0
  let documented = 0
  const tags = new Map<string, number>()

  for (const s of skills) {
    if (s.varsSchema != null) smart++
    if (Object.keys(s.referenceDocs ?? {}).length > 0) documented++
    for (const tag of s.tags ?? []) {
      tags.set(tag, (tags.get(tag) ?? 0) + 1)
    }
  }

  return { all: skills.length, smart, documented, tags }
}

export function applyFilters(
  skills: LibrarySkill[],
  filter: SkillFilter,
  activeTags: Set<string>,
): LibrarySkill[] {
  return skills.filter((s) => {
    if (filter === 'smart' && s.varsSchema == null) return false
    if (filter === 'documented' && Object.keys(s.referenceDocs ?? {}).length === 0) return false
    if (activeTags.size > 0) {
      const skillTags = new Set(s.tags ?? [])
      for (const tag of activeTags) {
        if (!skillTags.has(tag)) return false
      }
    }
    return true
  })
}

interface Props {
  allSkills: LibrarySkill[]
  activeFilter: SkillFilter
  activeTags: Set<string>
  onFilterChange: (f: SkillFilter) => void
  onTagToggle: (tag: string) => void
}

const FILTER_BUTTONS: { key: SkillFilter; label: string }[] = [
  { key: 'all', label: 'All' },
  { key: 'smart', label: 'Smart' },
  { key: 'documented', label: 'Docs' },
]

export function SkillFilterBar({ allSkills, activeFilter, activeTags, onFilterChange, onTagToggle }: Props) {
  const [showTags, setShowTags] = useState(false)
  const counts = useMemo(() => computeFilterCounts(allSkills), [allSkills])
  const sortedTags = useMemo(
    () => Array.from(counts.tags.entries()).sort((a, b) => b[1] - a[1]),
    [counts.tags],
  )

  if (allSkills.length === 0) return null

  return (
    <div className="px-2.5 pb-1.5 space-y-1">
      <div className="flex items-center gap-1 flex-wrap">
        {FILTER_BUTTONS.map(({ key, label }) => {
          const count = key === 'all' ? counts.all : key === 'smart' ? counts.smart : counts.documented
          const isActive = activeFilter === key
          return (
            <button
              key={key}
              onClick={() => onFilterChange(key)}
              className={`px-2 py-0.5 text-[10px] rounded-full border transition-colors ${
                isActive
                  ? 'border-primary/50 bg-primary/10 text-primary font-medium'
                  : 'border-border text-muted-foreground hover:text-foreground hover:border-border'
              }`}
            >
              {label}
              <span className="ml-1 opacity-60">{count}</span>
            </button>
          )
        })}
        {sortedTags.length > 0 && (
          <button
            onClick={() => setShowTags((p) => !p)}
            className={`px-2 py-0.5 text-[10px] rounded-full border transition-colors ${
              activeTags.size > 0
                ? 'border-primary/50 bg-primary/10 text-primary font-medium'
                : 'border-border text-muted-foreground hover:text-foreground'
            }`}
          >
            Tags{activeTags.size > 0 ? ` (${activeTags.size})` : ''}
          </button>
        )}
      </div>

      {showTags && sortedTags.length > 0 && (
        <div className="flex flex-wrap gap-1 pt-0.5">
          {sortedTags.map(([tag, count]) => {
            const isActive = activeTags.has(tag)
            return (
              <button
                key={tag}
                onClick={() => onTagToggle(tag)}
                className={`px-1.5 py-px text-[9px] rounded border transition-colors ${
                  isActive
                    ? 'border-sky-500/50 bg-sky-500/15 text-sky-400'
                    : 'border-border text-muted-foreground hover:text-foreground'
                }`}
              >
                {tag}
                <span className="ml-0.5 opacity-50">{count}</span>
              </button>
            )
          })}
        </div>
      )}
    </div>
  )
}
