import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useEffect } from 'react'
import { Layers, PanelRight, WifiOff } from 'lucide-react'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { ArtifactViewer } from '#/features/studio/session/ArtifactViewer'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { SessionSidebar } from '#/features/studio/session/SessionSidebar'
import { SessionInfoPanel } from '#/features/studio/session/SessionInfoPanel'
import { useSessionFiles, useSessionFileContent, useUploadSessionFile } from '#/features/studio/session/useSessionFiles'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
import { useGitStatus, useGitLog, useGitDiff } from '#/features/studio/session/useGitInfo'
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
  const [selectedCommitHash, setSelectedCommitHash] = useState<string | null>(null)

  // Commit-specific diff (only fetches when a commit is selected)
  const { data: commitDiff } = useGitDiff(
    selectedCommitHash ? `${selectedCommitHash}^..${selectedCommitHash}` : undefined,
  )

  // ── Auto-open canvas.html on first load ──
  useEffect(() => {
    if (openCanvasTabs.length > 0 || files.length === 0) return
    const canvasFile = files.find((f) => f.name === 'canvas.html')
    if (canvasFile) {
      setOpenCanvasTabs([canvasFile.path])
      setActiveCanvasTab(canvasFile.path)
      setActiveFilePath(canvasFile.path)
    }
  }, [files]) // eslint-disable-line react-hooks/exhaustive-deps -- run when files first load

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
    setSelectedCommitHash(null)
    const file = files.find((f) => f.path === path)
    if (file?.type === 'html') {
      // Only HTML goes through SessionCanvas (iframe + annotations)
      setOpenCanvasTabs((prev) => prev.includes(path) ? prev : [...prev, path])
      setActiveCanvasTab(path)
      setViewMode('canvas')
    } else {
      // Markdown, images, JSON, text → ArtifactViewer
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
    setSelectedCommitHash(null)
    setViewMode('canvas')
  }, [])

  const handleUploadFiles = useCallback((fileList: FileList) => {
    for (let i = 0; i < fileList.length; i++) {
      uploadMutation.mutate(fileList[i])
    }
  }, [uploadMutation])

  const handleShowDiff = useCallback(() => {
    setSelectedCommitHash(null) // show working-tree diff
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

  // ── View routing ──
  // Canvas = HTML only (iframe + annotations). Everything else = ArtifactViewer.
  const showCanvas = viewMode === 'canvas' && (activeFileType == null || activeFileType === 'html')
  const showDiff = viewMode === 'diff'
  const showArtifactViewer = viewMode === 'artifact' && activeFile != null
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

      {/* Desktop 3-panel layout */}
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
            activeFile={effectivePath}
            annotations={ann.annotations}
            isConnected={isConnected ?? false}
            onSelectFile={handleSelectFile}
            onUploadFiles={handleUploadFiles}
            onShowDiff={handleShowDiff}
            onSelectCommit={handleSelectCommit}
            gitStatus={gitStatus}
            gitLog={gitLog}
          />

          {/* Center: SessionCanvas handles its own tabs + toolbar */}
          <div className="flex-1 flex flex-col min-w-0 min-h-0">
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
              activeDiffText ? (
                <DiffViewer diffText={activeDiffText} />
              ) : (
                <div className="flex items-center justify-center h-full text-muted-foreground">
                  <div className="text-center">
                    <p className="text-sm font-medium">No diff available</p>
                    <p className="text-xs mt-1 max-w-[280px]">
                      {selectedCommitHash
                        ? `Loading diff for ${selectedCommitHash.slice(0, 7)}...`
                        : <>Run <code className="text-[10px] bg-muted px-1 rounded">/diff</code> to generate one</>
                      }
                    </p>
                  </div>
                </div>
              )
            )}
            {showArtifactViewer && activeFile && (
              <ArtifactViewer file={activeFile} content={fileContent ?? ''} />
            )}
            {!showCanvas && !showDiff && !showArtifactViewer && (
              <div className="flex items-center justify-center h-full text-muted-foreground">
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

          {/* Right panel */}
          {infoPanelOpen ? (
            <SessionInfoPanel
              gitStatus={gitStatus}
              gitLog={gitLog}
              onClose={() => setInfoPanelOpen(false)}
              onShowDiff={handleShowDiff}
              onSelectCommit={handleSelectCommit}
            />
          ) : (
            <button
              onClick={() => setInfoPanelOpen(true)}
              className="shrink-0 border-l border-border px-2 py-3 text-muted-foreground hover:text-foreground hover:bg-muted/30 transition"
              aria-label="Open session panel"
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
