// Left sidebar for Session page: Files, Git, Sessions tabs.
// Files are grouped by purpose (canvas, specs, screenshots, etc).

import { useState, useRef, useCallback } from 'react'
import {
  FileText, CheckSquare, Image, ChevronDown, ChevronRight, Plus, Layers, FileCode,
} from 'lucide-react'
import { CliStatusPopover } from '#/features/studio/CliStatusPopover'
import { ArtifactContextMenu } from './ArtifactContextMenu'
import { GitTab } from './GitTab'
import { SessionsTab } from './SessionsTab'
import { StagedAnnotationsPanel } from './StagedAnnotationsPanel'
import type { ArtifactMenuState } from './ArtifactContextMenu'
import type { SessionFile, StagedAnnotation } from './types'
import type { GitStatusResult, GitLogEntry } from './useGitInfo'

type SidebarTab = 'files' | 'git' | 'sessions'

const HIDDEN_FILES = new Set(['diff.txt', 'annotations.json'])

interface SessionSidebarProps {
  files: SessionFile[]
  activeFile: string | null
  stagedAnnotations: StagedAnnotation[]
  isConnected: boolean
  onSelectFile: (path: string) => void
  onDeleteFile: (path: string) => void
  onUploadFiles: (files: FileList) => void
  onNavigateToAnnotation: (filePath: string, annotationId: string) => void
  onDeleteAnnotation: (annotationId: string) => void
  onClearAnnotations: () => void
  onShowDiff: () => void
  onSelectCommit: (hash: string) => void
  gitStatus: GitStatusResult | null | undefined
  gitLog: GitLogEntry[] | null | undefined
}

// ── Smart file grouping by purpose ──

interface FileGroup {
  label: string
  icon: typeof FileText
  iconColor: string
  files: SessionFile[]
}

function categorizeFiles(files: SessionFile[]): { todo: SessionFile | null; groups: FileGroup[] } {
  const todo = files.find((f) => /^todo\.md$/i.test(f.name)) ?? null
  const visible = files.filter((f) => !HIDDEN_FILES.has(f.name) && !/^todo\.md$/i.test(f.name))

  const canvas: SessionFile[] = []
  const specs: SessionFile[] = []
  const screenshots: SessionFile[] = []
  const other: SessionFile[] = []

  for (const f of visible) {
    const name = f.name.toLowerCase()
    const path = f.path.toLowerCase()

    if (name.startsWith('canvas') || name === 'mockup.html' || path.includes('mockup')) {
      canvas.push(f)
    } else if (name.includes('spec') || name.includes('plan') || name.includes('vision') || name.includes('checklist') || name.includes('critique') || name.includes('handoff') || name.startsWith('job-spec')) {
      specs.push(f)
    } else if (f.type === 'image' || path.includes('screenshot')) {
      screenshots.push(f)
    } else {
      other.push(f)
    }
  }

  const groups: FileGroup[] = []
  if (canvas.length > 0) groups.push({ label: 'Canvas', icon: Layers, iconColor: 'text-sky-500', files: canvas })
  if (specs.length > 0) groups.push({ label: 'Specs & Plans', icon: FileCode, iconColor: 'text-violet-500', files: specs })
  if (screenshots.length > 0) groups.push({ label: 'Screenshots', icon: Image, iconColor: 'text-amber-500', files: screenshots })
  if (other.length > 0) groups.push({ label: 'Other', icon: FileText, iconColor: 'text-muted-foreground', files: other })

  return { todo, groups }
}

// ── File type icons ──

const FILE_ICONS: Record<SessionFile['type'], { icon: typeof FileText; color: string }> = {
  html: { icon: FileText, color: 'text-sky-500' },
  markdown: { icon: FileText, color: 'text-emerald-500' },
  image: { icon: Image, color: 'text-amber-500' },
  other: { icon: FileText, color: 'text-muted-foreground' },
}

export function SessionSidebar({
  files, activeFile, stagedAnnotations, isConnected,
  onSelectFile, onDeleteFile, onUploadFiles,
  onNavigateToAnnotation, onDeleteAnnotation, onClearAnnotations,
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

  const { todo, groups } = categorizeFiles(files)

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
            activeFile={activeFile}
            stagedAnnotations={stagedAnnotations}
            collapsedGroups={collapsedGroups}
            isConnected={isConnected}
            fileInputRef={fileInputRef}
            onSelectFile={onSelectFile}
            onUploadFiles={onUploadFiles}
            onToggleGroup={toggleGroup}
            onContextMenu={handleContextMenu}
            onNavigateToAnnotation={onNavigateToAnnotation}
            onDeleteAnnotation={onDeleteAnnotation}
            onClearAnnotations={onClearAnnotations}
          />
        )}
        {tab === 'git' && (
          <GitTab gitStatus={gitStatus} gitLog={gitLog} onShowDiff={onShowDiff} onSelectCommit={onSelectCommit} />
        )}
        {tab === 'sessions' && (
          <SessionsTab isConnected={isConnected} gitStatus={gitStatus} gitLog={gitLog} />
        )}
      </div>

      {/* Footer: CLI connection */}
      <div className="shrink-0 border-t border-border px-2 py-1.5">
        <CliStatusPopover onAddSkill={() => {}} />
      </div>

      {contextMenu && (
        <ArtifactContextMenu menu={contextMenu} onClose={() => setContextMenu(null)} onDelete={onDeleteFile} />
      )}
    </aside>
  )
}

