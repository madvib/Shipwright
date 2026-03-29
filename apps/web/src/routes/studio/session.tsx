import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useEffect } from 'react'
import { Layers, PanelRight, WifiOff, Maximize2, Minimize2 } from 'lucide-react'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { ArtifactViewer } from '#/features/studio/session/ArtifactViewer'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { SessionSidebar } from '#/features/studio/session/SessionSidebar'
import { SessionInfoPanel } from '#/features/studio/session/SessionInfoPanel'
import { useSessionFiles, useSessionFileContent, useUploadSessionFile } from '#/features/studio/session/useSessionFiles'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
import { useGitStatus, useGitLog } from '#/features/studio/session/useGitInfo'
import { SessionSkeleton } from '#/features/studio/session/SessionSkeleton'
import { DropZoneOverlay } from '#/features/studio/session/DropZoneOverlay'
import type { ViewMode } from '#/features/studio/session/types'

export const Route = createFileRoute('/studio/session')({
  component: SessionPage,
  pendingComponent: SessionSkeleton,
  ssr: false,
})

function SessionPage() {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'

  // ── All hooks called unconditionally at the top level ──
  const { files } = useSessionFiles()
  const uploadMutation = useUploadSessionFile()
  const { diffText } = useDiffContent()
  const { data: gitStatus } = useGitStatus()
  const { data: gitLog } = useGitLog(5)
  const ann = useAnnotations()

  // ── UI state ──
  const [activeFilePath, setActiveFilePath] = useState<string | null>(null)
  const [infoPanelOpen, setInfoPanelOpen] = useState(true)
  const [isDragging, setIsDragging] = useState(false)
  const [openCanvasTabs, setOpenCanvasTabs] = useState<string[]>([])
  const [activeCanvasTab, setActiveCanvasTab] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<ViewMode>('canvas')
  const [isFullscreen, setIsFullscreen] = useState(false)

  // ── Auto-open canvas.html on first load ──
  useEffect(() => {
    if (openCanvasTabs.length > 0 || files.length === 0) return
    const canvasFile = files.find((f) => f.name === 'canvas.html')
    if (canvasFile) {
      setOpenCanvasTabs([canvasFile.path])
      setActiveCanvasTab(canvasFile.path)
      setActiveFilePath(canvasFile.path)
    }
  }, [files]) // eslint-disable-line react-hooks/exhaustive-deps -- intentional: run when files first load

  // ── Derived state ──
  const effectivePath = activeFilePath ?? files.find((f) => f.name === 'canvas.html')?.path ?? null
  const activeFile = effectivePath ? files.find((f) => f.path === effectivePath) : null
  const activeFileType = activeFile?.type ?? null
  const { data: fileContent } = useSessionFileContent(effectivePath)

  // ── Callbacks ──
  const handleExport = useCallback(async () => {
    const content = JSON.stringify(ann.toExportJSON(), null, 2)
    if (mcp && isConnected) {
      try {
        await mcp.callTool('write_session_file', { path: 'annotations.json', content })
        return
      } catch { /* Fall through to browser download */ }
    }
    const blob = new Blob([content], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'annotations.json'
    a.click()
    URL.revokeObjectURL(url)
  }, [ann, mcp, isConnected])

  const handleSelectFile = useCallback((path: string) => {
    setActiveFilePath(path)
    const file = files.find((f) => f.path === path)
    const canvasFileTypes = new Set(['html', 'markdown', 'image'])
    if (file && canvasFileTypes.has(file.type)) {
      if (file.type === 'html') {
        setOpenCanvasTabs((prev) => prev.includes(path) ? prev : [...prev, path])
        setActiveCanvasTab(path)
      }
      setViewMode('canvas')
    } else {
      setViewMode('artifact')
    }
  }, [files])

  const handleCloseCanvasTab = useCallback((path: string) => {
    setOpenCanvasTabs((prev) => {
      const next = prev.filter((p) => p !== path)
      if (activeCanvasTab === path) {
        setActiveCanvasTab(next.length > 0 ? next[next.length - 1] : null)
        if (next.length === 0) setActiveFilePath(null)
      }
      return next
    })
  }, [activeCanvasTab])

  const handleSelectCanvasTab = useCallback((path: string) => {
    setActiveCanvasTab(path)
    setActiveFilePath(path)
    setViewMode('canvas')
  }, [])

  const handleUploadFiles = useCallback((fileList: FileList) => {
    for (let i = 0; i < fileList.length; i++) {
      uploadMutation.mutate(fileList[i])
    }
  }, [uploadMutation])

  const handleShowDiff = useCallback(() => {
    setViewMode('diff')
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    if (e.dataTransfer.types.includes('Files')) setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    if (e.currentTarget === e.target || !e.currentTarget.contains(e.relatedTarget as Node)) {
      setIsDragging(false)
    }
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
    if (!isConnected || !e.dataTransfer.files.length) return
    handleUploadFiles(e.dataTransfer.files)
  }, [isConnected, handleUploadFiles])

  const canvasTypes = new Set(['html', 'markdown', 'image'])
  const showCanvas = viewMode === 'canvas' && (activeFileType == null || canvasTypes.has(activeFileType ?? ''))
  const showDiff = viewMode === 'diff'
  const showArtifactViewer = viewMode === 'artifact' && activeFile != null && !canvasTypes.has(activeFileType ?? '')

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
          {!isFullscreen && (
            <SessionSidebar
              files={files}
              activeFile={effectivePath}
              annotations={ann.annotations}
              isConnected={isConnected ?? false}
              onSelectFile={handleSelectFile}
              onUploadFiles={handleUploadFiles}
              gitStatus={gitStatus}
              gitLog={gitLog}
            />
          )}

          {/* Center: content area */}
          <main className="flex-1 flex flex-col min-w-0">
            {/* Tab bar */}
            <div className="flex items-center border-b border-border bg-card/20 px-2 h-9 shrink-0 overflow-x-auto">
              {openCanvasTabs.map((tabPath) => {
                const tabFile = files.find((f) => f.path === tabPath)
                const isActive = viewMode === 'canvas' && activeCanvasTab === tabPath
                return (
                  <button
                    key={tabPath}
                    onClick={() => handleSelectCanvasTab(tabPath)}
                    className={`group flex items-center gap-1.5 px-3 py-1.5 text-xs whitespace-nowrap border-b-2 transition-colors ${
                      isActive
                        ? 'border-primary text-foreground font-medium'
                        : 'border-transparent text-muted-foreground hover:text-foreground'
                    }`}
                  >
                    {tabFile?.name ?? tabPath.split('/').pop()}
                    <span
                      onClick={(e) => { e.stopPropagation(); handleCloseCanvasTab(tabPath) }}
                      className="size-4 rounded flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-destructive/10 hover:text-destructive transition"
                    >
                      &times;
                    </span>
                  </button>
                )
              })}

              {/* Diff pseudo-tab */}
              {diffText != null && (
                <button
                  onClick={handleShowDiff}
                  className={`flex items-center gap-1.5 px-3 py-1.5 text-xs whitespace-nowrap border-b-2 transition-colors ${
                    showDiff
                      ? 'border-primary text-foreground font-medium'
                      : 'border-transparent text-muted-foreground hover:text-foreground'
                  }`}
                >
                  Diff
                </button>
              )}

              <div className="flex-1" />

              {/* Toolbar */}
              <div className="flex items-center gap-1">
                {showCanvas && (
                  <button
                    onClick={() => ann.setAnnotationMode(!ann.annotationMode)}
                    className={`h-6 px-2 rounded text-[10px] font-medium flex items-center gap-1 transition ${
                      ann.annotationMode
                        ? 'bg-primary/15 text-primary'
                        : 'text-muted-foreground hover:text-foreground hover:bg-muted/30'
                    }`}
                  >
                    Annotate
                  </button>
                )}
                <button
                  onClick={() => setIsFullscreen(!isFullscreen)}
                  className="flex size-6 items-center justify-center rounded text-muted-foreground hover:text-foreground hover:bg-muted/30 transition"
                  title={isFullscreen ? 'Exit fullscreen' : 'Fullscreen'}
                >
                  {isFullscreen ? <Minimize2 className="size-3.5" /> : <Maximize2 className="size-3.5" />}
                </button>
              </div>
            </div>

            {/* Content viewport */}
            <div className="flex-1 min-h-0 min-w-0 relative">
              {showCanvas && (
                <SessionCanvas
                  htmlContent={fileContent ?? ''}
                  fileType={activeFileType}
                  annotations={ann.annotations}
                  activeId={ann.activeId}
                  annotationMode={ann.annotationMode}
                  openTabs={openCanvasTabs}
                  activeTab={activeCanvasTab}
                  onTabSelect={handleSelectCanvasTab}
                  onTabClose={handleCloseCanvasTab}
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
              {showDiff && (
                diffText ? (
                  <DiffViewer diffText={diffText} />
                ) : (
                  <div className="flex items-center justify-center h-full text-muted-foreground">
                    <div className="text-center">
                      <p className="text-sm font-medium">No diff available</p>
                      <p className="text-xs mt-1 max-w-[280px]">
                        Run <code className="text-[10px] bg-muted px-1 rounded">/diff</code> to generate one
                      </p>
                    </div>
                  </div>
                )
              )}
              {showArtifactViewer && activeFile && (
                <ArtifactViewer file={activeFile} content={fileContent ?? ''} />
              )}
              {!showCanvas && !showDiff && !showArtifactViewer && (
                <div className="flex flex-1 items-center justify-center h-full text-muted-foreground">
                  <div className="text-center">
                    <Layers className="size-8 mx-auto mb-3 opacity-40" />
                    <p className="text-sm font-medium">No file open</p>
                    <p className="text-xs mt-1 max-w-[280px]">
                      Select a file from the sidebar to start editing.
                    </p>
                  </div>
                </div>
              )}
            </div>
          </main>

          {/* Right panel */}
          {!isFullscreen && infoPanelOpen ? (
            <SessionInfoPanel
              gitStatus={gitStatus}
              gitLog={gitLog}
              onClose={() => setInfoPanelOpen(false)}
              onShowDiff={handleShowDiff}
            />
          ) : !isFullscreen ? (
            <button
              onClick={() => setInfoPanelOpen(true)}
              className="shrink-0 border-l border-border px-2 py-3 text-muted-foreground hover:text-foreground hover:bg-muted/30 transition"
              aria-label="Open session panel"
            >
              <PanelRight className="size-4" />
            </button>
          ) : null}
        </div>

        {isDragging && isConnected && <DropZoneOverlay />}
      </div>
    </>
  )
}
