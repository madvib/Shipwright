import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback, useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Layers } from 'lucide-react'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { ArtifactViewer } from '#/features/studio/session/ArtifactViewer'
import { DiffViewer } from '#/features/studio/session/DiffViewer'
import { SessionSidebar } from '#/features/studio/session/SessionSidebar'
import { useSessionFiles, useSessionFileContent, useUploadSessionFile, useDeleteSessionFile } from '#/features/studio/session/useSessionFiles'
import { useSessionDrafts } from '#/features/studio/session/useSessionDrafts'
import { SessionTabBar } from '#/features/studio/session/SessionTabBar'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { RightDrawer } from '#/features/studio/session/RightDrawer'
import { useSessionHandlers } from '#/features/studio/session/useSessionHandlers'
import { useDiffContent } from '#/features/studio/session/useDiffContent'
import { useGitStatus, useGitLog, useGitDiff } from '#/features/studio/session/useGitInfo'
import { SessionSkeleton } from '#/features/studio/session/SessionSkeleton'
import { DropZoneOverlay } from '#/features/studio/session/DropZoneOverlay'
import { ViewHost } from '#/features/studio/views/ViewHost'

export const Route = createFileRoute('/studio/session')({
  component: SessionPage,
  pendingComponent: SessionSkeleton,
  ssr: false,
})

type ViewMode = 'file' | 'diff'
interface OpenTab { path: string; name: string; type: string }

