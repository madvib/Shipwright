// Right sidebar showing all files in .ship-session/ grouped by type.
// Files are sorted newest-first. Clicking an HTML file switches the canvas.

import { FileCode, Image, FileText, File, ChevronRight, PanelRightClose } from 'lucide-react'
import type { SessionFile } from './types'

interface SessionTimelineProps {
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
  const d = new Date(ts)
  const now = Date.now()
  const diffMs = now - ts
  if (diffMs < 60_000) return 'just now'
  if (diffMs < 3_600_000) return `${Math.floor(diffMs / 60_000)}m ago`
  return d.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
}

export function SessionTimeline({
  files,
  activeFile,
  isLoading,
  isConnected,
  onSelectFile,
  onClose,
}: SessionTimelineProps) {
  const sorted = [...files].sort((a, b) => b.modifiedAt - a.modifiedAt)
  const groups = groupByType(sorted)

  return (
    <div className="w-64 shrink-0 border-l border-border/60 bg-card/30 flex flex-col min-h-0">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/60 px-3 py-2 shrink-0">
        <h3 className="text-xs font-semibold text-foreground">Timeline</h3>
        <button
          onClick={onClose}
          className="text-muted-foreground hover:text-foreground"
          aria-label="Close timeline"
        >
          <PanelRightClose className="size-3.5" />
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto px-2 py-2 space-y-3">
        {!isConnected && (
          <p className="text-[11px] text-muted-foreground px-1 py-4 text-center">
            Connect CLI to view session files
          </p>
        )}

        {isConnected && isLoading && (
          <div className="space-y-2 px-1 py-2">
            {Array.from({ length: 4 }).map((_, i) => (
              <div key={i} className="h-8 rounded-md bg-muted animate-pulse" />
            ))}
          </div>
        )}

        {isConnected && !isLoading && files.length === 0 && (
          <p className="text-[11px] text-muted-foreground px-1 py-4 text-center">
            No session files yet. Agent artifacts will appear here.
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
                  <button
                    key={file.path}
                    onClick={() => onSelectFile(file.path)}
                    className={`w-full flex items-center gap-2 rounded-md px-2 py-1.5 text-left transition ${
                      activeFile === file.path
                        ? 'bg-primary/10 text-foreground'
                        : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
                    }`}
                  >
                    <div className="flex-1 min-w-0">
                      <p className="text-[11px] font-medium truncate">{file.name}</p>
                      <p className="text-[10px] text-muted-foreground/70">{formatTime(file.modifiedAt)}</p>
                    </div>
                    {activeFile === file.path && (
                      <ChevronRight className="size-3 shrink-0 text-primary" />
                    )}
                  </button>
                ))}
              </div>
            </div>
          )
        })}
      </div>
    </div>
  )
}
