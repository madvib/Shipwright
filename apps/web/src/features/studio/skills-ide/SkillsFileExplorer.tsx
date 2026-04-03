import { useState, useMemo, useCallback } from 'react'
import {
  Search, Plus, ChevronRight, ChevronDown, FileText, Zap,
  FolderOpen, Library, FileJson, BookOpen, FlaskConical, Settings2, ChevronsDownUp, Terminal,
} from 'lucide-react'
import type { Skill } from '@ship/ui'
import type { LibrarySkill } from './useSkillsLibrary'
import { makeTabId, SKILL_MD } from './useSkillsIDE'
import { SkillContextMenu, type ContextMenuState } from './SkillContextMenu'
import { SkillFilterBar, applyFilters, type SkillFilter } from './SkillFilters'

interface Props {
  filteredSkills: Skill[]
  activeTabId: string | null
  expandedFolders: Set<string>
  searchQuery: string
  isLoading: boolean
  getLibrarySkill: (id: string) => LibrarySkill | undefined
  onSearchChange: (q: string) => void
  onToggleFolder: (id: string) => void
  onCollapseAll: () => void
  onOpenFile: (skillId: string, filePath: string) => void
  onAddFile: (skillId: string, filePath: string, content: string) => void
  onDeleteFile: (skillId: string, filePath: string) => void
  onCreateSkill: () => void
}

function fileIcon(path: string) {
  if (path === SKILL_MD) return <FileText className="size-3 shrink-0 text-sky-400" />
  if (path.endsWith('.json')) return <FileJson className="size-3 shrink-0 text-amber-400" />
  if (path.endsWith('.md')) return <BookOpen className="size-3 shrink-0 text-emerald-400" />
  return <FileText className="size-3 shrink-0 text-muted-foreground" />
}

type FileGroupName = 'root' | 'assets' | 'references' | 'evals' | 'scripts'
function fileGroup(path: string): FileGroupName {
  if (path.startsWith('assets/')) return 'assets'
  if (path.startsWith('references/')) return 'references'
  if (path.startsWith('evals/')) return 'evals'
  if (path.startsWith('scripts/')) return 'scripts'
  return 'root'
}

const GROUP_META: Record<string, { label: string; icon: React.ReactNode }> = {
  assets: { label: 'assets', icon: <Settings2 className="size-2.5 text-muted-foreground" /> },
  references: { label: 'references', icon: <BookOpen className="size-2.5 text-muted-foreground" /> },
  evals: { label: 'evals', icon: <FlaskConical className="size-2.5 text-muted-foreground" /> },
  scripts: { label: 'scripts', icon: <Terminal className="size-2.5 text-muted-foreground" /> },
}

