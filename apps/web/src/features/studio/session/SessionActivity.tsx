// Right sidebar with four tabs: Preview, Diff, Artifacts, Notes.

import { useState, useRef, useCallback, useMemo } from 'react'
import { PanelRightClose, Plus, GitCompareArrows, FileText, Layers, StickyNote } from 'lucide-react'
import { PreviewTabContent } from './PreviewTabContent'
import { DiffTabContent } from './DiffTabContent'
import { ArtifactsContent } from './ArtifactsContent'
import { NotesTabContent } from './NotesTabContent'
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
    { id: 'canvas', label: 'Preview', icon: Layers },
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
        <h3 className="text-[11px] font-semibold text-foreground">Session</h3>
        <div className="flex items-center gap-0.5">
          {isConnected && (
            <>
              <button
                onClick={handleUploadClick}
                className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors"
                aria-label="Upload file"
                title="Upload file to session"
              >
                <Plus className="size-3" />
              </button>
              <div className="w-px h-3 bg-border/60 mx-0.5" />
            </>
          )}
          <button
            onClick={onClose}
            className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-muted/50 transition-colors"
            aria-label="Close activity panel"
          >
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
          <PreviewTabContent
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
