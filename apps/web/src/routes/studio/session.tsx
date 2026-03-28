import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { Layers, PanelRight, WifiOff } from 'lucide-react'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { SessionActivity } from '#/features/studio/session/SessionActivity'
import { ArtifactViewer } from '#/features/studio/session/ArtifactViewer'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { useSessionFiles, useSessionFileContent, useDeleteSessionFile, useUploadSessionFile } from '#/features/studio/session/useSessionFiles'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
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

  const { files, isLoading } = useSessionFiles()
  const [activeFilePath, setActiveFilePath] = useState<string | null>(null)
  const [activityOpen, setActivityOpen] = useState(true)
  const [isDragging, setIsDragging] = useState(false)
  const [openCanvasTabs, setOpenCanvasTabs] = useState<string[]>([])
  const [activeCanvasTab, setActiveCanvasTab] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<ViewMode>('canvas')

  const deleteMutation = useDeleteSessionFile()
  const uploadMutation = useUploadSessionFile()
  const { diffText } = useDiffContent()

  const effectivePath = activeFilePath ?? files.find((f) => f.name === 'canvas.html')?.path ?? null
  const activeFile = effectivePath ? files.find((f) => f.path === effectivePath) : null
  const activeFileType = activeFile?.type ?? null
  const { data: fileContent } = useSessionFileContent(effectivePath)
  const ann = useAnnotations()

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

  const handleDeleteFile = useCallback((path: string) => {
    deleteMutation.mutate(path)
    if (activeFilePath === path) setActiveFilePath(null)
  }, [deleteMutation, activeFilePath])

  const handleUploadFiles = useCallback((fileList: FileList) => {
    for (let i = 0; i < fileList.length; i++) {
      const file = fileList[i]
      uploadMutation.mutate(file)
    }
  }, [uploadMutation])

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
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border bg-muted/40">
          <Layers className="size-5 text-muted-foreground" />
        </div>
        <p className="font-display text-base font-semibold">Best on desktop</p>
        <p className="text-sm text-muted-foreground max-w-xs">
          The Session viewer is a multi-panel canvas. Open it on a wider screen for the full experience.
        </p>
      </div>

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
          {/* Main area: canvas, diff, or artifact viewer */}
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
            <div className="flex flex-1 flex-col min-h-0 min-w-0">
              {diffText ? (
                <DiffViewer diffText={diffText} />
              ) : (
                <div className="flex items-center justify-center h-full text-muted-foreground">
                  <div className="text-center">
                    <p className="text-sm font-medium">No diff available</p>
                    <p className="text-xs mt-1 max-w-[280px]">
                      Run <code className="text-[10px] bg-muted px-1 rounded">/diff</code> to generate one,
                      or have an agent write to .ship-session/diff.txt
                    </p>
                  </div>
                </div>
              )}
            </div>
          )}
          {showArtifactViewer && activeFile && (
            <div className="flex flex-1 flex-col min-h-0 min-w-0">
              <ArtifactViewer file={activeFile} content={fileContent ?? ''} />
            </div>
          )}
          {/* Fallback when nothing matches (e.g. artifact mode with no file) */}
          {!showCanvas && !showDiff && !showArtifactViewer && (
            <div className="flex flex-1 items-center justify-center text-muted-foreground">
              <div className="text-center">
                <Layers className="size-8 mx-auto mb-3 opacity-40" />
                <p className="text-sm font-medium">Select a file or view</p>
                <p className="text-xs mt-1 max-w-[280px]">
                  Use the sidebar to open a canvas, view a diff, or browse artifacts.
                </p>
              </div>
            </div>
          )}

          {activityOpen ? (
            <SessionActivity
              files={files}
              activeFile={effectivePath}
              isLoading={isLoading}
              isConnected={isConnected ?? false}
              annotations={ann.annotations}
              viewMode={viewMode}
              hasDiff={diffText != null}
              openCanvasTabs={openCanvasTabs}
              activeCanvasTab={activeCanvasTab}
              annotationMode={ann.annotationMode}
              onSelectFile={handleSelectFile}
              onDeleteFile={handleDeleteFile}
              onUploadFiles={handleUploadFiles}
              onRemoveAnnotation={ann.removeAnnotation}
              onExportAnnotations={handleExport}
              onClose={() => setActivityOpen(false)}
              onSetViewMode={setViewMode}
              onSelectCanvasTab={handleSelectCanvasTab}
              onToggleAnnotationMode={() => ann.setAnnotationMode(!ann.annotationMode)}
            />
          ) : (
            <button
              onClick={() => setActivityOpen(true)}
              className="shrink-0 border-l border-border/60 px-2 py-3 text-muted-foreground hover:text-foreground hover:bg-muted/30 transition"
              aria-label="Open activity panel"
            >
              <PanelRight className="size-4" />
            </button>
          )}
        </div>

        {isDragging && isConnected && <DropZoneOverlay />}
      </div>
    </>
  )
}
