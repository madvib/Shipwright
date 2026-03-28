// Right sidebar showing session content in tabs: Artifacts, TODO, Annotations.
// Compact styling with text-xs throughout and scrollable tab content.

import { useState } from 'react'
import { FileCode, Image, FileText, File, ChevronRight, PanelRightClose } from 'lucide-react'
import { SessionTodo } from './SessionTodo'
import { AnnotationsList } from './AnnotationsList'
import type { Annotation, SessionFile } from './types'

type TabId = 'artifacts' | 'todo' | 'annotations'

interface SessionActivityProps {
  files: SessionFile[]
  activeFile: string | null
  isLoading: boolean
  isConnected: boolean
  annotations: Annotation[]
  onSelectFile: (path: string) => void
  onRemoveAnnotation: (id: string) => void
  onExportAnnotations: () => void
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
    html: [], image: [], markdown: [], other: [],
  }
  for (const file of files) groups[file.type].push(file)
  return groups
}

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

export function SessionActivity({
  files,
  activeFile,
  isLoading,
  isConnected,
  annotations,
  onSelectFile,
  onRemoveAnnotation,
  onExportAnnotations,
  onClose,
}: SessionActivityProps) {
  const [activeTab, setActiveTab] = useState<TabId>('artifacts')
  const sorted = [...files].sort((a, b) => b.modifiedAt - a.modifiedAt)
  const groups = groupByType(sorted)

  const tabs: { id: TabId; label: string; count?: number }[] = [
    { id: 'artifacts', label: 'Artifacts', count: files.length || undefined },
    { id: 'todo', label: 'TODO' },
    { id: 'annotations', label: 'Notes', count: annotations.length || undefined },
  ]

  return (
    <div className="w-64 shrink-0 border-l border-border/60 bg-card/30 flex flex-col min-h-0">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/60 px-2.5 py-1.5 shrink-0">
        <h3 className="text-[11px] font-semibold text-foreground">Activity</h3>
        <button onClick={onClose} className="text-muted-foreground hover:text-foreground" aria-label="Close activity panel">
          <PanelRightClose className="size-3" />
        </button>
      </div>

      {/* Tab bar */}
      <div className="flex border-b border-border/60 shrink-0">
        {tabs.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`flex-1 px-2 py-1.5 text-[10px] font-medium transition border-b-2 ${
              activeTab === tab.id
                ? 'border-primary text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            {tab.label}
            {tab.count != null && <span className="ml-1 text-[9px] opacity-60">{tab.count}</span>}
          </button>
        ))}
      </div>

      {/* Scrollable tab content */}
      <div className="flex-1 overflow-y-auto px-1.5 py-1.5">
        {activeTab === 'artifacts' && (
          <ArtifactsContent groups={groups} activeFile={activeFile} isLoading={isLoading} isConnected={isConnected} filesCount={files.length} onSelectFile={onSelectFile} />
        )}
        {activeTab === 'todo' && <TodoContent isConnected={isConnected} />}
        {activeTab === 'annotations' && (
          <AnnotationsList annotations={annotations} onRemove={onRemoveAnnotation} onExport={onExportAnnotations} />
        )}
      </div>
    </div>
  )
}

function ArtifactsContent({
  groups, activeFile, isLoading, isConnected, filesCount, onSelectFile,
}: {
  groups: Record<SessionFile['type'], SessionFile[]>
  activeFile: string | null
  isLoading: boolean
  isConnected: boolean
  filesCount: number
  onSelectFile: (path: string) => void
}) {
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

  return (
    <div className="space-y-2">
      {(Object.keys(groups) as SessionFile['type'][]).map((type) => {
        const group = groups[type]
        if (group.length === 0) return null
        const cfg = TYPE_CONFIG[type]
        const Icon = cfg.icon
        return (
          <div key={type}>
            <div className="flex items-center gap-1 px-0.5 mb-0.5">
              <Icon className={`size-2.5 ${cfg.color}`} />
              <span className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wider">{cfg.label}</span>
              <span className="text-[9px] text-muted-foreground/50">{group.length}</span>
            </div>
            <div className="space-y-px">
              {group.map((file) => (
                <button
                  key={file.path}
                  onClick={() => onSelectFile(file.path)}
                  className={`w-full flex items-center gap-1.5 rounded px-1.5 py-1 text-left transition ${
                    activeFile === file.path ? 'bg-primary/10 text-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
                  }`}
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-[10px] font-medium truncate">{file.name}</p>
                    <div className="flex items-center gap-1.5">
                      <span className="text-[9px] text-muted-foreground/70">{formatTime(file.modifiedAt)}</span>
                      <span className="text-[9px] text-muted-foreground/50">{formatSize(file.size)}</span>
                    </div>
                  </div>
                  {activeFile === file.path && <ChevronRight className="size-2.5 shrink-0 text-primary" />}
                </button>
              ))}
            </div>
          </div>
        )
      })}
    </div>
  )
}

function TodoContent({ isConnected }: { isConnected: boolean }) {
  if (!isConnected) {
    return <p className="text-[10px] text-muted-foreground px-1 py-6 text-center">Connect CLI for TODO management</p>
  }
  return <SessionTodo />
}
