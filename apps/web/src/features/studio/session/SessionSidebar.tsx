// Left sidebar for Session page: Files, Git, Sessions tabs.
// Mirrors the Skills IDE SkillsFileExplorer pattern (w-60, border-r, file tree).

import { useState, useRef, useCallback } from 'react'
import {
  FileText, CheckSquare, Image, ChevronDown, ChevronRight,
  Plus, MapPin, GitCompareArrows, Circle,
} from 'lucide-react'
import { CliStatusPopover } from '#/features/studio/CliStatusPopover'
import { ArtifactContextMenu } from './ArtifactContextMenu'
import type { ArtifactMenuState } from './ArtifactContextMenu'
import type { SessionFile, Annotation } from './types'
import type { GitStatusResult, GitLogEntry } from './useGitInfo'

type SidebarTab = 'files' | 'git' | 'sessions'

// Files that are internal state — hide from the artifact tree
const HIDDEN_FILES = new Set(['diff.txt', 'annotations.json'])

function isTodoFile(name: string): boolean {
  return /^todo\.md$/i.test(name)
}

interface SessionSidebarProps {
  files: SessionFile[]
  activeFile: string | null
  annotations: Annotation[]
  isConnected: boolean
  onSelectFile: (path: string) => void
  onUploadFiles: (files: FileList) => void
  onShowDiff: () => void
  onSelectCommit: (hash: string) => void
  gitStatus: GitStatusResult | null | undefined
  gitLog: GitLogEntry[] | null | undefined
}

const FILE_ICONS: Record<SessionFile['type'], { icon: typeof FileText; color: string }> = {
  html: { icon: FileText, color: 'text-sky-500' },
  markdown: { icon: FileText, color: 'text-emerald-500' },
  image: { icon: Image, color: 'text-amber-500' },
  other: { icon: FileText, color: 'text-muted-foreground' },
}

