// Renders an iframe with the session HTML content and an annotation overlay.
// The iframe loads HTML via srcdoc. Theme is synced from studio to iframe
// via postMessage so the canvas content respects the current theme.

import { useRef, useEffect, useCallback, useMemo } from 'react'
import { MousePointerClick, Trash2, Download, Square } from 'lucide-react'
import { AnnotationOverlay } from './AnnotationOverlay'
import type { Annotation } from './types'

interface SessionCanvasProps {
  htmlContent: string
  annotations: Annotation[]
  activeId: string | null
  annotationMode: boolean
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

/** Inject a theme listener script into srcdoc HTML so the iframe responds to
 *  postMessage({ type: 'theme', theme: 'dark' | 'light' }) from the parent. */
function injectThemeListener(html: string): string {
  const script = `<script>
window.addEventListener('message', function(e) {
  if (e.data && e.data.type === 'theme') {
    var root = document.documentElement;
    root.classList.remove('light', 'dark');
    root.classList.add(e.data.theme);
    root.setAttribute('data-theme', e.data.theme);
    root.style.colorScheme = e.data.theme;
  }
});
</script>`
  // Insert before </head> if present, otherwise before </html> or at end
  if (html.includes('</head>')) return html.replace('</head>', script + '</head>')
  if (html.includes('</html>')) return html.replace('</html>', script + '</html>')
  return html + script
}

function getResolvedTheme(): string {
  const root = document.documentElement
  if (root.classList.contains('dark')) return 'dark'
  if (root.classList.contains('light')) return 'light'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function SessionCanvas({
  htmlContent,
  annotations,
  activeId,
  annotationMode,
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
    return injectThemeListener(htmlContent)
  }, [htmlContent])

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
      </div>

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
