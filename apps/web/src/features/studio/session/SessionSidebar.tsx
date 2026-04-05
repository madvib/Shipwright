// Left sidebar for Session page: Files, Git, Sessions tabs.
// Files are grouped by namespace (first path segment = skill territory).

import { useState, useRef, useCallback, useMemo } from 'react'
import {
  FileText, CheckSquare, Image, ChevronDown, ChevronRight, Plus, Folder, MonitorPlay, ExternalLink,
} from 'lucide-react'
import { ArtifactContextMenu } from './ArtifactContextMenu'
import { GitTab } from './GitTab'
import { SessionsTab } from './SessionsTab'
import { useSkillsLibrary } from '#/features/studio/skills-ide/useSkillsLibrary'
import type { ArtifactMenuState } from './ArtifactContextMenu'
import type { SessionFile } from './types'
import type { GitStatusResult, GitLogEntry } from './useGitInfo'

// Parse `artifacts: [...]` from SKILL.md frontmatter
function parseArtifacts(content: string): string[] {
  const m = content.match(/^artifacts:\s*\[([^\]]+)\]/m)
  if (!m) return []
  return m[1].split(',').map((s) => s.trim().replace(/['"]/g, ''))
}

type SidebarTab = 'files' | 'git' | 'sessions'

const HIDDEN_FILES = new Set(['diff.txt', 'annotations.json'])

interface SessionSidebarProps {
  files: SessionFile[]
  activeFile: string | null
  onSelectFile: (path: string) => void
  onDeleteFile: (path: string) => void
  onUploadFiles: (files: FileList) => void
  onShowDiff: () => void
  onSelectCommit: (hash: string) => void
  gitStatus: GitStatusResult | null | undefined
  gitLog: GitLogEntry[] | null | undefined
}

// ── Namespace grouping — first path segment is the skill's claimed territory ──

interface SubGroup {
  name: string
  files: SessionFile[]
  latestAt: number
}

interface NamespaceGroup {
  name: string
  subGroups: SubGroup[]   // named sub-directories (e.g. timestamped runs)
  rootFiles: SessionFile[] // files directly in the namespace
  latestAt: number
}

function relativeTime(ms: number): string {
  const diff = Date.now() - ms
  if (diff < 60_000) return 'just now'
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`
  return `${Math.floor(diff / 86_400_000)}d ago`
}

function groupByNamespace(files: SessionFile[]): {
  todo: SessionFile | null
  groups: NamespaceGroup[]
  root: SessionFile[]
} {
  const todo = files.find((f) => /^todo\.md$/i.test(f.name)) ?? null
  const visible = files.filter((f) => !HIDDEN_FILES.has(f.name) && !/^todo\.md$/i.test(f.name))

  const byNs = new Map<string, SessionFile[]>()
  const root: SessionFile[] = []

  for (const f of visible) {
    const slash = f.path.indexOf('/')
    if (slash === -1) {
      root.push(f)
    } else {
      const ns = f.path.slice(0, slash)
      if (!byNs.has(ns)) byNs.set(ns, [])
      byNs.get(ns)!.push(f)
    }
  }

  const groups: NamespaceGroup[] = Array.from(byNs.entries())
    .map(([name, nsFiles]) => {
      const bySub = new Map<string, SessionFile[]>()
      const rootFiles: SessionFile[] = []
      for (const f of nsFiles) {
        const rel = f.path.slice(name.length + 1)
        const nextSlash = rel.indexOf('/')
        if (nextSlash === -1) {
          rootFiles.push(f)
        } else {
          const sub = rel.slice(0, nextSlash)
          if (!bySub.has(sub)) bySub.set(sub, [])
          bySub.get(sub)!.push(f)
        }
      }
      const subGroups: SubGroup[] = Array.from(bySub.entries())
        .map(([sub, subFiles]) => ({
          name: sub,
          files: subFiles.sort((a, b) => b.modifiedAt - a.modifiedAt),
          latestAt: Math.max(...subFiles.map((f) => f.modifiedAt)),
        }))
        .sort((a, b) => b.latestAt - a.latestAt)
      return {
        name,
        subGroups,
        rootFiles: rootFiles.sort((a, b) => b.modifiedAt - a.modifiedAt),
        latestAt: Math.max(...nsFiles.map((f) => f.modifiedAt)),
      }
    })
    .sort((a, b) => b.latestAt - a.latestAt)

  return { todo, groups, root: root.sort((a, b) => b.modifiedAt - a.modifiedAt) }
}

// ── File type icons ──

const FILE_ICONS: Record<SessionFile['type'], { icon: typeof FileText; color: string }> = {
  html: { icon: FileText, color: 'text-sky-500' },
  markdown: { icon: FileText, color: 'text-emerald-500' },
  image: { icon: Image, color: 'text-amber-500' },
  url: { icon: MonitorPlay, color: 'text-indigo-500' },
  other: { icon: FileText, color: 'text-muted-foreground' },
}

export function SessionSidebar({
  files, activeFile,
  onSelectFile, onDeleteFile, onUploadFiles,
  onShowDiff, onSelectCommit,
  gitStatus, gitLog,
}: SessionSidebarProps) {
  const [tab, setTab] = useState<SidebarTab>('files')
  const [collapsedGroups, setCollapsedGroups] = useState<Set<string>>(new Set())
  const [contextMenu, setContextMenu] = useState<ArtifactMenuState | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const toggleGroup = (label: string) => {
    setCollapsedGroups((prev) => {
      const next = new Set(prev)
      next.has(label) ? next.delete(label) : next.add(label)
      return next
    })
  }

  const handleContextMenu = useCallback((e: React.MouseEvent, file: SessionFile) => {
    e.preventDefault()
    setContextMenu({ x: e.clientX, y: e.clientY, file })
  }, [])

  const { todo, groups, root } = groupByNamespace(files)

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-border bg-card/30">
      {/* Tab bar — h-9 matches SessionTabBar height */}
      <div className="flex items-center border-b border-border shrink-0 h-9">
        {(['files', 'git', 'sessions'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`flex-1 h-full text-center text-[11px] font-medium border-b-2 transition-colors capitalize ${
              tab === t ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto">
        {tab === 'files' && (
          <FilesTab
            todo={todo}
            groups={groups}
            root={root}
            activeFile={activeFile}
            collapsedGroups={collapsedGroups}
            fileInputRef={fileInputRef}
            onSelectFile={onSelectFile}
            onUploadFiles={onUploadFiles}
            onToggleGroup={toggleGroup}
            onContextMenu={handleContextMenu}
          />
        )}
        {tab === 'git' && (
          <GitTab gitStatus={gitStatus} gitLog={gitLog} onShowDiff={onShowDiff} onSelectCommit={onSelectCommit} />
        )}
        {tab === 'sessions' && (
          <SessionsTab />
        )}
      </div>

      {contextMenu && (
        <ArtifactContextMenu menu={contextMenu} onClose={() => setContextMenu(null)} onDelete={onDeleteFile} />
      )}
    </aside>
  )
}

// ── Files Tab ──

function FilesTab({ todo, groups, root, activeFile, collapsedGroups, fileInputRef, onSelectFile, onUploadFiles, onToggleGroup, onContextMenu }: {
  todo: SessionFile | null
  groups: NamespaceGroup[]
  root: SessionFile[]
  activeFile: string | null
  collapsedGroups: Set<string>
  fileInputRef: React.RefObject<HTMLInputElement | null>
  onSelectFile: (path: string) => void
  onUploadFiles: (files: FileList) => void
  onToggleGroup: (label: string) => void
  onContextMenu: (e: React.MouseEvent, file: SessionFile) => void
}) {
  const { skills } = useSkillsLibrary()

  // Map stable-id → skill for namespace header enrichment
  const skillByNs = useMemo(() => {
    const map = new Map<string, { description: string | null; artifacts: string[] }>()
    for (const s of skills) {
      if (s.stableId) map.set(s.stableId, { description: s.description ?? null, artifacts: parseArtifacts(s.content) })
    }
    return map
  }, [skills])

  return (
    <div className="py-1.5">
      {todo && (
        <div className="px-3 mb-1">
          <FileEntry file={todo} isActive={activeFile === todo.path} onClick={() => onSelectFile(todo.path)} onContextMenu={(e) => onContextMenu(e, todo)} isTodo />
        </div>
      )}

      {/* Skill tiles — full width, divider-separated */}
      {groups.map((group) => {
        const nsCollapsed = collapsedGroups.has(group.name)
        const meta = skillByNs.get(group.name)
        const isUrlSkill = meta?.artifacts.includes('url') ?? false
        const urlFile = group.rootFiles.find((f) => f.type === 'url')
        const isRunning = isUrlSkill && !!urlFile
        return (
          <div key={group.name} className="border-b border-border/15 last:border-0">
            {/* Tile header */}
            <div
              role="button" tabIndex={0}
              onClick={() => onToggleGroup(group.name)}
              onKeyDown={(e) => e.key === 'Enter' && onToggleGroup(group.name)}
              className="px-2.5 py-2 cursor-pointer hover:bg-white/[0.02] transition-colors select-none"
            >
              <div className="flex items-center gap-1.5 mb-0.5">
                {isUrlSkill && (
                  <div className={`size-[5px] rounded-full shrink-0 ${isRunning ? 'bg-emerald-500' : 'bg-muted-foreground/20'}`} />
                )}
                <span className="text-[11px] font-semibold text-foreground/80 flex-1 truncate">{group.name}</span>
                {nsCollapsed ? <ChevronRight className="size-[11px] text-muted-foreground/25 shrink-0" /> : <ChevronDown className="size-[11px] text-muted-foreground/25 shrink-0" />}
              </div>
              {meta?.description && (
                <p className="text-[9.5px] text-muted-foreground/60 leading-snug truncate mb-1">{meta.description}</p>
              )}
              <div className="flex items-center gap-1">
                {meta?.artifacts.map((a) => (
                  <span key={a} className="text-[8.5px] font-medium px-1 py-0.5 rounded-sm bg-white/[0.06] text-white/[0.38] capitalize">{a}</span>
                ))}
                <span className="flex-1" />
                <span className="text-[9px] text-muted-foreground/30 tabular-nums">{relativeTime(group.latestAt)}</span>
                {isRunning && urlFile && (
                  <button
                    onClick={(e) => { e.stopPropagation(); onSelectFile(urlFile.path) }}
                    className="ml-1.5 flex items-center gap-0.5 text-[9.5px] font-medium text-primary/65 hover:text-primary transition-colors"
                  >
                    Open <ExternalLink className="size-[8px]" />
                  </button>
                )}
              </div>
            </div>
            {/* Expanded file tree */}
            {!nsCollapsed && (
              <div className="bg-black/[0.06] border-t border-border/10 px-2.5 py-1.5 space-y-0.5">
                {group.rootFiles.map((f) => (
                  <FileEntry key={f.path} file={f} isActive={activeFile === f.path} onClick={() => onSelectFile(f.path)} onContextMenu={(e) => onContextMenu(e, f)} stripPrefix={group.name} />
                ))}
                {group.subGroups.map((sub) => {
                  const subKey = `${group.name}/${sub.name}`
                  const subCollapsed = collapsedGroups.has(subKey)
                  return (
                    <div key={subKey}>
                      <button onClick={() => onToggleGroup(subKey)} className="flex items-center gap-1.5 w-full px-1 py-1">
                        <Folder className="size-3 text-muted-foreground/50 shrink-0" />
                        <span className="text-[11px] text-muted-foreground/70 truncate flex-1">{sub.name}</span>
                        <span className="text-[9px] text-muted-foreground/30 tabular-nums">{relativeTime(sub.latestAt)}</span>
                        {subCollapsed ? <ChevronRight className="size-2.5 text-muted-foreground/30 shrink-0 ml-1" /> : <ChevronDown className="size-2.5 text-muted-foreground/30 shrink-0 ml-1" />}
                      </button>
                      {!subCollapsed && (
                        <div className="pl-2.5 border-l border-border/30 ml-1.5 space-y-0.5">
                          {sub.files.map((f) => (
                            <FileEntry key={f.path} file={f} isActive={activeFile === f.path} onClick={() => onSelectFile(f.path)} onContextMenu={(e) => onContextMenu(e, f)} stripPrefix={subKey} />
                          ))}
                        </div>
                      )}
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        )
      })}

      {root.length > 0 && (
        <div className="px-3 space-y-0.5 mt-1 pt-1 border-t border-border/15">
          {root.map((f) => (
            <FileEntry key={f.path} file={f} isActive={activeFile === f.path} onClick={() => onSelectFile(f.path)} onContextMenu={(e) => onContextMenu(e, f)} />
          ))}
        </div>
      )}

      <div className="px-3">
        <button
          onClick={() => fileInputRef.current?.click()}
          className="flex items-center gap-1.5 w-full mt-2 px-2 py-1.5 rounded-md text-xs text-muted-foreground hover:text-foreground hover:bg-muted/30 transition"
        >
          <Plus className="size-3.5" />
          <span>Add files</span>
        </button>
        <input ref={fileInputRef} type="file" multiple className="hidden" onChange={(e) => { if (e.target.files?.length) onUploadFiles(e.target.files); e.target.value = '' }} />
      </div>
    </div>
  )
}

// ── Helpers ──

function FileEntry({ file, isActive, onClick, onContextMenu, isTodo, stripPrefix }: {
  file: SessionFile; isActive: boolean; onClick: () => void
  onContextMenu: (e: React.MouseEvent) => void; isTodo?: boolean; stripPrefix?: string
}) {
  const { icon: Icon, color } = FILE_ICONS[file.type] ?? FILE_ICONS.other
  // Strip the known prefix (namespace or namespace/subgroup) so only the relevant tail shows
  const displayPath = stripPrefix ? file.path.slice(stripPrefix.length + 1) : file.path
  const lastSlash = displayPath.lastIndexOf('/')
  const subdir = lastSlash > 0 ? displayPath.slice(0, lastSlash + 1) : null
  const filename = lastSlash > 0 ? displayPath.slice(lastSlash + 1) : displayPath
  return (
    <button
      onClick={onClick}
      onContextMenu={onContextMenu}
      className={`flex items-center gap-2 w-full px-2 py-1.5 rounded-md text-xs transition ${
        isActive ? 'border-l-2 border-primary bg-primary/5 text-primary font-medium' : 'text-muted-foreground hover:text-foreground hover:bg-muted/30'
      }`}
    >
      {isTodo
        ? <CheckSquare className={`size-3.5 shrink-0 ${isActive ? 'text-primary' : 'text-emerald-500'}`} />
        : <Icon className={`size-3.5 shrink-0 ${isActive ? 'text-primary' : color}`} />
      }
      <span className="truncate text-left min-w-0">
        {subdir && <span className="opacity-40">{subdir}</span>}
        {filename}
      </span>
    </button>
  )
}