function SessionPage() {
  const { workspaces } = useDaemon()
  const active = workspaces
    .filter((w) => w.status === 'active')
    .sort((a, b) => (b.last_activated_at ?? '').localeCompare(a.last_activated_at ?? ''))
  const workspaceId = active[0]?.branch ?? ''

  const { files } = useSessionFiles()
  const uploadMutation = useUploadSessionFile()
  const deleteMutation = useDeleteSessionFile()
  const { diffText } = useDiffContent()
  const { data: gitStatus } = useGitStatus()
  const { data: gitLog } = useGitLog(5)
  const drafts = useSessionDrafts()

  const [openTabs, setOpenTabs] = useState<OpenTab[]>([])
  const [activeTabPath, setActiveTabPath] = useState<string | null>(null)
  const [viewMode, setViewMode] = useState<ViewMode>('file')
  const [selectedCommitHash, setSelectedCommitHash] = useState<string | null>(null)
  const [activeView, setActiveView] = useState<string | null>(null)

  // Fetch view HTML when a view is active
  const { data: viewHtml } = useQuery({
    queryKey: ['view', activeView],
    queryFn: async () => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/view/${activeView}`)
      if (!res.ok) return null
      const body = await res.json() as { ok: boolean; data: { html: string } }
      return body.data.html
    },
    enabled: activeView != null,
    staleTime: 60_000,
  })

  const ann = useAnnotations(activeTabPath)
  const { data: commitDiff } = useGitDiff(
    selectedCommitHash ? `${selectedCommitHash}^..${selectedCommitHash}` : undefined,
  )

  // Auto-open canvas.html on first load
  useEffect(() => {
    if (openTabs.length > 0 || files.length === 0) return
    const canvasFile = files.find((f) => f.name === 'canvas.html')
    if (canvasFile) {
      setOpenTabs([{ path: canvasFile.path, name: canvasFile.name, type: canvasFile.type }])
      setActiveTabPath(canvasFile.path)
    }
  }, [files]) // eslint-disable-line react-hooks/exhaustive-deps

  const activeTab = openTabs.find((t) => t.path === activeTabPath) ?? null
  const activeFile = activeTabPath ? files.find((f) => f.path === activeTabPath) : null
  const { data: fileContent } = useSessionFileContent(activeTabPath)

  useEffect(() => {
    if (activeTabPath && fileContent != null) drafts.openFile(activeTabPath, fileContent)
  }, [activeTabPath, fileContent]) // eslint-disable-line react-hooks/exhaustive-deps

  // Cmd+S to save
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault()
        if (activeTabPath && drafts.isDirty(activeTabPath)) drafts.saveFile(activeTabPath)
      }
    }
    window.addEventListener('keydown', handler)
    return () => window.removeEventListener('keydown', handler)
  }, [activeTabPath, drafts])

  const closeTab = useCallback((path: string) => {
    setOpenTabs((prev) => {
      const next = prev.filter((t) => t.path !== path)
      if (activeTabPath === path) setActiveTabPath(next.length > 0 ? next[next.length - 1].path : null)
      return next
    })
  }, [activeTabPath])

  const openFile = useCallback((path: string) => {
    const file = files.find((f) => f.path === path)
    if (!file) return
    setSelectedCommitHash(null)
    setViewMode('file')
    setActiveTabPath(path)
    setOpenTabs((prev) => {
      if (prev.some((t) => t.path === path)) return prev
      if (activeTabPath) return prev.map((t) => t.path === activeTabPath ? { path, name: file.name, type: file.type } : t)
      return [...prev, { path, name: file.name, type: file.type }]
    })
  }, [files, activeTabPath])

  const selectTab = useCallback((path: string) => {
    setActiveTabPath(path)
    setSelectedCommitHash(null)
    setViewMode('file')
    setActiveView(null)
  }, [])

  const handleOpenView = useCallback((viewName: string) => {
    setActiveView(viewName)
    setViewMode('file')
  }, [])

  const {
    isDragging, handleDeleteFile, handleExport, handleComment, handleDiffComment,
    handleUploadFiles, handleShowDiff, handleSelectCommit,
    handleSendToAgent, handleDragOver, handleDragLeave, handleDrop,
  } = useSessionHandlers({
    workspaceId,
    ann, openFile, closeTab,
    setViewMode, setSelectedCommitHash,
    uploadMutate: uploadMutation.mutate,
    deleteMutate: deleteMutation.mutate,
  })

  const isHtml = activeTab?.type === 'html'
  const showView = activeView != null && viewHtml != null
  const showCanvas = !showView && viewMode === 'file' && isHtml
  const showArtifact = !showView && viewMode === 'file' && activeFile != null && !isHtml
  const showDiff = !showView && viewMode === 'diff'
  const activeDiffText = selectedCommitHash ? commitDiff : diffText

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
        <div className="flex flex-1 min-h-0">
          <SessionSidebar
            files={files}
            activeFile={activeTabPath}
            onSelectFile={openFile}
            onDeleteFile={handleDeleteFile}
            onUploadFiles={handleUploadFiles}
            onShowDiff={handleShowDiff}
            onSelectCommit={handleSelectCommit}
            onOpenView={handleOpenView}
            gitStatus={gitStatus}
            gitLog={gitLog}
          />

          <div className="flex-1 flex min-w-0 min-h-0">
            <div className="flex-1 flex flex-col min-w-0 min-h-0">
              <SessionTabBar
                openTabs={openTabs}
                activeTabPath={activeTabPath}
                viewMode={viewMode}
                unsavedPaths={drafts.unsavedPaths}
                selectedCommitHash={selectedCommitHash}
                onSelectTab={selectTab}
                onCloseTab={closeTab}
              />

              <div className="flex-1 flex flex-col min-h-0 min-w-0">
                {showView && (
                  <ViewHost html={viewHtml} />
                )}
                {showCanvas && (
                  <SessionCanvas
                    key={activeTabPath}
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
                    onComment={handleComment}
                  />
                )}
                {showDiff && (
                  activeDiffText
                    ? <DiffViewer diffText={activeDiffText} onComment={handleDiffComment} />
                    : (
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
                {!showView && !showCanvas && !showArtifact && !showDiff && (
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

            <RightDrawer
              stagedAnnotations={ann.allStaged}
              onSend={handleSendToAgent}
              onRemoveAnnotation={ann.removeAnnotation}
              onUploadFiles={handleUploadFiles}
              disabled={false}
            />
          </div>
        </div>

        {isDragging && <DropZoneOverlay />}
      </div>
    </>
  )
}
