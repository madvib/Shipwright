/** Filter bar for the Skills IDE file explorer. */

import { useState, useMemo, useRef, useEffect } from 'react'
import { Check, Search, X } from 'lucide-react'
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
          <TagsPopoverButton
            sortedTags={sortedTags}
            activeTags={activeTags}
            open={showTags}
            onOpenChange={setShowTags}
            onTagToggle={onTagToggle}
          />
        )}
      </div>
    </div>
  )
}

// -- Tags Popover -------------------------------------------------------------

interface TagsPopoverProps {
  sortedTags: [string, number][]
  activeTags: Set<string>
  open: boolean
  onOpenChange: (open: boolean) => void
  onTagToggle: (tag: string) => void
}

function TagsPopoverButton({ sortedTags, activeTags, open, onOpenChange, onTagToggle }: TagsPopoverProps) {
  const triggerRef = useRef<HTMLButtonElement>(null)
  const popoverRef = useRef<HTMLDivElement>(null)
  const [search, setSearch] = useState('')

  // Position popover below the trigger button
  useEffect(() => {
    if (!open || !triggerRef.current || !popoverRef.current) return
    const rect = triggerRef.current.getBoundingClientRect()
    const pop = popoverRef.current
    pop.style.top = `${rect.bottom + 4}px`
    pop.style.left = `${Math.max(8, rect.left)}px`
  }, [open])

  // Reset search when closing
  useEffect(() => {
    if (!open) setSearch('')
  }, [open])

  const filteredTags = search
    ? sortedTags.filter(([tag]) => tag.toLowerCase().includes(search.toLowerCase()))
    : sortedTags

  const clearAll = () => {
    for (const tag of activeTags) onTagToggle(tag)
  }

  return (
    <>
      <button
        ref={triggerRef}
        onClick={() => onOpenChange(!open)}
        className={`px-2 py-0.5 text-[10px] rounded-full border transition-colors ${
          activeTags.size > 0
            ? 'border-primary/50 bg-primary/10 text-primary font-medium'
            : 'border-border text-muted-foreground hover:text-foreground'
        }`}
      >
        Tags{activeTags.size > 0 ? ` (${activeTags.size})` : ''}
      </button>

      {open && (
        <>
          <div className="fixed inset-0 z-40" onClick={() => onOpenChange(false)} />
          <div
            ref={popoverRef}
            className="fixed z-50 w-52 rounded-lg border border-border bg-popover shadow-lg animate-in fade-in slide-in-from-top-1 duration-150"
          >
            {/* Search input */}
            <div className="px-2 py-1.5 border-b border-border">
              <div className="flex items-center gap-1.5 rounded border border-border bg-background/80 px-2 py-1 focus-within:border-primary/50 transition-colors">
                <Search className="size-3 text-muted-foreground shrink-0" />
                <input
                  value={search}
                  onChange={(e) => setSearch(e.target.value)}
                  placeholder="Filter tags..."
                  className="min-w-0 flex-1 bg-transparent text-[10px] text-foreground placeholder:text-muted-foreground focus:outline-none"
                  autoFocus
                  spellCheck={false}
                />
                {search && (
                  <button onClick={() => setSearch('')} className="text-muted-foreground hover:text-foreground">
                    <X className="size-2.5" />
                  </button>
                )}
              </div>
            </div>

            {/* Tag list */}
            <div className="max-h-[300px] overflow-y-auto p-1">
              {filteredTags.length === 0 ? (
                <p className="px-2 py-3 text-center text-[10px] text-muted-foreground">No matching tags</p>
              ) : (
                filteredTags.map(([tag, count]) => {
                  const isActive = activeTags.has(tag)
                  return (
                    <button
                      key={tag}
                      onClick={() => onTagToggle(tag)}
                      className={`flex items-center gap-2 w-full px-2 py-1 rounded text-left transition-colors ${
                        isActive ? 'bg-primary/10 text-primary' : 'text-foreground/70 hover:bg-muted/50 hover:text-foreground'
                      }`}
                    >
                      <span className={`flex size-3.5 shrink-0 items-center justify-center rounded border transition-colors ${
                        isActive ? 'border-primary bg-primary' : 'border-border'
                      }`}>
                        {isActive && <Check className="size-2.5 text-primary-foreground" />}
                      </span>
                      <span className="flex-1 truncate text-[10px]">{tag}</span>
                      <span className="text-[9px] text-muted-foreground tabular-nums">({count})</span>
                    </button>
                  )
                })
              )}
            </div>

            {/* Clear all */}
            {activeTags.size > 0 && (
              <div className="border-t border-border px-2 py-1.5">
                <button
                  onClick={clearAll}
                  className="w-full text-center text-[10px] text-muted-foreground hover:text-foreground transition-colors py-0.5"
                >
                  Clear all
                </button>
              </div>
            )}
          </div>
        </>
      )}
    </>
  )
}
