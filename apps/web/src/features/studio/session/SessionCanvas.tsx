// Renders an iframe with the session HTML content and an annotation overlay.
// The iframe loads HTML via srcdoc to avoid cross-origin restrictions.

import { useRef } from 'react'
import { MousePointerClick, Trash2, Download, Square } from 'lucide-react'
import { AnnotationOverlay } from './AnnotationOverlay'
import type { Annotation } from './types'

interface SessionCanvasProps {
  htmlContent: string
  annotations: Annotation[]
  activeId: string | null
  annotationMode: boolean
  onAnnotationClick: (id: string) => void
  onRemoveAnnotation: (id: string) => void
  onAddClick: (selector: string, text: string, note: string, x: number, y: number) => void
  onAddBox: (rect: [number, number, number, number], elements: string[], note: string) => void
  onToggleAnnotationMode: () => void
  onClearAnnotations: () => void
  onExport: () => void
}

export function SessionCanvas({
  htmlContent,
  annotations,
  activeId,
  annotationMode,
  onAnnotationClick,
  onRemoveAnnotation,
  onAddClick,
  onAddBox,
  onToggleAnnotationMode,
  onClearAnnotations,
  onExport,
}: SessionCanvasProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null)

  return (
    <div className="flex flex-1 flex-col min-h-0 min-w-0">
      {/* Toolbar */}
      <div className="flex items-center gap-1.5 border-b border-border/60 px-3 py-1.5 shrink-0 bg-card/50">
        <button
          onClick={onToggleAnnotationMode}
          className={`flex items-center gap-1.5 rounded-md px-2.5 py-1 text-[11px] font-medium transition ${
            annotationMode
              ? 'bg-primary text-primary-foreground'
              : 'text-muted-foreground hover:bg-muted hover:text-foreground'
          }`}
        >
          {annotationMode ? (
            <>
              <Square className="size-3" />
              Annotating
            </>
          ) : (
            <>
              <MousePointerClick className="size-3" />
              Annotate
            </>
          )}
        </button>

        {annotations.length > 0 && (
          <>
            <span className="text-[10px] text-muted-foreground tabular-nums">
              {annotations.length} note{annotations.length !== 1 ? 's' : ''}
            </span>
            <div className="flex-1" />
            <button
              onClick={onExport}
              className="flex items-center gap-1 rounded-md px-2 py-1 text-[11px] text-muted-foreground hover:bg-muted hover:text-foreground"
            >
              <Download className="size-3" />
              Export
            </button>
            <button
              onClick={onClearAnnotations}
              className="flex items-center gap-1 rounded-md px-2 py-1 text-[11px] text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
            >
              <Trash2 className="size-3" />
              Clear
            </button>
          </>
        )}
      </div>

      {/* Canvas area */}
      <div className="relative flex-1 min-h-0 overflow-hidden bg-muted/30">
        {htmlContent ? (
          <>
            <iframe
              ref={iframeRef}
              srcDoc={htmlContent}
              title="Session canvas"
              sandbox="allow-same-origin allow-scripts"
              className="absolute inset-0 w-full h-full border-0 bg-white"
            />
            <AnnotationOverlay
              iframeRef={iframeRef}
              annotations={annotations}
              activeId={activeId}
              annotationMode={annotationMode}
              onAnnotationClick={onAnnotationClick}
              onRemoveAnnotation={onRemoveAnnotation}
              onAddClick={onAddClick}
              onAddBox={onAddBox}
            />
          </>
        ) : (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            <div className="text-center">
              <Square className="size-8 mx-auto mb-3 opacity-40" />
              <p className="text-sm font-medium">No canvas loaded</p>
              <p className="text-xs mt-1 max-w-[280px]">
                Select an HTML artifact from the activity panel, or have an agent write to
                .ship-session/canvas.html
              </p>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
