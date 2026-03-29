import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useEffect } from 'react'
import { Layers, WifiOff, X } from 'lucide-react'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { ArtifactViewer } from '#/features/studio/session/ArtifactViewer'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { SessionSidebar } from '#/features/studio/session/SessionSidebar'
import { useSessionFiles, useSessionFileContent, useUploadSessionFile } from '#/features/studio/session/useSessionFiles'
import { useSessionDrafts } from '#/features/studio/session/useSessionDrafts'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
import { useGitStatus, useGitLog, useGitDiff } from '#/features/studio/session/useGitInfo'
import { SessionSkeleton } from '#/features/studio/session/SessionSkeleton'
import { DropZoneOverlay } from '#/features/studio/session/DropZoneOverlay'

export const Route = createFileRoute('/studio/session')({
  component: SessionPage,
  pendingComponent: SessionSkeleton,
  ssr: false,
})

type ViewMode = 'file' | 'diff'

interface OpenTab {
  path: string
  name: string
  type: string
}

function SessionPage() {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'

  // ── All hooks unconditionally at top level ──
  const { files } = useSessionFiles()
  const uploadMutation = useUploadSessionFile()
  const { diffText } = useDiffContent()
  const { data: gitStatus } = useGitStatus()
  const { data: gitLog } = useGitLog(5)
  const ann = useAnnotations()
  const drafts = useSessionDrafts()

  // ── Tab state: ALL file types ──
  const [openTabs, setOpenTabs] = useState<OpenTab[]>([])
  const [activeTabPath, setActiveTabPath] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<ViewMode>('file')
  const [selectedCommitHash, setSelectedCommitHash] = useState<string | null>(null)
  const [isDragging, setIsDragging] = useState(false)

  const { data: commitDiff } = useGitDiff(
    selectedCommitHash ? `${selectedCommitHash}^..${selectedCommitHash}` : undefined,
  )

  // ── Auto-open canvas.html on first load ──
  useEffect(() => {
    if (openTabs.length > 0 || files.length === 0) return
    const canvasFile = files.find((f) => f.name === 'canvas.html')
    if (canvasFile) {
      setOpenTabs([{ path: canvasFile.path, name: canvasFile.name, type: canvasFile.type }])
      setActiveTabPath(canvasFile.path)
    }
  }, [files]) // eslint-disable-line react-hooks/exhaustive-deps

  // ── Derived ──
  const activeTab = openTabs.find((t) => t.path === activeTabPath) ?? null
  const activeFile = activeTabPath ? files.find((f) => f.path === activeTabPath) : null
  const { data: fileContent } = useSessionFileContent(activeTabPath)

  // Initialize draft when server content arrives for open file
  useEffect(() => {
    if (activeTabPath && fileContent != null) {
      drafts.openFile(activeTabPath, fileContent)
    }
  }, [activeTabPath, fileContent]) // eslint-disable-line react-hooks/exhaustive-deps

  // Cmd+S to save active file
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault()
        if (activeTabPath && drafts.isDirty(activeTabPath)) {
          drafts.saveFile(activeTabPath)
        }
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [activeTabPath, drafts])

  // ── Callbacks ──
  const openFile = useCallback((path: string) => {
    const file = files.find((f) => f.path === path)
    if (!file) return
    setSelectedCommitHash(null)
    setViewMode('file')
    setActiveTabPath(path)
    setOpenTabs((prev) => {
      if (prev.some((t) => t.path === path)) return prev
      return [...prev, { path, name: file.name, type: file.type }]
    })
  }, [files])

  const closeTab = useCallback((path: string) => {
    setOpenTabs((prev) => {
      const next = prev.filter((t) => t.path !== path)
      if (activeTabPath === path) {
        setActiveTabPath(next.length > 0 ? next[next.length - 1].path : null)
      }
      return next
    })
  }, [activeTabPath])

  const selectTab = useCallback((path: string) => {
    setActiveTabPath(path)
    setSelectedCommitHash(null)
    setViewMode('file')
  }, [])

  const handleExport = useCallback(async () => {
    const content = JSON.stringify(ann.toExportJSON(), null, 2)
    if (mcp && isConnected) {
      try {
        await mcp.callTool('write_session_file', { path: 'annotations.json', content })
        return
      } catch { /* fall through */ }
    }
    const blob = new Blob([content], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'annotations.json'
    a.click()
    URL.revokeObjectURL(url)
  }, [ann, mcp, isConnected])

  const handleUploadFiles = useCallback((fileList: FileList) => {
    for (let i = 0; i < fileList.length; i++) uploadMutation.mutate(fileList[i])
  }, [uploadMutation])

  const handleShowDiff = useCallback(() => {
    setSelectedCommitHash(null)
    setViewMode('diff')
  }, [])

  const handleSelectCommit = useCallback((hash: string) => {
    setSelectedCommitHash(hash)
    setViewMode('diff')
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    if (e.dataTransfer.types.includes('Files')) setIsDragging(true)
  }, [])
  const handleDragLeave = useCallback((e: React.DragEvent) => {
    if (e.currentTarget === e.target || !e.currentTarget.contains(e.relatedTarget as Node)) setIsDragging(false)
  }, [])
  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
    if (!isConnected || !e.dataTransfer.files.length) return
    handleUploadFiles(e.dataTransfer.files)
  }, [isConnected, handleUploadFiles])

  // ── View routing ──
  const isHtml = activeTab?.type === 'html'
  const showCanvas = viewMode === 'file' && isHtml
  const showArtifact = viewMode === 'file' && activeFile != null && !isHtml
  const showDiff = viewMode === 'diff'
  const activeDiffText = selectedCommitHash ? commitDiff : diffText

  return (
    <>
      {/* Mobile fallback */}
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border bg-muted/40">
          <Layers className="size-5 text-muted-foreground" />
        </div>
        <p className="font-display text-base font-semibold">Best on desktop</p>
        <p className="text-sm text-muted-foreground max-w-xs">
          The Session viewer is a multi-panel canvas. Open it on a wider screen for the full experience.
        </p>
      </div>

      {/* Desktop layout */}
      <div
        className="hidden md:flex flex-1 flex-col h-full min-h-0 overflow-hidden relative"
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {!isConnected && (
          <div className="flex items-center gap-2 px-4 py-1.5 border-b border-amber-500/30 bg-amber-500/10 text-[11px] text-amber-600 dark:text-amber-400 shrink-0">
            <WifiOff className="size-3 shrink-0" />
            CLI disconnected — session artifacts require a running CLI
          </div>
        )}

        <div className="flex flex-1 min-h-0">
          {/* Left sidebar */}
          <SessionSidebar
            files={files}
            activeFile={activeTabPath}
            annotations={ann.annotations}
            isConnected={isConnected ?? false}
            onSelectFile={openFile}
            onUploadFiles={handleUploadFiles}
            onShowDiff={handleShowDiff}
            onSelectCommit={handleSelectCommit}
            gitStatus={gitStatus}
            gitLog={gitLog}
          />

          {/* Center content */}
          <div className="flex-1 flex flex-col min-w-0 min-h-0">
            {/* Tab bar — always visible, manages ALL open files */}
            <div className="flex items-center border-b border-border bg-card/20 px-1 h-9 shrink-0 overflow-x-auto">
              {openTabs.map((tab) => {
                const isActive = viewMode === 'file' && tab.path === activeTabPath
                const unsaved = drafts.unsavedPaths.has(tab.path)
                return (
                  <button
                    key={tab.path}
                    onClick={() => selectTab(tab.path)}
                    className={`group flex items-center gap-1.5 px-3 py-1.5 text-xs whitespace-nowrap border-b-2 transition-colors ${
                      isActive
                        ? 'border-primary text-foreground font-medium'
                        : 'border-transparent text-muted-foreground hover:text-foreground'
                    }`}
                  >
                    {unsaved && <span className="size-1.5 rounded-full bg-primary shrink-0" title="Unsaved" />}
                    <span className="truncate max-w-[140px]">{tab.name}</span>
                    <span
                      onClick={(e) => { e.stopPropagation(); closeTab(tab.path) }}
                      className="size-4 rounded flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-destructive/10 hover:text-destructive transition"
                    >
                      <X className="size-3" />
                    </span>
                  </button>
                )
              })}

              {/* Diff pseudo-tab */}
              {showDiff && (
                <button
                  className="flex items-center gap-1.5 px-3 py-1.5 text-xs whitespace-nowrap border-b-2 border-primary text-foreground font-medium"
                >
                  {selectedCommitHash ? `Diff ${selectedCommitHash.slice(0, 7)}` : 'Diff'}
                </button>
              )}

              {openTabs.length === 0 && !showDiff && (
                <span className="px-3 py-1.5 text-xs text-muted-foreground/40">No files open</span>
              )}
            </div>

            {/* Content area */}
            <div className="flex-1 flex flex-col min-h-0 min-w-0">
              {showCanvas && (
                <SessionCanvas
                  htmlContent={fileContent ?? ''}
                  fileType={activeFile?.type}
                  annotations={ann.annotations}
                  activeId={ann.activeId}
                  annotationMode={ann.annotationMode}
                  openTabs={activeTabPath ? [activeTabPath] : []}
                  activeTab={activeTabPath}
                  onTabSelect={selectTab}
                  onTabClose={closeTab}
                  onAnnotationClick={ann.toggleActiveId}
                  onDismissActive={ann.dismissActive}
                  onRemoveAnnotation={ann.removeAnnotation}
                  onAddClick={ann.addClickAnnotation}
                  onAddBox={ann.addBoxAnnotation}
                  onAddAction={ann.addActionAnnotation}
                  onToggleAnnotationMode={() => ann.setAnnotationMode(!ann.annotationMode)}
                  onClearAnnotations={ann.clearAnnotations}
                  onExport={handleExport}
                />
              )}
              {showArtifact && activeFile && (
                <ArtifactViewer
                  file={activeFile}
                  content={fileContent ?? ''}
                  draftContent={drafts.getDraft(activeFile.path)}
                  isDirty={drafts.isDirty(activeFile.path)}
                  onContentChange={drafts.updateContent}
                  onSave={drafts.saveFile}
                />
              )}
              {showDiff && (
                activeDiffText ? (
                  <DiffViewer diffText={activeDiffText} />
                ) : (
                  <div className="flex items-center justify-center h-full text-muted-foreground">
                    <div className="text-center">
                      <p className="text-sm font-medium">No diff available</p>
                      <p className="text-xs mt-1">
                        {selectedCommitHash ? `Loading ${selectedCommitHash.slice(0, 7)}...` : 'No changes to show'}
                      </p>
                    </div>
                  </div>
                )
              )}
              {!showCanvas && !showArtifact && !showDiff && (
                <div className="flex items-center justify-center h-full text-muted-foreground">
                  <div className="text-center">
                    <Layers className="size-8 mx-auto mb-3 opacity-40" />
                    <p className="text-sm font-medium">No file open</p>
                    <p className="text-xs mt-1">Select a file from the sidebar</p>
                  </div>
                </div>
              )}
            </div>
          </div>
        </div>

        {isDragging && isConnected && <DropZoneOverlay />}
      </div>
    </>
  )
}
