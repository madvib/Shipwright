import { createFileRoute } from '@tanstack/react-router'
import { useState, useCallback } from 'react'
import { Layers, PanelRight, WifiOff } from 'lucide-react'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { SessionCanvas } from '#/features/studio/session/SessionCanvas'
import { SessionTimeline } from '#/features/studio/session/SessionTimeline'
import { useSessionFiles, useSessionFileContent } from '#/features/studio/session/useSessionFiles'
import { useAnnotations } from '#/features/studio/session/useAnnotations'
import { SessionSkeleton } from '#/features/studio/session/SessionSkeleton'

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
  const [timelineOpen, setTimelineOpen] = useState(true)

  // Default to canvas.html if it exists and nothing is selected
  const effectivePath = activeFilePath ?? files.find((f) => f.name === 'canvas.html')?.path ?? null
  const { data: htmlContent } = useSessionFileContent(
    effectivePath && files.find((f) => f.path === effectivePath)?.type === 'html' ? effectivePath : null,
  )

  const ann = useAnnotations()

  const handleExport = useCallback(() => {
    const json = ann.toExportJSON()
    const blob = new Blob([JSON.stringify(json, null, 2)], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'annotations.json'
    a.click()
    URL.revokeObjectURL(url)
  }, [ann])

  const handleSelectFile = useCallback((path: string) => {
    setActiveFilePath(path)
  }, [])

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
      <div className="hidden md:flex flex-1 flex-col h-full min-h-0 overflow-hidden">
        {!isConnected && (
          <div className="flex items-center gap-2 px-4 py-1.5 border-b border-amber-500/30 bg-amber-500/10 text-[11px] text-amber-600 dark:text-amber-400 shrink-0">
            <WifiOff className="size-3 shrink-0" />
            CLI disconnected — session files require a running CLI
          </div>
        )}

        <div className="flex flex-1 min-h-0">
          <SessionCanvas
            htmlContent={htmlContent ?? ''}
            annotations={ann.annotations}
            activeId={ann.activeId}
            annotationMode={ann.annotationMode}
            onAnnotationClick={ann.setActiveId}
            onRemoveAnnotation={ann.removeAnnotation}
            onAddClick={ann.addClickAnnotation}
            onAddBox={ann.addBoxAnnotation}
            onToggleAnnotationMode={() => ann.setAnnotationMode(!ann.annotationMode)}
            onClearAnnotations={ann.clearAnnotations}
            onExport={handleExport}
          />

          {timelineOpen ? (
            <SessionTimeline
              files={files}
              activeFile={effectivePath}
              isLoading={isLoading}
              isConnected={isConnected ?? false}
              onSelectFile={handleSelectFile}
              onClose={() => setTimelineOpen(false)}
            />
          ) : (
            <button
              onClick={() => setTimelineOpen(true)}
              className="shrink-0 border-l border-border/60 px-2 py-3 text-muted-foreground hover:text-foreground hover:bg-muted/30 transition"
              aria-label="Open timeline"
            >
              <PanelRight className="size-4" />
            </button>
          )}
        </div>
      </div>
    </>
  )
}