// ── Files Tab ──

function FilesTab({ todo, groups, activeFile, stagedAnnotations, collapsedGroups, isConnected, fileInputRef, onSelectFile, onUploadFiles, onToggleGroup, onContextMenu, onNavigateToAnnotation, onDeleteAnnotation, onClearAnnotations }: {
  todo: SessionFile | null
  groups: FileGroup[]
  activeFile: string | null
  stagedAnnotations: StagedAnnotation[]
  collapsedGroups: Set<string>
  isConnected: boolean
  fileInputRef: React.RefObject<HTMLInputElement | null>
  onSelectFile: (path: string) => void
  onUploadFiles: (files: FileList) => void
  onToggleGroup: (label: string) => void
  onContextMenu: (e: React.MouseEvent, file: SessionFile) => void
  onNavigateToAnnotation: (filePath: string, annotationId: string) => void
  onDeleteAnnotation: (annotationId: string) => void
  onClearAnnotations: () => void
}) {
  return (
    <div className="px-3 pt-3 pb-2">
      {todo && (
        <div className="mb-3">
          <FileEntry file={todo} isActive={activeFile === todo.path} onClick={() => onSelectFile(todo.path)} onContextMenu={(e) => onContextMenu(e, todo)} isTodo />
        </div>
      )}
      {groups.map((group) => {
        const collapsed = collapsedGroups.has(group.label)
        const GroupIcon = group.icon
        return (
          <div key={group.label} className="mb-2">
            <button onClick={() => onToggleGroup(group.label)} className="flex items-center gap-1.5 w-full px-0 py-2">
              <GroupIcon className={`size-3.5 ${group.iconColor} shrink-0`} />
              <span className="text-xs font-semibold text-muted-foreground">{group.label}</span>
              <span className="text-[10px] text-muted-foreground/50">{group.files.length}</span>
              <div className="flex-1" />
              {collapsed ? <ChevronRight className="size-3 text-muted-foreground/40 shrink-0" /> : <ChevronDown className="size-3 text-muted-foreground/40 shrink-0" />}
            </button>
            {!collapsed && (
              <div className="space-y-0.5">
                {group.files.map((f) => (
                  <FileEntry key={f.path} file={f} isActive={activeFile === f.path} onClick={() => onSelectFile(f.path)} onContextMenu={(e) => onContextMenu(e, f)} />
                ))}
              </div>
            )}
          </div>
        )
      })}
      <button
        onClick={() => fileInputRef.current?.click()}
        disabled={!isConnected}
        className="flex items-center gap-1.5 w-full mt-2 px-2 py-1.5 rounded-md text-xs text-muted-foreground hover:text-foreground hover:bg-muted/30 transition disabled:opacity-40 disabled:cursor-not-allowed"
      >
        <Plus className="size-3.5" />
        <span>Add files</span>
      </button>
      <input ref={fileInputRef} type="file" multiple className="hidden" onChange={(e) => { if (e.target.files?.length) onUploadFiles(e.target.files); e.target.value = '' }} />
      {stagedAnnotations.length > 0 && (
        <StagedAnnotationsPanel
          staged={stagedAnnotations}
          onNavigate={onNavigateToAnnotation}
          onDelete={onDeleteAnnotation}
          onClearAll={onClearAnnotations}
        />
      )}
    </div>
  )
}

// ── Helpers ──

function SectionHeader({ label, count, open, onToggle }: {
  label: string; count?: number; open: boolean; onToggle: () => void
}) {
  return (
    <button onClick={onToggle} className="flex items-center gap-1 w-full">
      {open ? <ChevronDown className="size-3 text-muted-foreground/40 shrink-0" /> : <ChevronRight className="size-3 text-muted-foreground/40 shrink-0" />}
      <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60">{label}</span>
      {count != null && <span className="text-[9px] text-muted-foreground/40 bg-muted/50 px-1.5 py-0.5 rounded">{count}</span>}
    </button>
  )
}

function FileEntry({ file, isActive, onClick, onContextMenu, isTodo }: {
  file: SessionFile; isActive: boolean; onClick: () => void
  onContextMenu: (e: React.MouseEvent) => void; isTodo?: boolean
}) {
  const { icon: Icon, color } = FILE_ICONS[file.type]
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
      <span className="truncate text-left">{file.name}</span>
    </button>
  )
}
