// Right sidebar showing artifacts in .ship-session/ grouped by type, plus a TODO checklist.
// Artifacts are sorted newest-first. Clicking a file opens it in the main viewer.

import { FileCode, Image, FileText, File, ChevronRight, PanelRightClose } from 'lucide-react'
import { SessionTodo } from './SessionTodo'
import type { SessionFile } from './types'

interface SessionActivityProps {
  files: SessionFile[]
  activeFile: string | null
  isLoading: boolean
  isConnected: boolean
  onSelectFile: (path: string) => void
  onClose: () => void
}

const TYPE_CONFIG = {
  html: { icon: FileCode, label: 'HTML', color: 'text-orange-500' },
  image: { icon: Image, label: 'Images', color: 'text-blue-500' },
  markdown: { icon: FileText, label: 'Markdown', color: 'text-emerald-500' },
  other: { icon: File, label: 'Other', color: 'text-muted-foreground' },
} as const

function groupByType(files: SessionFile[]): Record<SessionFile['type'], SessionFile[]> {
  const groups: Record<SessionFile['type'], SessionFile[]> = {
    html: [],
    image: [],
    markdown: [],
    other: [],
  }
  for (const file of files) {
    groups[file.type].push(file)
  }
  return groups
}

function formatTime(ts: number): string {
  const now = Date.now()
  const diffMs = now - ts
  if (diffMs < 60_000) return 'just now'
  if (diffMs < 3_600_000) return `${Math.floor(diffMs / 60_000)}m ago`
  const d = new Date(ts)
  return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

export function SessionActivity({
  files,
  activeFile,
  isLoading,
  isConnected,
  onSelectFile,
  onClose,
}: SessionActivityProps) {
  const sorted = [...files].sort((a, b) => b.modifiedAt - a.modifiedAt)
  const groups = groupByType(sorted)

  return (
    <div className="w-72 shrink-0 border-l border-border/60 bg-card/30 flex flex-col min-h-0">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/60 px-3 py-2 shrink-0">
        <h3 className="text-xs font-semibold text-foreground">Activity</h3>
        <button
          onClick={onClose}
          className="text-muted-foreground hover:text-foreground"
          aria-label="Close activity panel"
        >
          <PanelRightClose className="size-3.5" />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-4">
        {/* Artifacts section */}
        <ArtifactsSection
          groups={groups}
          activeFile={activeFile}
          isLoading={isLoading}
          isConnected={isConnected}
          filesCount={files.length}
          onSelectFile={onSelectFile}
        />

        {/* TODO section */}
        {isConnected && (
          <div>
            <div className="flex items-center gap-1.5 px-1 mb-1.5">
              <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                TODO
              </span>
            </div>
            <SessionTodo />
          </div>
        )}
      </div>
    </div>
  )
}

function ArtifactsSection({
  groups,
  activeFile,
  isLoading,
  isConnected,
  filesCount,
  onSelectFile,
}: {
  groups: Record<SessionFile['type'], SessionFile[]>
  activeFile: string | null
  isLoading: boolean
  isConnected: boolean
  filesCount: number
  onSelectFile: (path: string) => void
}) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-1.5 px-1">
        <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
          Artifacts
        </span>
        {filesCount > 0 && (
          <span className="text-[10px] text-muted-foreground/60">{filesCount}</span>
        )}
      </div>

      {!isConnected && (
        <p className="text-[11px] text-muted-foreground px-1 py-4 text-center">
          Connect CLI to view session artifacts
        </p>
      )}

      {isConnected && isLoading && (
        <div className="space-y-2 px-1 py-2">
          {Array.from({ length: 4 }).map((_, i) => (
            <div key={i} className="h-8 rounded-md bg-muted animate-pulse" />
          ))}
        </div>
      )}

      {isConnected && !isLoading && filesCount === 0 && (
        <p className="text-[11px] text-muted-foreground px-1 py-4 text-center">
          No artifacts yet. Agent outputs will appear here.
        </p>
      )}

      {(Object.keys(groups) as SessionFile['type'][]).map((type) => {
        const group = groups[type]
        if (group.length === 0) return null
        const cfg = TYPE_CONFIG[type]
        const Icon = cfg.icon

        return (
          <div key={type}>
            <div className="flex items-center gap-1.5 px-1 mb-1">
              <Icon className={`size-3 ${cfg.color}`} />
              <span className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {cfg.label}
              </span>
              <span className="text-[10px] text-muted-foreground/60">{group.length}</span>
            </div>

            <div className="space-y-0.5">
              {group.map((file) => (
                <ArtifactRow
                  key={file.path}
                  file={file}
                  isActive={activeFile === file.path}
                  onSelect={() => onSelectFile(file.path)}
                />
              ))}
            </div>
          </div>
        )
      })}
    </div>
  )
}

function ArtifactRow({
  file,
  isActive,
  onSelect,
}: {
  file: SessionFile
  isActive: boolean
  onSelect: () => void
}) {
  return (
    <button
      onClick={onSelect}
      className={`w-full flex items-center gap-2 rounded-md px-2 py-1.5 text-left transition ${
        isActive
          ? 'bg-primary/10 text-foreground'
          : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
      }`}
    >
      <div className="flex-1 min-w-0">
        <p className="text-[11px] font-medium truncate">{file.name}</p>
        <div className="flex items-center gap-2">
          <span className="text-[10px] text-muted-foreground/70">
            {formatTime(file.modifiedAt)}
          </span>
          <span className="text-[10px] text-muted-foreground/50">
            {formatSize(file.size)}
          </span>
        </div>
      </div>
      {isActive && <ChevronRight className="size-3 shrink-0 text-primary" />}
    </button>
  )
}
