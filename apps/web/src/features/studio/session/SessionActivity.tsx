// Right sidebar showing session content in tabs: Artifacts, TODO, Annotations.

import { useState, useRef, useCallback, useMemo } from 'react'
import { PanelRightClose, Plus } from 'lucide-react'
import { SessionTodo } from './SessionTodo'
import { AnnotationsList } from './AnnotationsList'
import { ArtifactsContent } from './ArtifactsContent'
import { ArtifactContextMenu } from './ArtifactContextMenu'
import type { ArtifactMenuState } from './ArtifactContextMenu'
import type { Annotation, SessionFile } from './types'

type TabId = 'artifacts' | 'todo' | 'annotations'

interface SessionActivityProps {
  files: SessionFile[]
  activeFile: string | null
  isLoading: boolean
  isConnected: boolean
  annotations: Annotation[]
  onSelectFile: (path: string) => void
  onDeleteFile: (path: string) => void
  onUploadFiles: (files: FileList) => void
  onRemoveAnnotation: (id: string) => void
  onExportAnnotations: () => void
  onClose: () => void
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
  onSelectFile,
  onDeleteFile,
  onUploadFiles,
  onRemoveAnnotation,
  onExportAnnotations,
  onClose,
}: SessionActivityProps) {
  const [activeTab, setActiveTab] = useState<TabId>('artifacts')
  const [contextMenu, setContextMenu] = useState<ArtifactMenuState | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const sorted = useMemo(() => [...files].sort((a, b) => b.modifiedAt - a.modifiedAt), [files])
  const dirGroups = useMemo(() => groupByDirectory(sorted), [sorted])

  const tabs: { id: TabId; label: string; count?: number }[] = [
    { id: 'artifacts', label: 'Artifacts', count: files.length || undefined },
    { id: 'todo', label: 'TODO' },
    { id: 'annotations', label: 'Notes', count: annotations.length || undefined },
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
        {activeTab === 'todo' && <TodoContent isConnected={isConnected} />}
        {activeTab === 'annotations' && (
          <AnnotationsList annotations={annotations} onRemove={onRemoveAnnotation} onExport={onExportAnnotations} />
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

function TodoContent({ isConnected }: { isConnected: boolean }) {
  if (!isConnected) {
    return <p className="text-[10px] text-muted-foreground px-1 py-6 text-center">Connect CLI for TODO management</p>
  }
  return <SessionTodo />
}
