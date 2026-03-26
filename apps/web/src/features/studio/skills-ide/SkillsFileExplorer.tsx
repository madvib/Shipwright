import { useState } from 'react'
import {
  Search, Plus, ChevronRight, ChevronDown, FileText, Zap, File, Trash2,
} from 'lucide-react'
import type { Skill } from '@ship/ui'

interface Props {
  filteredSkills: Skill[]
  activeTabId: string | null
  expandedFolders: Set<string>
  searchQuery: string
  onSearchChange: (q: string) => void
  onToggleFolder: (id: string) => void
  onOpenFile: (skillId: string, filePath: string) => void
  onCreateSkill: () => void
  onAddFile: (skillId: string, filePath: string) => void
  onDeleteFile: (skillId: string, filePath: string) => void
  getFilesForSkill: (skillId: string) => string[]
}

function AddFileInline({ skillId, onAdd }: { skillId: string; onAdd: (skillId: string, filePath: string) => void }) {
  const [editing, setEditing] = useState(false)
  const [name, setName] = useState('')

  if (!editing) {
    return (
      <button
        onClick={() => setEditing(true)}
        className="flex w-full items-center gap-1.5 pl-9 pr-3 py-1 text-[11px] text-muted-foreground/50 hover:text-muted-foreground transition-colors"
      >
        <Plus className="size-3 shrink-0" />
        <span>Add file...</span>
      </button>
    )
  }

  const handleSubmit = () => {
    const trimmed = name.trim()
    if (trimmed) {
      onAdd(skillId, trimmed)
    }
    setEditing(false)
    setName('')
  }

  return (
    <div className="flex items-center gap-1 pl-9 pr-3 py-0.5">
      <input
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => { if (e.key === 'Enter') handleSubmit(); if (e.key === 'Escape') { setEditing(false); setName('') } }}
        onBlur={handleSubmit}
        placeholder="filename.ext"
        className="flex-1 min-w-0 bg-transparent text-[11px] text-foreground placeholder:text-muted-foreground/50 focus:outline-none border-b border-primary/50"
        autoFocus
        spellCheck={false}
      />
    </div>
  )
}

export function SkillsFileExplorer({
  filteredSkills,
  activeTabId,
  expandedFolders,
  searchQuery,
  onSearchChange,
  onToggleFolder,
  onOpenFile,
  onCreateSkill,
  onAddFile,
  onDeleteFile,
  getFilesForSkill,
}: Props) {
  return (
    <div className="flex w-60 shrink-0 flex-col border-r border-border/40 bg-card/30">
      {/* Search */}
      <div className="p-2.5 border-b border-border/30">
        <div className="flex items-center gap-2 rounded-md border border-border/50 bg-background/80 px-2.5 py-1.5 focus-within:border-primary/50 transition-colors">
          <Search className="size-3.5 text-muted-foreground/50 shrink-0" />
          <input
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder="Search skills..."
            className="min-w-0 flex-1 bg-transparent text-xs text-foreground placeholder:text-muted-foreground/50 focus:outline-none"
            spellCheck={false}
          />
        </div>
      </div>

      {/* Scrollable sections */}
      <div className="flex-1 overflow-y-auto">
        {/* Project Skills */}
        <div className="py-1.5">
          <div className="flex items-center justify-between px-3.5 py-1">
            <span className="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/50">
              Project Skills
            </span>
            <button
              onClick={onCreateSkill}
              className="text-muted-foreground/50 hover:text-primary transition-colors"
              title="New skill"
            >
              <Plus className="size-3.5" />
            </button>
          </div>

          {filteredSkills.length === 0 && (
            <div className="px-4 py-4 text-center">
              {searchQuery ? (
                <p className="text-[11px] italic text-muted-foreground/50">No matches.</p>
              ) : (
                <div className="space-y-2">
                  <p className="text-[11px] text-muted-foreground/60">No skills yet.</p>
                  <button
                    onClick={onCreateSkill}
                    className="inline-flex items-center gap-1.5 text-[11px] text-primary hover:text-primary/80 transition-colors"
                  >
                    <Plus className="size-3" />
                    Create your first skill
                  </button>
                </div>
              )}
            </div>
          )}

          {filteredSkills.map((skill) => {
            const expanded = expandedFolders.has(skill.id)
            const files = getFilesForSkill(skill.id)
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
                  {files.length > 1 && (
                    <span className="ml-auto text-[9px] text-muted-foreground/50">
                      {files.length}
                    </span>
                  )}
                </button>

                {/* File list */}
                {expanded && (
                  <>
                    {files.map((filePath) => {
                      const tabId = `${skill.id}::${filePath}`
                      const isActive = activeTabId === tabId
                      const isSkillMd = filePath === 'SKILL.md'
                      return (
                        <div key={filePath} className="group flex items-center">
                          <button
                            onClick={() => onOpenFile(skill.id, filePath)}
                            className={`flex flex-1 items-center gap-1.5 pl-9 pr-2 py-1 text-xs transition-colors border-l-2 min-w-0 ${
                              isActive
                                ? 'border-primary bg-primary/5 text-primary'
                                : 'border-transparent text-muted-foreground/60 hover:text-muted-foreground hover:bg-muted/30'
                            }`}
                          >
                            {isSkillMd
                              ? <FileText className="size-3 shrink-0" />
                              : <File className="size-3 shrink-0" />}
                            <span className="truncate">{filePath}</span>
                          </button>
                          {!isSkillMd && (
                            <button
                              onClick={() => onDeleteFile(skill.id, filePath)}
                              className="hidden group-hover:block pr-3 text-muted-foreground/50 hover:text-destructive transition-colors"
                              title="Delete file"
                            >
                              <Trash2 className="size-3" />
                            </button>
                          )}
                        </div>
                      )
                    })}

                    {/* Add file */}
                    <AddFileInline skillId={skill.id} onAdd={onAddFile} />

                    {/* Source badge */}
                    {skill.source && skill.source !== 'custom' && (
                      <div className="pl-9 pr-3 py-0.5">
                        <span className="text-[9px] text-muted-foreground/50 bg-muted/50 px-1.5 rounded">
                          {skill.source}
                        </span>
                      </div>
                    )}
                  </>
                )}
              </div>
            )
          })}
        </div>

        <div className="h-px bg-border/20 mx-3" />

        {/* Installed */}
        <div className="py-1.5">
          <div className="px-3.5 py-1">
            <span className="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground/50">
              Installed
            </span>
          </div>
          <p className="px-4 py-3 text-[11px] italic text-muted-foreground/50">
            No registry skills installed.
          </p>
        </div>

      </div>
    </div>
  )
}
