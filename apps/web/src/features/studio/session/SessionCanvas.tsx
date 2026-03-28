// Renders an iframe with the session HTML content and an annotation overlay.
// The iframe loads HTML via srcdoc. Theme is synced from studio to iframe
// via postMessage so the canvas content respects the current theme.

import { useRef, useEffect, useCallback, useMemo } from 'react'
import { MousePointerClick, Trash2, Download, Square, Maximize } from 'lucide-react'
import { AnnotationOverlay } from './AnnotationOverlay'
import type { Annotation } from './types'

interface SessionCanvasProps {
  htmlContent: string
  fileType?: string | null
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

/** Wrap non-HTML content in a styled HTML shell for iframe rendering */
function wrapContent(content: string, fileType?: string | null): string {
  if (!content) return ''
  if (fileType === 'image') {
    // base64 data URI — render as zoomable image
    return `<!DOCTYPE html><html><head><style>
      * { margin: 0; padding: 0; box-sizing: border-box; }
      body { background: #1a1a1a; display: flex; align-items: center; justify-content: center; min-height: 100vh; overflow: auto; }
      img { max-width: 100%; cursor: zoom-in; transition: transform 0.2s; }
      img.zoomed { cursor: zoom-out; transform: scale(2); transform-origin: center; }
    </style></head><body>
      <img src="${content}" onclick="this.classList.toggle('zoomed')" />
    </body></html>`
  }
  if (fileType === 'markdown') {
    // Simple markdown-to-HTML for iframe rendering
    const html = content
      .replace(/^### (.+)$/gm, '<h3>$1</h3>')
      .replace(/^## (.+)$/gm, '<h2>$1</h2>')
      .replace(/^# (.+)$/gm, '<h1>$1</h1>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/`([^`]+)`/g, '<code>$1</code>')
      .replace(/^- (.+)$/gm, '<li>$1</li>')
      .replace(/(<li>.*<\/li>)/s, '<ul>$1</ul>')
      .replace(/\n\n/g, '</p><p>')
      .replace(/^(?!<[hulo])/gm, '<p>')
    return `<!DOCTYPE html><html><head><style>
      * { margin: 0; padding: 0; box-sizing: border-box; }
      body { font-family: -apple-system, sans-serif; padding: 2rem; max-width: 48rem; margin: 0 auto; line-height: 1.6; color: #e8e0d6; background: #18140f; }
      h1, h2, h3 { margin: 1.5rem 0 0.75rem; font-weight: 700; }
      h1 { font-size: 1.75rem; } h2 { font-size: 1.35rem; } h3 { font-size: 1.1rem; }
      p { margin: 0.5rem 0; } code { background: #2a2520; padding: 0.15rem 0.4rem; border-radius: 0.25rem; font-size: 0.9em; }
      ul { padding-left: 1.5rem; margin: 0.5rem 0; } li { margin: 0.25rem 0; }
      strong { font-weight: 600; }
    </style></head><body>${html}</body></html>`
  }
  // Default: treat as HTML
  return content
}

export function SessionCanvas({
  htmlContent,
  fileType,
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
    const wrapped = wrapContent(htmlContent, fileType)
    return injectThemeListener(wrapped)
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
