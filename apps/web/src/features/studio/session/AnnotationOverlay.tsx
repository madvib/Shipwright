// Transparent overlay that captures clicks and drag-to-draw for annotations.
// Sits on top of the session canvas iframe and intercepts user interaction
// when annotation mode is active.

import { useState, useCallback, useRef, type RefObject, type MouseEvent } from 'react'
import { AnnotationMarker } from './AnnotationMarker'
import type { Annotation } from './types'

interface AnnotationOverlayProps {
  iframeRef: RefObject<HTMLIFrameElement | null>
  annotations: Annotation[]
  activeId: string | null
  annotationMode: boolean
  onAnnotationClick: (id: string) => void
  onRemoveAnnotation: (id: string) => void
  onAddClick: (selector: string, text: string, note: string, x: number, y: number) => void
  onAddBox: (rect: [number, number, number, number], elements: string[], note: string) => void
}

interface PendingInput {
  type: 'click' | 'box'
  x: number
  y: number
  selector?: string
  elementText?: string
  rect?: [number, number, number, number]
  elements?: string[]
}

function getCSSSelector(el: Element): string {
  if (el.id) return `#${el.id}`
  const tag = el.tagName.toLowerCase()
  if (el.className && typeof el.className === 'string') {
    const classes = el.className.trim().split(/\s+/).slice(0, 3).join('.')
    if (classes) return `${tag}.${classes}`
  }
  return tag
}

function getElementsInRect(
  doc: Document,
  rect: [number, number, number, number],
): string[] {
  const [rx, ry, rw, rh] = rect
  const selectors: string[] = []
  const all = doc.querySelectorAll('*')
  for (const el of all) {
    const r = el.getBoundingClientRect()
    if (r.left >= rx && r.top >= ry && r.right <= rx + rw && r.bottom <= ry + rh) {
      selectors.push(getCSSSelector(el))
    }
    if (selectors.length >= 10) break
  }
  return selectors
}

