import {
  Search, Plus, ChevronRight, ChevronDown, FileText, Zap,
  FolderOpen, Library,
} from 'lucide-react'
import type { Skill } from '@ship/ui'
import type { LibrarySkill } from './useSkillsLibrary'

interface Props {
  filteredSkills: Skill[]
  activeTabId: string | null
  expandedFolders: Set<string>
  searchQuery: string
  isConnected: boolean
  getLibrarySkill: (id: string) => LibrarySkill | undefined
  onSearchChange: (q: string) => void
  onToggleFolder: (id: string) => void
  onOpenSkill: (id: string) => void
  onCreateSkill: () => void
}


export function SkillsFileExplorer({
  filteredSkills,
  activeTabId,
  expandedFolders,
  searchQuery,
  isConnected,
  getLibrarySkill,
  onSearchChange,
  onToggleFolder,
  onOpenSkill,
  onCreateSkill,
}: Props) {
  const projectSkills = filteredSkills.filter((s) => {
    const ls = getLibrarySkill(s.id)
    return !ls || ls.origin === 'project'
  })
  const librarySkills = filteredSkills.filter((s) => {
    const ls = getLibrarySkill(s.id)
    return ls?.origin === 'library'
  })

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
        <SkillSection
          label="Project Skills"
          icon={<FolderOpen className="size-3 text-muted-foreground/40" />}
          skills={projectSkills}
          activeTabId={activeTabId}
          expandedFolders={expandedFolders}
          searchQuery={searchQuery}
          getLibrarySkill={getLibrarySkill}
          onToggleFolder={onToggleFolder}
          onOpenSkill={onOpenSkill}
          action={<button onClick={onCreateSkill} className="text-muted-foreground/40 hover:text-primary transition-colors" title="New skill"><Plus className="size-3.5" /></button>}
        />

        <div className="h-px bg-border/20 mx-3" />

        {/* Library Skills */}
        <SkillSection
          label="Library"
          icon={<Library className="size-3 text-muted-foreground/40" />}
          skills={librarySkills}
          activeTabId={activeTabId}
          expandedFolders={expandedFolders}
          searchQuery={searchQuery}
          getLibrarySkill={getLibrarySkill}
          onToggleFolder={onToggleFolder}
          onOpenSkill={onOpenSkill}
        />

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

        {/* Connection hint */}
        {!isConnected && filteredSkills.length === 0 && (
          <div className="px-3.5 py-4 text-center">
            <p className="text-[10px] text-muted-foreground/40 leading-relaxed">
              Connect to CLI to see your skills from ~/.ship
            </p>
          </div>
        )}
      </div>
    </div>
  )
}

function SkillSection({
  label,
  icon,
  skills,
  activeTabId,
  expandedFolders,
  searchQuery,
  getLibrarySkill,
  onToggleFolder,
  onOpenSkill,
  action,
}: {
  label: string
  icon: React.ReactNode
  skills: Skill[]
  activeTabId: string | null
  expandedFolders: Set<string>
  searchQuery: string
  getLibrarySkill: (id: string) => LibrarySkill | undefined
  onToggleFolder: (id: string) => void
  onOpenSkill: (id: string) => void
  action?: React.ReactNode
}) {
  return (
    <div className="py-1.5">
      <div className="flex items-center justify-between px-3.5 py-1">
        <span className="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/40">
          {icon}
          {label}
          <span className="text-muted-foreground/20 font-normal normal-case">
            ({skills.length})
          </span>
        </span>
        {action}
      </div>

      {skills.length === 0 && (
        <p className="px-4 py-3 text-[11px] italic text-muted-foreground/30">
          {searchQuery ? 'No matches.' : 'No skills yet.'}
        </p>
      )}

      {skills.map((skill) => {
        const expanded = expandedFolders.has(skill.id)
        const isActive = activeTabId === skill.id
        const ls = getLibrarySkill(skill.id)
        const originLabel = ls?.origin === 'library' ? 'library' : null

        return (
          <div key={skill.id}>
            <button
              onClick={() => onToggleFolder(skill.id)}
              className="flex w-full items-center gap-1.5 px-3 py-1 text-xs text-muted-foreground/60 hover:text-muted-foreground transition-colors"
            >
              {expanded
                ? <ChevronDown className="size-3 shrink-0" />
                : <ChevronRight className="size-3 shrink-0" />}
              <Zap className="size-3 shrink-0 text-primary/60" />
              <span className="truncate">{skill.name || skill.id}</span>
              {originLabel && (
                <span className="ml-auto text-[9px] text-violet-500/70 bg-violet-500/10 px-1.5 rounded">
                  {originLabel}
                </span>
              )}
            </button>

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
              </button>
            )}
          </div>
        )
      })}
    </div>
  )
}