export function SessionSidebar({
  files, activeFile, annotations, isConnected,
  onSelectFile, onUploadFiles, onShowDiff, onSelectCommit,
  gitStatus, gitLog,
}: SessionSidebarProps) {
  const [tab, setTab] = useState<SidebarTab>('files')
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(new Set())
  const [todoOpen, setTodoOpen] = useState(true)
  const [artifactsOpen, setArtifactsOpen] = useState(true)
  const [annotationsOpen, setAnnotationsOpen] = useState(false)
  const [contextMenu, setContextMenu] = useState<ArtifactMenuState | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  const toggleFolder = (folder: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev)
      next.has(folder) ? next.delete(folder) : next.add(folder)
      return next
    })
  }

  const handleContextMenu = useCallback((e: React.MouseEvent, file: SessionFile) => {
    e.preventDefault()
    setContextMenu({ x: e.clientX, y: e.clientY, file })
  }, [])

  // Split files: todo, visible artifacts (no hidden state files), folders
  const todoFile = files.find((f) => isTodoFile(f.name))
  const visibleFiles = files.filter((f) => !HIDDEN_FILES.has(f.name) && !isTodoFile(f.name))
  const { rootFiles, folders } = groupFiles(visibleFiles)

  return (
    <aside className="flex w-60 shrink-0 flex-col border-r border-border bg-card/30">
      {/* Tab bar */}
      <div className="flex border-b border-border shrink-0">
        {(['files', 'git', 'sessions'] as const).map((t) => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className={`flex-1 py-2.5 text-center text-[11px] font-medium border-b-2 transition-colors capitalize ${
              tab === t ? 'border-primary text-primary' : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            {t}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* ═══ FILES TAB ═══ */}
        {tab === 'files' && (
          <div className="px-3 pt-3 pb-2">
            {/* TODO section */}
            {todoFile && (
              <>
                <SectionHeader label="Todo" open={todoOpen} onToggle={() => setTodoOpen(!todoOpen)} />
                {todoOpen && (
                  <div className="mt-1 mb-2">
                    <FileEntry
                      file={todoFile}
                      isActive={activeFile === todoFile.path}
                      onClick={() => onSelectFile(todoFile.path)}
                      onContextMenu={(e) => handleContextMenu(e, todoFile)}
                      isTodo
                    />
                  </div>
                )}
              </>
            )}

            {/* Artifacts section */}
            <SectionHeader
              label="Artifacts"
              count={visibleFiles.length}
              open={artifactsOpen}
              onToggle={() => setArtifactsOpen(!artifactsOpen)}
              action={
                <button
                  onClick={() => fileInputRef.current?.click()}
                  disabled={!isConnected}
                  className="flex size-4 items-center justify-center rounded text-muted-foreground hover:text-foreground transition disabled:opacity-40"
                >
                  <Plus className="size-3" />
                </button>
              }
            />
            <input
              ref={fileInputRef}
              type="file"
              multiple
              className="hidden"
              onChange={(e) => {
                if (e.target.files?.length) onUploadFiles(e.target.files)
                e.target.value = ''
              }}
            />

            {artifactsOpen && (
              <div className="mt-1 space-y-0.5">
                {rootFiles.map((f) => (
                  <FileEntry
                    key={f.path}
                    file={f}
                    isActive={activeFile === f.path}
                    onClick={() => onSelectFile(f.path)}
                    onContextMenu={(e) => handleContextMenu(e, f)}
                  />
                ))}

                {Object.entries(folders).map(([dir, dirFiles]) => (
                  <div key={dir} className="mt-1">
                    <button
                      onClick={() => toggleFolder(dir)}
                      className="flex items-center gap-1.5 w-full px-2 py-1.5 text-xs font-medium text-muted-foreground hover:text-foreground transition"
                    >
                      {expandedFolders.has(dir) ? <ChevronDown className="size-3" /> : <ChevronRight className="size-3" />}
                      {dir}/
                      <span className="ml-auto text-[9px]">{dirFiles.length}</span>
                    </button>
                    {expandedFolders.has(dir) && (
                      <div className="pl-4 space-y-0.5">
                        {dirFiles.map((f) => (
                          <FileEntry
                            key={f.path}
                            file={f}
                            isActive={activeFile === f.path}
                            onClick={() => onSelectFile(f.path)}
                            onContextMenu={(e) => handleContextMenu(e, f)}
                          />
                        ))}
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}

            {/* Annotations section */}
            {annotations.length > 0 && (
              <>
                <div className="my-3 border-t border-border/40" />
                <SectionHeader label="Annotations" count={annotations.length} open={annotationsOpen} onToggle={() => setAnnotationsOpen(!annotationsOpen)} />
                {annotationsOpen && (
                  <div className="mt-1 space-y-0.5">
                    {annotations.map((ann, i) => (
                      <div key={ann.id} className="flex items-center gap-2 px-2 py-1 rounded text-xs text-muted-foreground hover:text-foreground hover:bg-muted/30 cursor-pointer transition">
                        <MapPin className="size-3 text-primary shrink-0" />
                        <span className="truncate">{ann.type === 'click' ? ann.note || ann.text : ann.type === 'box' ? ann.note : ann.text}</span>
                        <span className="ml-auto text-[9px] text-muted-foreground/40">#{i + 1}</span>
                      </div>
                    ))}
                  </div>
                )}
              </>
            )}
          </div>
        )}

        {/* ═══ GIT TAB ═══ */}
        {tab === 'git' && (
          <div className="px-3 pt-3">
            {gitStatus && (
              <div className="mb-3">
                <div className="flex items-center gap-2 mb-2.5">
                  <span className="text-xs font-medium font-mono">{gitStatus.branch}</span>
                  <Circle className={`size-1.5 shrink-0 ${gitStatus.clean ? 'fill-emerald-500 text-emerald-500' : 'fill-amber-500 text-amber-500'}`} />
                </div>

                {!gitStatus.clean && (
                  <>
                    {/* Summary counts */}
                    <div className="flex gap-3 text-[10px] text-muted-foreground mb-2.5">
                      {(gitStatus.staged?.length ?? 0) > 0 && (
                        <span className="flex items-center gap-1">
                          <Circle className="size-1.5 fill-emerald-500 text-emerald-500" />
                          {gitStatus.staged!.length} staged
                        </span>
                      )}
                      {(gitStatus.modified?.length ?? 0) > 0 && (
                        <span className="flex items-center gap-1">
                          <Circle className="size-1.5 fill-amber-500 text-amber-500" />
                          {gitStatus.modified!.length} modified
                        </span>
                      )}
                      {(gitStatus.untracked?.length ?? 0) > 0 && (
                        <span className="flex items-center gap-1">
                          <Circle className="size-1.5 fill-red-500 text-red-500" />
                          {gitStatus.untracked!.length} untracked
                        </span>
                      )}
                    </div>

                    {/* Changed files */}
                    <div className="space-y-0.5 mb-2">
                      {[
                        ...((gitStatus.staged ?? []) as unknown[]).map((f) => ({ f, badge: 'S', cls: 'bg-emerald-500/10 text-emerald-500' })),
                        ...((gitStatus.modified ?? []) as unknown[]).map((f) => ({ f, badge: 'M', cls: 'bg-amber-500/10 text-amber-500' })),
                        ...((gitStatus.untracked ?? []) as unknown[]).map((f) => ({ f, badge: '?', cls: 'bg-red-500/10 text-red-500' })),
                      ].map(({ f, badge, cls }, i) => {
                        const p = typeof f === 'string' ? f : (f as { path?: string })?.path ?? ''
                        return (
                          <div key={p || i} className="flex items-center gap-2 px-2 py-1 rounded text-xs text-muted-foreground hover:text-foreground hover:bg-muted/30 cursor-pointer transition">
                            <span className={`text-[9px] font-mono px-1 rounded font-bold ${cls}`}>{badge}</span>
                            <span className="truncate">{p.split('/').pop()}</span>
                          </div>
                        )
                      })}
                    </div>

                    <button
                      onClick={onShowDiff}
                      className="w-full flex items-center justify-center gap-1.5 text-[10px] text-primary font-medium py-1.5 rounded-md border border-primary/20 hover:bg-primary/5 transition"
                    >
                      <GitCompareArrows className="size-3" />
                      View diff
                    </button>
                  </>
                )}
              </div>
            )}

            {gitLog && gitLog.length > 0 && (
              <>
                {gitStatus && <div className="border-t border-border/40 mb-3" />}
                <div className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50 mb-2">Recent Commits</div>
                <div className="space-y-0.5">
                  {gitLog.slice(0, 10).map((entry) => (
                    <div
                      key={entry.hash}
                      onClick={() => onSelectCommit(entry.hash)}
                      className="group cursor-pointer rounded px-2 py-1.5 -mx-1 hover:bg-muted/30 transition"
                    >
                      <div className="flex items-center gap-2 text-xs">
                        <span className="font-mono text-[9px] text-primary/60 shrink-0">{entry.hash.slice(0, 7)}</span>
                        <span className="truncate text-muted-foreground group-hover:text-foreground transition">{entry.subject}</span>
                      </div>
                      <div className="text-[9px] text-muted-foreground/40 mt-0.5 pl-[52px]">{entry.date}</div>
                    </div>
                  ))}
                </div>
              </>
            )}

            {!gitStatus && (
              <p className="text-[10px] text-muted-foreground/60">Connect CLI to see git info.</p>
            )}
          </div>
        )}

        {/* ═══ SESSIONS TAB ═══ */}
        {tab === 'sessions' && (
          <div className="px-3 pt-3 text-xs text-muted-foreground">
            <p className="text-[10px] text-muted-foreground/60">
              Workspace and session history will appear here when workspaces are active.
            </p>
          </div>
        )}
      </div>

      {/* Footer: CLI connection */}
      <div className="shrink-0 border-t border-border px-2 py-1.5">
        <CliStatusPopover onAddSkill={() => {}} />
      </div>

      {contextMenu && (
        <ArtifactContextMenu
          menu={contextMenu}
          onClose={() => setContextMenu(null)}
          onDelete={() => setContextMenu(null)}
        />
      )}
    </aside>
  )
}

// ── Collapsible section header ──

function SectionHeader({
  label, count, open, onToggle, action,
}: {
  label: string
  count?: number
  open: boolean
  onToggle: () => void
  action?: React.ReactNode
}) {
  return (
    <div className="flex items-center gap-1">
      <button onClick={onToggle} className="flex items-center gap-1 flex-1 min-w-0">
        {open ? <ChevronDown className="size-3 text-muted-foreground/40 shrink-0" /> : <ChevronRight className="size-3 text-muted-foreground/40 shrink-0" />}
        <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60">{label}</span>
        {count != null && (
          <span className="text-[9px] text-muted-foreground/40">{count}</span>
        )}
      </button>
      {action}
    </div>
  )
}

// ── File entry ──

function FileEntry({
  file, isActive, onClick, onContextMenu, isTodo,
}: {
  file: SessionFile
  isActive: boolean
  onClick: () => void
  onContextMenu: (e: React.MouseEvent) => void
  isTodo?: boolean
}) {
  const { icon: Icon, color } = FILE_ICONS[file.type]

  return (
    <button
      onClick={onClick}
      onContextMenu={onContextMenu}
      className={`flex items-center gap-2 w-full px-2 py-1.5 rounded-md text-xs transition ${
        isActive
          ? 'border-l-2 border-primary bg-primary/5 text-primary font-medium'
          : 'text-muted-foreground hover:text-foreground hover:bg-muted/30'
      }`}
    >
      {isTodo ? (
        <CheckSquare className={`size-3.5 shrink-0 ${isActive ? 'text-primary' : 'text-emerald-500'}`} />
      ) : (
        <Icon className={`size-3.5 shrink-0 ${isActive ? 'text-primary' : color}`} />
      )}
      <span className="truncate text-left">{file.name}</span>
    </button>
  )
}

// ── Helpers ──

function groupFiles(files: SessionFile[]): { rootFiles: SessionFile[]; folders: Record<string, SessionFile[]> } {
  const rootFiles: SessionFile[] = []
  const folders: Record<string, SessionFile[]> = {}

  for (const f of files) {
    const parts = f.path.split('/')
    if (parts.length === 1) {
      rootFiles.push(f)
    } else {
      const dir = parts.slice(0, -1).join('/')
      if (!folders[dir]) folders[dir] = []
      folders[dir].push(f)
    }
  }

  return { rootFiles, folders }
}