export function SkillsFileExplorer({
  filteredSkills, activeTabId, expandedFolders, searchQuery,
  isLoading, getLibrarySkill,
  onSearchChange, onToggleFolder, onCollapseAll, onOpenFile, onAddFile, onDeleteFile, onCreateSkill,
}: Props) {
  const [activeFilter, setActiveFilter] = useState<SkillFilter>('all')
  const [activeTags, setActiveTags] = useState<Set<string>>(new Set())
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null)

  const handleTagToggle = useCallback((tag: string) => {
    setActiveTags((prev) => {
      const next = new Set(prev)
      if (next.has(tag)) next.delete(tag); else next.add(tag)
      return next
    })
  }, [])

  const allLibrarySkills = useMemo(
    () => filteredSkills.map((s) => getLibrarySkill(s.id)).filter(Boolean) as LibrarySkill[],
    [filteredSkills, getLibrarySkill],
  )
  const finalSkills = useMemo(
    () => applyFilters(allLibrarySkills, activeFilter, activeTags),
    [allLibrarySkills, activeFilter, activeTags],
  )

  const projectSkills = finalSkills.filter((s) => s.origin !== 'library')
  const librarySkillsList = finalSkills.filter((s) => s.origin === 'library')

  return (
    <div className="flex w-60 shrink-0 flex-col border-r border-border bg-card/30">
      <div className="p-2.5 border-b border-border space-y-1.5">
        <div className="flex items-center gap-2 rounded-md border border-border bg-background/80 px-2.5 py-1.5 focus-within:border-primary/50 transition-colors">
          <Search className="size-3.5 text-muted-foreground shrink-0" />
          <input
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search skills..."
            className="min-w-0 flex-1 bg-transparent text-xs text-foreground placeholder:text-muted-foreground focus:outline-none"
            spellCheck={false}
          />
          {expandedFolders.size > 0 && (
            <button onClick={onCollapseAll} className="text-muted-foreground hover:text-foreground transition-colors" title="Collapse all">
              <ChevronsDownUp className="size-3.5" />
            </button>
          )}
        </div>
        <SkillFilterBar
          allSkills={allLibrarySkills}
          activeFilter={activeFilter}
          activeTags={activeTags}
          onFilterChange={setActiveFilter}
          onTagToggle={handleTagToggle}
        />
      </div>

      <div className="flex-1 overflow-y-auto">
        {isLoading && filteredSkills.length === 0 ? (
          <div className="p-3 space-y-2">
            {Array.from({ length: 5 }).map((_, i) => (
              <div key={i} className="h-6 w-full animate-pulse rounded bg-muted" />
            ))}
          </div>
        ) : (
          <>
            <SkillSection
              label="Project Skills"
              icon={<FolderOpen className="size-3 text-muted-foreground" />}
              skills={projectSkills}
              activeTabId={activeTabId}
              expandedFolders={expandedFolders}
              searchQuery={searchQuery}
              getLibrarySkill={getLibrarySkill}
              onToggleFolder={onToggleFolder}
              onOpenFile={onOpenFile}
              onContextMenu={setContextMenu}
              action={<button onClick={onCreateSkill} className="text-muted-foreground hover:text-primary transition-colors" title="New skill"><Plus className="size-3.5" /></button>}
            />

            <div className="h-px bg-border mx-3" />

            <SkillSection
              label="Library"
              icon={<Library className="size-3 text-muted-foreground" />}
              skills={librarySkillsList}
              activeTabId={activeTabId}
              expandedFolders={expandedFolders}
              searchQuery={searchQuery}
              getLibrarySkill={getLibrarySkill}
              onToggleFolder={onToggleFolder}
              onOpenFile={onOpenFile}
              onContextMenu={setContextMenu}
            />

          </>
        )}
      </div>

      {contextMenu && (
        <SkillContextMenu
          menu={contextMenu}
          onAddFile={onAddFile}
          onDeleteFile={onDeleteFile}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  )
}

interface SectionProps {
  label: string; icon: React.ReactNode; skills: Skill[]; activeTabId: string | null
  expandedFolders: Set<string>; searchQuery: string; action?: React.ReactNode
  getLibrarySkill: (id: string) => LibrarySkill | undefined
  onToggleFolder: (id: string) => void
  onOpenFile: (skillId: string, filePath: string) => void
  onContextMenu: (state: ContextMenuState) => void
}

function SkillSection({
  label, icon, skills, activeTabId, expandedFolders, searchQuery,
  getLibrarySkill, onToggleFolder, onOpenFile, onContextMenu, action,
}: SectionProps) {
  return (
    <div className="py-1.5">
      <div className="flex items-center justify-between px-3.5 py-1">
        <span className="flex items-center gap-1.5 text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground">
          {icon}
          {label}
          <span className="text-muted-foreground font-normal normal-case">
            ({skills.length})
          </span>
        </span>
        {action}
      </div>

      {skills.length === 0 && (
        <p className="px-4 py-3 text-[11px] italic text-muted-foreground">
          {searchQuery ? 'No matches.' : 'No skills yet.'}
        </p>
      )}

      {skills.map((skill) => {
        const expanded = expandedFolders.has(skill.id)
        const ls = getLibrarySkill(skill.id)
        const originLabel = ls?.origin === 'library' ? 'library' : null
        const files = ls?.files ?? [SKILL_MD]
        const hasVars = ls?.varsSchema != null
        const hasEvals = ls?.evals != null
        const hasRefs = Object.keys(ls?.referenceDocs ?? {}).length > 0

        const rootFiles = files.filter((f) => fileGroup(f) === 'root')
        const groups: FileGroupName[] = ['assets', 'references', 'evals', 'scripts']

        return (
          <div key={skill.id}>
            <button
              onClick={() => onToggleFolder(skill.id)}
              onContextMenu={(e) => {
                if (!ls) return
                e.preventDefault()
                onContextMenu({ mode: 'folder', skill: ls, x: e.clientX, y: e.clientY })
              }}
              className="flex w-full items-center gap-1.5 px-3 py-1 text-xs text-foreground/70 hover:text-foreground transition-colors"
            >
              {expanded ? <ChevronDown className="size-3 shrink-0" /> : <ChevronRight className="size-3 shrink-0" />}
              <Zap className="size-3 shrink-0 text-primary/80" />
              <span className="truncate font-medium">{skill.name || skill.id}</span>
              <span className="ml-auto flex items-center gap-1">
                {hasVars && <span className="size-1.5 rounded-full bg-amber-400" title="Has variables" />}
                {hasRefs && <span className="size-1.5 rounded-full bg-emerald-400" title="Has docs" />}
                {hasEvals && <span className="size-1.5 rounded-full bg-violet-400" title="Has evals" />}
              </span>
              {originLabel && (
                <span className="text-[9px] text-violet-400 bg-violet-500/15 px-1.5 rounded">{originLabel}</span>
              )}
            </button>
            {expanded && ls && (
              <div className="pb-1">
                {rootFiles.map((f) => (
                  <FileEntry key={f} skill={ls} skillId={skill.id} filePath={f} activeTabId={activeTabId} indent={0} onOpenFile={onOpenFile} onContextMenu={onContextMenu} />
                ))}
                {groups.map((g) => {
                  const gFiles = files.filter((f) => fileGroup(f) === g)
                  return gFiles.length > 0 ? <FileGroup key={g} group={g} files={gFiles} skill={ls} skillId={skill.id} activeTabId={activeTabId} onOpenFile={onOpenFile} onContextMenu={onContextMenu} /> : null
                })}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}

function FileGroup({ group, files, skill, skillId, activeTabId, onOpenFile, onContextMenu }: {
  group: string; files: string[]; skill: LibrarySkill; skillId: string; activeTabId: string | null
  onOpenFile: (sid: string, fp: string) => void
  onContextMenu: (state: ContextMenuState) => void
}) {
  const meta = GROUP_META[group]
  if (!meta) return null
  return (
    <div>
      <div className="flex items-center gap-1.5 pl-9 pr-3 py-0.5 text-[10px] text-muted-foreground font-medium">
        {meta.icon}{meta.label}/
      </div>
      {files.map((f) => <FileEntry key={f} skill={skill} skillId={skillId} filePath={f} activeTabId={activeTabId} indent={1} onOpenFile={onOpenFile} onContextMenu={onContextMenu} />)}
    </div>
  )
}

function FileEntry({ skill, skillId, filePath, activeTabId, indent, onOpenFile, onContextMenu }: {
  skill: LibrarySkill; skillId: string; filePath: string; activeTabId: string | null; indent: number
  onOpenFile: (sid: string, fp: string) => void
  onContextMenu: (state: ContextMenuState) => void
}) {
  const tabId = makeTabId(skillId, filePath)
  const isActive = activeTabId === tabId
  const pl = indent === 0 ? 'pl-9' : 'pl-12'
  return (
    <button
      onClick={() => onOpenFile(skillId, filePath)}
      onContextMenu={(e) => {
        e.preventDefault()
        onContextMenu({ mode: 'file', skill, filePath, x: e.clientX, y: e.clientY })
      }}
      className={`flex w-full items-center gap-1.5 ${pl} pr-3 py-1 text-xs transition-colors border-l-2 ${isActive ? 'border-primary bg-primary/5 text-primary' : 'border-transparent text-foreground/70 hover:text-foreground hover:bg-muted/30'}`}
    >
      {fileIcon(filePath)}
      <span className="truncate">{filePath.split('/').pop() ?? filePath}</span>
    </button>
  )
}
