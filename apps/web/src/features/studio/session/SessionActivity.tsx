// Right sidebar with four tabs: Canvas, Diff, Artifacts, Notes.

import { useState, useRef, useCallback, useMemo } from 'react'
import { PanelRightClose, Plus, GitCompareArrows, FileText, Layers, StickyNote } from 'lucide-react'
import { SessionTodo } from './SessionTodo'
import { AnnotationsList } from './AnnotationsList'
import { ArtifactsContent } from './ArtifactsContent'
import { ArtifactContextMenu } from './ArtifactContextMenu'
import type { ArtifactMenuState } from './ArtifactContextMenu'
import type { Annotation, SessionFile, ViewMode } from './types'

type TabId = 'canvas' | 'diff' | 'artifacts' | 'notes'

interface SessionActivityProps {
  files: SessionFile[]
  activeFile: string | null
  isLoading: boolean
  isConnected: boolean
  annotations: Annotation[]
  viewMode: ViewMode
  hasDiff: boolean
  openCanvasTabs: string[]
  activeCanvasTab: string | null
  annotationMode: boolean
  onSelectFile: (path: string) => void
  onDeleteFile: (path: string) => void
  onUploadFiles: (files: FileList) => void
  onRemoveAnnotation: (id: string) => void
  onExportAnnotations: () => void
  onClose: () => void
  onSetViewMode: (mode: ViewMode) => void
  onSelectCanvasTab: (path: string) => void
  onToggleAnnotationMode: () => void
}

function groupByDirectory(files: SessionFile[]): Record<string, SessionFile[]> {
  const groups: Record<string, SessionFile[]> = {}
  for (const file of files) {
    const dir = file.path.includes('/') ? file.path.substring(0, file.path.lastIndexOf('/')) : ''
    if (!groups[dir]) groups[dir] = []
    groups[dir].push(file)
  }
  return groups
}

