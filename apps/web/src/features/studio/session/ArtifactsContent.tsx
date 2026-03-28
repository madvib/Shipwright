// Artifacts tab content: file browser grouped by directory with collapsible folders.

import { useState, useMemo } from 'react'
import { FileCode, Image, FileText, File, ChevronRight, ChevronDown, FolderOpen } from 'lucide-react'
import type { SessionFile } from './types'

const TYPE_GROUP_LABELS: Record<SessionFile['type'], string> = {
  html: 'HTML',
  markdown: 'Markdown',
  image: 'Images',
  other: 'Other',
}

const TYPE_GROUP_ORDER: SessionFile['type'][] = ['html', 'markdown', 'image', 'other']

const TYPE_CONFIG = {
  html: { icon: FileCode, color: 'text-orange-500' },
  image: { icon: Image, color: 'text-blue-500' },
  markdown: { icon: FileText, color: 'text-emerald-500' },
  other: { icon: File, color: 'text-muted-foreground' },
} as const

function formatTime(ts: number): string {
  const diffMs = Date.now() - ts
  if (diffMs < 60_000) return 'just now'
  if (diffMs < 3_600_000) return `${Math.floor(diffMs / 60_000)}m ago`
  return new Date(ts).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
}

function FileRow({ file, isActive, onSelect, onContextMenu }: {
  file: SessionFile
  isActive: boolean
  onSelect: () => void
  onContextMenu: (e: React.MouseEvent) => void
}) {
  const cfg = TYPE_CONFIG[file.type]
  const Icon = cfg.icon
  return (
    <button
      onClick={onSelect}
      onContextMenu={onContextMenu}
      className={`w-full flex items-center gap-1.5 rounded px-1.5 py-1 text-left transition ${
        isActive ? 'bg-primary/10 text-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
      }`}
    >
      <Icon className={`size-2.5 shrink-0 ${cfg.color}`} />
      <div className="flex-1 min-w-0">
        <p className="text-[10px] font-medium truncate">{file.name}</p>
        <div className="flex items-center gap-1.5">
          <span className="text-[9px] text-muted-foreground/70">{formatTime(file.modifiedAt)}</span>
          <span className="text-[9px] text-muted-foreground/50">{formatSize(file.size)}</span>
        </div>
      </div>
      {isActive && <ChevronRight className="size-2.5 shrink-0 text-primary" />}
    </button>
  )
}

export function ArtifactsContent({
  dirGroups, activeFile, isLoading, isConnected, filesCount, onSelectFile, onContextMenu,
}: {
  dirGroups: Record<string, SessionFile[]>
  activeFile: string | null
  isLoading: boolean
  isConnected: boolean
  filesCount: number
  onSelectFile: (path: string) => void
  onContextMenu: (e: React.MouseEvent, file: SessionFile) => void
}) {
  const [collapsedDirs, setCollapsedDirs] = useState<Set<string>>(new Set())

  if (!isConnected) {
    return <p className="text-[10px] text-muted-foreground px-1 py-6 text-center">Connect CLI to view session artifacts</p>
  }
  if (isLoading) {
    return (
      <div className="space-y-1.5 px-0.5 py-2">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="h-7 rounded-md bg-muted animate-pulse" />
        ))}
      </div>
    )
  }
  if (filesCount === 0) {
    return <p className="text-[10px] text-muted-foreground px-1 py-6 text-center">No artifacts yet. Agent outputs will appear here.</p>
  }

  const rootFiles = dirGroups[''] ?? []
  const folderKeys = Object.keys(dirGroups).filter((k) => k !== '').sort()

  // Group root files by type for organized display
  const rootTypeGroups = useMemo(() => {
    const groups: Partial<Record<SessionFile['type'], SessionFile[]>> = {}
    for (const file of rootFiles) {
      if (!groups[file.type]) groups[file.type] = []
      groups[file.type]!.push(file)
    }
    return groups
  }, [rootFiles])

  const toggleDir = (dir: string) => {
    setCollapsedDirs((prev) => {
      const next = new Set(prev)
      if (next.has(dir)) next.delete(dir)
      else next.add(dir)
      return next
    })
  }

  return (
    <div className="space-y-1">
      {rootFiles.length > 0 && (
        <div className="space-y-2">
          <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1">Session</p>
          {TYPE_GROUP_ORDER.map((type) => {
            const group = rootTypeGroups[type]
            if (!group || group.length === 0) return null
            return (
              <div key={type}>
                <p className="text-[9px] text-muted-foreground/60 px-1.5 mb-0.5">{TYPE_GROUP_LABELS[type]}</p>
                <div className="space-y-px">
                  {group.map((file) => (
                    <FileRow
                      key={file.path}
                      file={file}
                      isActive={activeFile === file.path}
                      onSelect={() => onSelectFile(file.path)}
                      onContextMenu={(e) => onContextMenu(e, file)}
                    />
                  ))}
                </div>
              </div>
            )
          })}
        </div>
      )}
      {folderKeys.map((dir) => {
        const files = dirGroups[dir]
        const isCollapsed = collapsedDirs.has(dir)
        return (
          <div key={dir}>
            <button
              onClick={() => toggleDir(dir)}
              className="flex items-center gap-1 px-0.5 py-0.5 w-full text-left hover:bg-muted/30 rounded transition"
            >
              {isCollapsed
                ? <ChevronRight className="size-2.5 text-muted-foreground shrink-0" />
                : <ChevronDown className="size-2.5 text-muted-foreground shrink-0" />}
              <FolderOpen className="size-2.5 text-muted-foreground shrink-0" />
              <span className="text-[9px] font-semibold text-muted-foreground truncate">{dir}</span>
              <span className="text-[9px] text-muted-foreground/50 shrink-0">{files.length}</span>
            </button>
            {!isCollapsed && (
              <div className="space-y-px pl-2">
                {files.map((file) => (
                  <FileRow
                    key={file.path}
                    file={file}
                    isActive={activeFile === file.path}
                    onSelect={() => onSelectFile(file.path)}
                    onContextMenu={(e) => onContextMenu(e, file)}
                  />
                ))}
              </div>
            )}
          </div>
        )
      })}
    </div>
  )
}
