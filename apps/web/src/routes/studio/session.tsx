import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { Layers, PanelRight, WifiOff } from 'lucide-react'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { SessionActivity } from '#/features/studio/session/SessionActivity'
import { ArtifactViewer } from '#/features/studio/session/ArtifactViewer'
import { useSessionFiles, useSessionFileContent, useDeleteSessionFile, useUploadSessionFile } from '#/features/studio/session/useSessionFiles'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { SessionSkeleton } from '#/features/studio/session/SessionSkeleton'
import { DropZoneOverlay } from '#/features/studio/session/DropZoneOverlay'

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

  const deleteMutation = useDeleteSessionFile()
  const uploadMutation = useUploadSessionFile()

  // Default to canvas.html if it exists and nothing is selected
  const effectivePath = activeFilePath ?? files.find((f) => f.name === 'canvas.html')?.path ?? null
  const activeFile = effectivePath ? files.find((f) => f.path === effectivePath) : null
  const activeFileType = activeFile?.type ?? null

  // Fetch content for the active file (all types — images return as base64 data URI)
  const { data: fileContent } = useSessionFileContent(effectivePath)

  const ann = useAnnotations()

  // Export annotations to .ship-session/annotations.json via MCP, with
  // a fallback to browser download if MCP is not connected.
  const handleExport = useCallback(async () => {
    const json = ann.toExportJSON()
    const content = JSON.stringify(json, null, 2)

    if (mcp && isConnected) {
      try {
        await mcp.callTool('write_session_file', { path: 'annotations.json', content })
        return
      } catch {
        // Fall through to browser download
      }
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

  const showCanvas = activeFileType === 'html' || activeFileType == null
  const showArtifactViewer = activeFile != null && activeFileType !== 'html'

  return (
    <>
      {/* Mobile fallback */}
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border bg-muted/40">
          <Layers className="size-5 text-muted-foreground" />
        </div>
        <div>
          <p className="font-display text-base font-semibold">Best on desktop</p>
          <p className="mt-1 text-sm text-muted-foreground max-w-xs">
            The Session viewer is a multi-panel canvas. Open it on a wider screen for the full experience.
          </p>
        </div>
      </div>

      {/* Full layout */}
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
          {/* Main area: canvas or artifact viewer */}
          {showCanvas && (
            <SessionCanvas
              htmlContent={fileContent ?? ''}
              annotations={ann.annotations}
              activeId={ann.activeId}
              annotationMode={ann.annotationMode}
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
          {showArtifactViewer && (
            <div className="flex flex-1 flex-col min-h-0 min-w-0">
              <ArtifactViewer file={activeFile} content={fileContent ?? ''} />
            </div>
          )}

          {activityOpen ? (
            <SessionActivity
              files={files}
              activeFile={effectivePath}
              isLoading={isLoading}
              isConnected={isConnected ?? false}
              annotations={ann.annotations}
              onSelectFile={handleSelectFile}
              onDeleteFile={handleDeleteFile}
              onUploadFiles={handleUploadFiles}
              onRemoveAnnotation={ann.removeAnnotation}
              onExportAnnotations={handleExport}
              onClose={() => setActivityOpen(false)}
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