export function SessionActivity({
  files,
  activeFile,
  isLoading,
  isConnected,
  annotations,
  viewMode,
  hasDiff,
  openCanvasTabs,
  activeCanvasTab,
  annotationMode,
  onSelectFile,
  onDeleteFile,
  onUploadFiles,
  onRemoveAnnotation,
  onExportAnnotations,
  onClose,
  onSetViewMode,
  onSelectCanvasTab,
  onToggleAnnotationMode,
}: SessionActivityProps) {
  const [activeTab, setActiveTab] = useState<TabId>('artifacts')
  const [contextMenu, setContextMenu] = useState<ArtifactMenuState | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const sorted = useMemo(() => [...files].sort((a, b) => b.modifiedAt - a.modifiedAt), [files])
  const dirGroups = useMemo(() => groupByDirectory(sorted), [sorted])

  const tabs: { id: TabId; label: string; icon: typeof Layers; count?: number }[] = [
    { id: 'canvas', label: 'Canvas', icon: Layers },
    { id: 'diff', label: 'Diff', icon: GitCompareArrows },
    { id: 'artifacts', label: 'Artifacts', icon: FileText, count: files.length || undefined },
    { id: 'notes', label: 'Notes', icon: StickyNote, count: annotations.length || undefined },
  ]

  const handleContextMenu = useCallback((e: React.MouseEvent, file: SessionFile) => {
    e.preventDefault()
    setContextMenu({ file, x: e.clientX, y: e.clientY })
  }, [])

  const handleUploadClick = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

  const handleFileInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files.length > 0) {
      onUploadFiles(e.target.files)
      e.target.value = ''
    }
  }, [onUploadFiles])

  return (
    <div className="w-64 shrink-0 border-l border-border/60 bg-card/30 flex flex-col min-h-0">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/60 px-2.5 py-1.5 shrink-0">
        <h3 className="text-[11px] font-semibold text-foreground">Activity</h3>
        <div className="flex items-center gap-1">
          {isConnected && (
            <button
              onClick={handleUploadClick}
              className="text-muted-foreground hover:text-foreground transition-colors"
              aria-label="Upload file"
              title="Upload file to session"
            >
              <Plus className="size-3" />
            </button>
          )}
          <button onClick={onClose} className="text-muted-foreground hover:text-foreground" aria-label="Close activity panel">
            <PanelRightClose className="size-3" />
          </button>
        </div>
        <input ref={fileInputRef} type="file" className="hidden" onChange={handleFileInputChange} multiple />
      </div>

      {/* Tab bar */}
      <div className="flex border-b border-border/60 shrink-0">
        {tabs.map((tab) => {
          const Icon = tab.icon
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex-1 flex items-center justify-center gap-1 px-1 py-1.5 text-[10px] font-medium transition border-b-2 ${
                activeTab === tab.id
                  ? 'border-primary text-foreground'
                  : 'border-transparent text-muted-foreground hover:text-foreground'
              }`}
              title={tab.label}
            >
              <Icon className="size-3" />
              <span className="hidden xl:inline">{tab.label}</span>
              {tab.count != null && <span className="text-[9px] opacity-60">{tab.count}</span>}
            </button>
          )
        })}
      </div>

      {/* Scrollable tab content */}
      <div className="flex-1 overflow-y-auto px-1.5 py-1.5">
        {activeTab === 'canvas' && (
          <CanvasTabContent
            openTabs={openCanvasTabs}
            activeTab={activeCanvasTab}
            annotationMode={annotationMode}
            viewMode={viewMode}
            onSelectTab={(path) => { onSelectCanvasTab(path); onSetViewMode('canvas') }}
            onToggleAnnotationMode={onToggleAnnotationMode}
            onSetViewMode={onSetViewMode}
          />
        )}
        {activeTab === 'diff' && (
          <DiffTabContent
            hasDiff={hasDiff}
            isActive={viewMode === 'diff'}
            onShowDiff={() => onSetViewMode('diff')}
          />
        )}
        {activeTab === 'artifacts' && (
          <ArtifactsContent
            dirGroups={dirGroups}
            activeFile={activeFile}
            isLoading={isLoading}
            isConnected={isConnected}
            filesCount={files.length}
            onSelectFile={onSelectFile}
            onContextMenu={handleContextMenu}
          />
        )}
        {activeTab === 'notes' && (
          <NotesTabContent
            isConnected={isConnected}
            annotations={annotations}
            onRemoveAnnotation={onRemoveAnnotation}
            onExportAnnotations={onExportAnnotations}
          />
        )}
      </div>

      {contextMenu && (
        <ArtifactContextMenu
          menu={contextMenu}
          onDelete={onDeleteFile}
          onClose={() => setContextMenu(null)}
        />
      )}
    </div>
  )
}

function CanvasTabContent({ openTabs, activeTab, annotationMode, viewMode, onSelectTab, onToggleAnnotationMode, onSetViewMode }: {
  openTabs: string[]
  activeTab: string | null
  annotationMode: boolean
  viewMode: ViewMode
  onSelectTab: (path: string) => void
  onToggleAnnotationMode: () => void
  onSetViewMode: (mode: ViewMode) => void
}) {
  return (
    <div className="space-y-3">
      {viewMode !== 'canvas' && (
        <button
          onClick={() => onSetViewMode('canvas')}
          className="flex items-center gap-1.5 w-full px-2 py-1.5 text-[11px] font-medium text-primary hover:bg-primary/10 rounded-md transition"
        >
          <Layers className="size-3" />
          Switch to canvas view
        </button>
      )}
      <button
        onClick={onToggleAnnotationMode}
        className={`flex items-center gap-1.5 w-full px-2 py-1.5 text-[11px] font-medium rounded-md transition ${
          annotationMode ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
        }`}
      >
        {annotationMode ? 'Annotating...' : 'Toggle annotations'}
      </button>
      <div>
        <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">Open tabs</p>
        {openTabs.length === 0 ? (
          <p className="text-[10px] text-muted-foreground px-1">No canvas tabs open</p>
        ) : (
          <div className="space-y-px">
            {openTabs.map((path) => {
              const name = path.split('/').pop() ?? path
              return (
                <button
                  key={path}
                  onClick={() => onSelectTab(path)}
                  className={`w-full flex items-center gap-1.5 rounded px-1.5 py-1 text-left transition text-[10px] ${
                    activeTab === path ? 'bg-primary/10 text-foreground font-medium' : 'text-muted-foreground hover:bg-muted/50'
                  }`}
                >
                  <Layers className="size-2.5 shrink-0" />
                  <span className="truncate">{name}</span>
                </button>
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}

function DiffTabContent({ hasDiff, isActive, onShowDiff }: {
  hasDiff: boolean; isActive: boolean; onShowDiff: () => void
}) {
  if (!hasDiff) {
    return (
      <div className="text-center py-6 px-2">
        <GitCompareArrows className="size-6 mx-auto mb-2 text-muted-foreground/40" />
        <p className="text-[10px] text-muted-foreground">
          No diff available. Run <code className="text-[9px] bg-muted px-1 rounded">/diff</code> to generate one.
        </p>
      </div>
    )
  }
  return (
    <button
      onClick={onShowDiff}
      className={`flex items-center gap-1.5 w-full px-2 py-1.5 text-[11px] font-medium rounded-md transition ${
        isActive ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
      }`}
    >
      <GitCompareArrows className="size-3" />
      {isActive ? 'Viewing diff' : 'View diff'}
    </button>
  )
}

function NotesTabContent({ isConnected, annotations, onRemoveAnnotation, onExportAnnotations }: {
  isConnected: boolean; annotations: Annotation[]
  onRemoveAnnotation: (id: string) => void; onExportAnnotations: () => void
}) {
  return (
    <div className="space-y-3">
      <div>
        <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">TODO</p>
        {isConnected ? <SessionTodo /> : (
          <p className="text-[10px] text-muted-foreground px-1">Connect CLI for TODO management</p>
        )}
      </div>
      <div>
        <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">Annotations</p>
        <AnnotationsList annotations={annotations} onRemove={onRemoveAnnotation} onExport={onExportAnnotations} />
      </div>
    </div>
  )
}