export function AnnotationOverlay({
  iframeRef,
  annotations,
  activeId,
  annotationMode,
  onAnnotationClick,
  onRemoveAnnotation,
  onAddClick,
  onAddBox,
}: AnnotationOverlayProps) {
  const overlayRef = useRef<HTMLDivElement>(null)
  const [pending, setPending] = useState<PendingInput | null>(null)
  const [noteText, setNoteText] = useState('')
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null)
  const [dragCurrent, setDragCurrent] = useState<{ x: number; y: number } | null>(null)

  const getOverlayPos = useCallback((e: MouseEvent) => {
    const overlay = overlayRef.current
    if (!overlay) return { x: 0, y: 0 }
    const rect = overlay.getBoundingClientRect()
    return { x: e.clientX - rect.left, y: e.clientY - rect.top }
  }, [])

  const handleMouseDown = useCallback(
    (e: MouseEvent) => {
      if (!annotationMode) return
      e.preventDefault()
      const pos = getOverlayPos(e)
      setDragStart(pos)
      setDragCurrent(pos)
    },
    [annotationMode, getOverlayPos],
  )

  const handleMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!dragStart) return
      setDragCurrent(getOverlayPos(e))
    },
    [dragStart, getOverlayPos],
  )

  const handleMouseUp = useCallback(
    (e: MouseEvent) => {
      if (!annotationMode || !dragStart) {
        setDragStart(null)
        setDragCurrent(null)
        return
      }

      const end = getOverlayPos(e)
      const dx = Math.abs(end.x - dragStart.x)
      const dy = Math.abs(end.y - dragStart.y)

      if (dx < 8 && dy < 8) {
        // Click — identify element under cursor
        const iframe = iframeRef.current
        let selector = 'body'
        let text = ''
        if (iframe?.contentDocument) {
          try {
            const el = iframe.contentDocument.elementFromPoint(end.x, end.y)
            if (el) {
              selector = getCSSSelector(el)
              text = (el.textContent ?? '').trim().slice(0, 80)
            }
          } catch {
            // Cross-origin restriction, fall back to body
          }
        }
        setPending({ type: 'click', x: end.x, y: end.y, selector, elementText: text })
      } else {
        // Box drag
        const x = Math.min(dragStart.x, end.x)
        const y = Math.min(dragStart.y, end.y)
        const w = dx
        const h = dy
        const rect: [number, number, number, number] = [
          Math.round(x),
          Math.round(y),
          Math.round(w),
          Math.round(h),
        ]

        let elements: string[] = []
        const iframe = iframeRef.current
        if (iframe?.contentDocument) {
          try {
            elements = getElementsInRect(iframe.contentDocument, rect)
          } catch {
            // Cross-origin fallback
          }
        }

        setPending({
          type: 'box',
          x: x + w / 2,
          y: y + h / 2,
          rect,
          elements,
        })
      }

      setDragStart(null)
      setDragCurrent(null)
    },
    [annotationMode, dragStart, getOverlayPos, iframeRef],
  )

  const confirmNote = useCallback(() => {
    if (!pending || !noteText.trim()) return

    if (pending.type === 'click') {
      onAddClick(pending.selector ?? 'body', pending.elementText ?? '', noteText.trim(), pending.x, pending.y)
    } else if (pending.rect) {
      onAddBox(pending.rect, pending.elements ?? [], noteText.trim())
    }

    setPending(null)
    setNoteText('')
  }, [pending, noteText, onAddClick, onAddBox])

  const cancelNote = useCallback(() => {
    setPending(null)
    setNoteText('')
  }, [])

  // Compute drag rect for visual feedback
  const dragRect =
    dragStart && dragCurrent
      ? {
          x: Math.min(dragStart.x, dragCurrent.x),
          y: Math.min(dragStart.y, dragCurrent.y),
          w: Math.abs(dragCurrent.x - dragStart.x),
          h: Math.abs(dragCurrent.y - dragStart.y),
        }
      : null

  return (
    <div
      ref={overlayRef}
      className={`absolute inset-0 z-10 ${annotationMode ? 'cursor-crosshair' : 'pointer-events-none'}`}
      onMouseDown={handleMouseDown}
      onMouseMove={handleMouseMove}
      onMouseUp={handleMouseUp}
    >
      {/* Existing annotations */}
      {annotations.map((ann, i) => (
        <AnnotationMarker
          key={ann.id}
          annotation={ann}
          index={i}
          isActive={activeId === ann.id}
          onClick={() => onAnnotationClick(ann.id)}
          onRemove={() => onRemoveAnnotation(ann.id)}
        />
      ))}

      {/* Active drag rect */}
      {dragRect && dragRect.w > 4 && dragRect.h > 4 && (
        <div
          className="absolute border-2 border-dashed border-primary bg-primary/10 rounded"
          style={{ left: dragRect.x, top: dragRect.y, width: dragRect.w, height: dragRect.h }}
        />
      )}

      {/* Pending note input */}
      {pending && (
        <div
          className="absolute z-30 w-64 rounded-lg border border-border bg-popover p-3 shadow-xl"
          style={{ left: Math.min(pending.x, 200), top: pending.y + 12 }}
          onMouseDown={(e) => e.stopPropagation()}
        >
          {pending.type === 'click' && pending.selector && (
            <p className="text-[10px] font-mono text-muted-foreground mb-1.5 truncate">
              {pending.selector}
            </p>
          )}
          <textarea
            autoFocus
            value={noteText}
            onChange={(e) => setNoteText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                confirmNote()
              }
              if (e.key === 'Escape') cancelNote()
            }}
            placeholder="Add note about this element..."
            className="w-full rounded-md border border-border bg-background px-2.5 py-1.5 text-xs text-foreground placeholder:text-muted-foreground resize-none focus:outline-none focus:ring-1 focus:ring-primary"
            rows={2}
          />
          <div className="flex justify-end gap-1.5 mt-2">
            <button
              onClick={cancelNote}
              className="rounded px-2 py-1 text-[11px] text-muted-foreground hover:text-foreground"
            >
              Cancel
            </button>
            <button
              onClick={confirmNote}
              disabled={!noteText.trim()}
              className="rounded bg-primary px-2.5 py-1 text-[11px] font-medium text-primary-foreground disabled:opacity-40"
            >
              Add
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
