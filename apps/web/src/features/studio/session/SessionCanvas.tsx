// Renders an iframe with the session HTML content and an annotation overlay.
// Theme is synced both via srcdoc injection (initial) and postMessage (updates).

import { useRef, useEffect, useCallback, useMemo } from 'react'
import { MousePointerClick, Trash2, Download, Square, Maximize, X } from 'lucide-react'
import { AnnotationOverlay } from './AnnotationOverlay'
import { getResolvedTheme, injectThemeListener, injectThemeAttribute, wrapContent } from './canvas-helpers'
import type { Annotation } from './types'

interface SessionCanvasProps {
  htmlContent: string
  fileType?: string | null
  annotations: Annotation[]
  activeId: string | null
  annotationMode: boolean
  openTabs: string[]
  activeTab: string | null
  onTabSelect: (path: string) => void
  onTabClose: (path: string) => void
  onAnnotationClick: (id: string) => void
  onDismissActive: () => void
  onRemoveAnnotation: (id: string) => void
  onAddClick: (selector: string, text: string, note: string, x: number, y: number) => void
  onAddBox: (rect: [number, number, number, number], elements: string[], note: string) => void
  onAddAction: (action: string, text: string) => void
  onToggleAnnotationMode: () => void
  onClearAnnotations: () => void
  onExport: () => void
}

export function SessionCanvas({
  htmlContent,
  fileType,
  annotations,
  activeId,
  annotationMode,
  openTabs,
  activeTab,
  onTabSelect,
  onTabClose,
  onAnnotationClick,
  onDismissActive,
  onRemoveAnnotation,
  onAddClick,
  onAddBox,
  onAddAction,
  onToggleAnnotationMode,
  onClearAnnotations,
  onExport,
}: SessionCanvasProps) {
  const iframeRef = useRef<HTMLIFrameElement>(null)

  const themedContent = useMemo(() => {
    if (!htmlContent) return ''
    const wrapped = wrapContent(htmlContent, fileType)
    const withTheme = injectThemeAttribute(wrapped)
    return injectThemeListener(withTheme)
  }, [htmlContent, fileType])

  // Sync theme to iframe on load and when theme changes
  const postTheme = useCallback(() => {
    const iframe = iframeRef.current
    if (!iframe?.contentWindow) return
    iframe.contentWindow.postMessage({ type: 'theme', theme: getResolvedTheme() }, '*')
  }, [])

  useEffect(() => {
    // Observe class changes on <html> to detect theme switches
    const observer = new MutationObserver(() => postTheme())
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'data-theme'] })
    return () => observer.disconnect()
  }, [postTheme])

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
        <button
          onClick={() => document.documentElement.requestFullscreen?.()}
          className="flex items-center gap-1 rounded-md px-2 py-1 text-[11px] text-muted-foreground hover:bg-muted hover:text-foreground"
          title="Fullscreen (Esc to exit)"
        >
          <Maximize className="size-3" />
        </button>
      </div>

      {/* Canvas tabs */}
      {openTabs.length > 1 && (
        <div className="flex items-center border-b border-border bg-card/20 px-2 h-8 shrink-0 overflow-x-auto">
          {openTabs.map((tabPath) => {
            const fileName = tabPath.split('/').pop() ?? tabPath
            const isActive = tabPath === activeTab
            return (
              <button
                key={tabPath}
                onClick={() => onTabSelect(tabPath)}
                className={`group flex items-center gap-1.5 px-3 py-1 text-xs whitespace-nowrap border-b-2 transition-colors ${
                  isActive ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground hover:text-foreground'
                }`}
              >
                <span className="truncate max-w-[140px]">{fileName}</span>
                <span
                  onClick={(e) => { e.stopPropagation(); onTabClose(tabPath) }}
                  className="ml-1 text-muted-foreground hover:text-destructive transition-colors"
                >
                  <X className="size-3" />
                </span>
              </button>
            )
          })}
        </div>
      )}

      {/* Canvas area */}
      <div className="relative flex-1 min-h-0 overflow-hidden bg-muted/30">
        {htmlContent ? (
          <>
            <iframe
              ref={iframeRef}
              srcDoc={themedContent}
              title="Session canvas"
              sandbox="allow-same-origin allow-scripts"
              className="absolute inset-0 w-full h-full border-0 bg-white"
              onLoad={postTheme}
            />
            <AnnotationOverlay
              iframeRef={iframeRef}
              annotations={annotations}
              activeId={activeId}
              annotationMode={annotationMode}
              onAnnotationClick={onAnnotationClick}
              onDismissActive={onDismissActive}
              onRemoveAnnotation={onRemoveAnnotation}
              onAddClick={onAddClick}
              onAddBox={onAddBox}
              onAddAction={onAddAction}
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
