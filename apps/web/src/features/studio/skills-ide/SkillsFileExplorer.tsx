import {
  Search, Plus, ChevronRight, ChevronDown, FileText, Package, Zap,
} from 'lucide-react'
import type { Skill } from '@ship/ui'

interface Props {
  filteredSkills: Skill[]
  activeTabId: string | null
  expandedFolders: Set<string>
  searchQuery: string
  onSearchChange: (q: string) => void
  onToggleFolder: (id: string) => void
  onOpenSkill: (id: string) => void
  onCreateSkill: () => void
}

// Placeholder installed skills (read-only display)
const INSTALLED_SKILLS = [
  { id: 'code-review', name: 'code-review', version: 'v1.2.0' },
  { id: 'debug-expert', name: 'debug-expert', version: 'v1.0.0' },
  { id: 'frontend-design', name: 'frontend-design', version: 'v0.2.0' },
]

export function SkillsFileExplorer({
  filteredSkills,
  activeTabId,
  expandedFolders,
  searchQuery,
  onSearchChange,
  onToggleFolder,
  onOpenSkill,
  onCreateSkill,
}: Props) {
  return (
    <div className="flex w-60 shrink-0 flex-col border-r border-border/40 bg-card/30">
      {/* Search */}
      <div className="p-2.5 border-b border-border/30">
        <div className="flex items-center gap-2 rounded-md border border-border/50 bg-background/80 px-2.5 py-1.5 focus-within:border-primary/50 transition-colors">
          <Search className="size-3.5 text-muted-foreground/40 shrink-0" />
          <input
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search skills..."
            className="min-w-0 flex-1 bg-transparent text-xs text-foreground placeholder:text-muted-foreground/30 focus:outline-none"
            spellCheck={false}
          />
        </div>
      </div>

      {/* Scrollable sections */}
      <div className="flex-1 overflow-y-auto">
        {/* Project Skills */}
        <div className="py-1.5">
          <div className="flex items-center justify-between px-3.5 py-1">
            <span className="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/40">
              Project Skills
            </span>
            <button
              onClick={onCreateSkill}
              className="text-muted-foreground/40 hover:text-primary transition-colors"
              title="New skill"
            >
              <Plus className="size-3.5" />
            </button>
          </div>

          {filteredSkills.length === 0 && (
            <p className="px-4 py-3 text-[11px] italic text-muted-foreground/30">
              {searchQuery ? 'No matches.' : 'No skills yet.'}
            </p>
          )}

          {filteredSkills.map((skill) => {
            const expanded = expandedFolders.has(skill.id)
            const isActive = activeTabId === skill.id
            return (
              <div key={skill.id}>
                {/* Folder row */}
                <button
                  onClick={() => onToggleFolder(skill.id)}
                  className="flex w-full items-center gap-1.5 px-3 py-1 text-xs text-muted-foreground/60 hover:text-muted-foreground transition-colors"
                >
                  {expanded
                    ? <ChevronDown className="size-3 shrink-0" />
                    : <ChevronRight className="size-3 shrink-0" />}
                  <Zap className="size-3 shrink-0 text-primary/60" />
                  <span className="truncate">{skill.name || skill.id}</span>
                </button>

                {/* SKILL.md file */}
                {expanded && (
                  <button
                    onClick={() => onOpenSkill(skill.id)}
                    className={`flex w-full items-center gap-1.5 pl-9 pr-3 py-1 text-xs transition-colors border-l-2 ${
                      isActive
                        ? 'border-primary bg-primary/5 text-primary'
                        : 'border-transparent text-muted-foreground/50 hover:text-muted-foreground hover:bg-muted/30'
                    }`}
                  >
                    <FileText className="size-3 shrink-0" />
                    <span className="truncate">SKILL.md</span>
                    {skill.source && skill.source !== 'custom' && (
                      <span className="ml-auto text-[9px] text-muted-foreground/30 bg-muted/50 px-1.5 rounded">
                        {skill.source}
                      </span>
                    )}
                  </button>
                )}
              </div>
            )
          })}
        </div>

        <div className="h-px bg-border/20 mx-3" />

        {/* Installed */}
        <div className="py-1.5">
          <div className="px-3.5 py-1">
            <span className="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/40">
              Installed
            </span>
          </div>
          {INSTALLED_SKILLS.map((s) => (
            <div
              key={s.id}
              className="flex items-center gap-1.5 px-3 py-1 text-xs text-muted-foreground/50"
            >
              <Package className="size-3.5 shrink-0 text-blue-400/70" />
              <span className="flex-1 truncate">{s.name}</span>
              <span className="text-[9px] text-muted-foreground/30 bg-muted/50 px-1.5 rounded">
                {s.version}
              </span>
            </div>
          ))}
        </div>

        <div className="h-px bg-border/20 mx-3" />

        {/* Templates */}
        <div className="py-1.5">
          <div className="px-3.5 py-1">
            <span className="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/40">
              Templates
            </span>
          </div>
          <button
            onClick={onCreateSkill}
            className="flex w-full items-center gap-1.5 px-3 py-1 text-xs text-muted-foreground/40 hover:text-muted-foreground transition-colors"
          >
            <Plus className="size-3.5 shrink-0" />
            <span>New from template...</span>
          </button>
        </div>
      </div>
    </div>
  )
}
