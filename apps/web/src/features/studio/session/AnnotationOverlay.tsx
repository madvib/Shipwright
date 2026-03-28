// Transparent overlay that captures clicks for annotations.
// Uses onPointerDown so scrolling still works underneath. Only one
// comment input is open at a time; clicking elsewhere closes it.

import { useState, useCallback, useRef, type RefObject } from 'react'
import { AnnotationMarker } from './AnnotationMarker'
import { AnnotationInput } from './AnnotationInput'
import { getCSSSelector, getElementsInRect } from './annotation-helpers'
import type { Annotation } from './types'

interface AnnotationOverlayProps {
  iframeRef: RefObject<HTMLIFrameElement | null>
  annotations: Annotation[]
  activeId: string | null
  annotationMode: boolean
  onAnnotationClick: (id: string) => void
  onDismissActive: () => void
  onRemoveAnnotation: (id: string) => void
  onAddClick: (selector: string, text: string, note: string, x: number, y: number) => void
  onAddBox: (rect: [number, number, number, number], elements: string[], note: string) => void
  onAddAction: (action: string, text: string) => void
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

export function AnnotationOverlay({
  iframeRef,
  annotations,
  activeId,
  annotationMode,
  onAnnotationClick,
  onDismissActive,
  onRemoveAnnotation,
  onAddClick,
  onAddBox,
  onAddAction,
}: AnnotationOverlayProps) {
  const overlayRef = useRef<HTMLDivElement>(null)
  const [pending, setPending] = useState<PendingInput | null>(null)
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null)
  const [dragCurrent, setDragCurrent] = useState<{ x: number; y: number } | null>(null)

  const getOverlayPos = useCallback(
    (clientX: number, clientY: number) => {
      const overlay = overlayRef.current
      if (!overlay) return { x: 0, y: 0 }
      const rect = overlay.getBoundingClientRect()
      return { x: clientX - rect.left, y: clientY - rect.top }
    },
    [],
  )

  const cancelNote = useCallback(() => setPending(null), [])

  const confirmNote = useCallback(
    (noteText: string) => {
      if (!pending || !noteText.trim()) return
      if (pending.type === 'click') {
        onAddClick(pending.selector ?? 'body', pending.elementText ?? '', noteText.trim(), pending.x, pending.y)
      } else if (pending.rect) {
        onAddBox(pending.rect, pending.elements ?? [], noteText.trim())
      }
      setPending(null)
    },
    [pending, onAddClick, onAddBox],
  )

  const handleOverlayPointerDown = useCallback(
    (e: React.PointerEvent) => {
      if (!annotationMode) return
      const target = e.target as HTMLElement
      if (target.closest('[data-annotation-marker]') || target.closest('[data-annotation-input]')) return

      // Check for data-ship-action buttons in the iframe
      const pos = getOverlayPos(e.clientX, e.clientY)
      const iframe = iframeRef.current
      if (iframe?.contentDocument) {
        try {
          const el = iframe.contentDocument.elementFromPoint(pos.x, pos.y)
          if (el) {
            const actionEl = el.closest('[data-ship-action]')
            if (actionEl) {
              const action = actionEl.getAttribute('data-ship-action')
              if (action) {
                e.preventDefault()
                e.stopPropagation()
                onAddAction(action, actionEl.textContent?.trim() ?? action)
                return
              }
            }
          }
        } catch { /* cross-origin */ }
      }

      // Close pending input if open
      if (pending) {
        cancelNote()
        e.preventDefault()
        e.stopPropagation()
        return
      }
      if (activeId) onDismissActive()

      e.preventDefault()
      e.stopPropagation()
      setDragStart(pos)
      setDragCurrent(pos)
    },
    [annotationMode, pending, activeId, cancelNote, onDismissActive, iframeRef, getOverlayPos, onAddAction],
  )

  const handlePointerMove = useCallback(
    (e: React.PointerEvent) => {
      if (!dragStart) return
      setDragCurrent(getOverlayPos(e.clientX, e.clientY))
    },
    [dragStart, getOverlayPos],
  )

  const handlePointerUp = useCallback(
    (e: React.PointerEvent) => {
      if (!annotationMode || !dragStart) {
        setDragStart(null)
        setDragCurrent(null)
        return
      }
      const end = getOverlayPos(e.clientX, e.clientY)
      const dx = Math.abs(end.x - dragStart.x)
      const dy = Math.abs(end.y - dragStart.y)

      if (dx < 8 && dy < 8) {
        let selector = 'body'
        let text = ''
        const iframe = iframeRef.current
        if (iframe?.contentDocument) {
          try {
            const el = iframe.contentDocument.elementFromPoint(end.x, end.y)
            if (el) {
              selector = getCSSSelector(el)
              text = (el.textContent ?? '').trim().slice(0, 80)
            }
          } catch { /* cross-origin */ }
        }
        setPending({ type: 'click', x: end.x, y: end.y, selector, elementText: text })
      } else {
        const x = Math.min(dragStart.x, end.x)
        const y = Math.min(dragStart.y, end.y)
        const rect: [number, number, number, number] = [Math.round(x), Math.round(y), Math.round(dx), Math.round(dy)]
        let elements: string[] = []
        const iframe = iframeRef.current
        if (iframe?.contentDocument) {
          try { elements = getElementsInRect(iframe.contentDocument, rect) } catch { /* cross-origin */ }
        }
        setPending({ type: 'box', x: x + dx / 2, y: y + dy / 2, rect, elements })
      }

      setDragStart(null)
      setDragCurrent(null)
    },
    [annotationMode, dragStart, getOverlayPos, iframeRef],
  )

  const dragRect = dragStart && dragCurrent
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
      style={annotationMode ? { touchAction: 'pan-y' } : undefined}
      onPointerDown={handleOverlayPointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
    >
      {annotations.map((ann, i) => {
        if (ann.type === 'action') return null
        return (
          <AnnotationMarker
            key={ann.id}
            annotation={ann}
            index={i}
            isActive={activeId === ann.id}
            onClick={() => onAnnotationClick(ann.id)}
            onRemove={() => onRemoveAnnotation(ann.id)}
          />
        )
      })}

      {dragRect && dragRect.w > 4 && dragRect.h > 4 && (
        <div
          className="absolute border-2 border-dashed border-primary bg-primary/10 rounded pointer-events-none"
          style={{ left: dragRect.x, top: dragRect.y, width: dragRect.w, height: dragRect.h }}
        />
      )}

      {pending && (
        <AnnotationInput
          pending={pending}
          onConfirm={confirmNote}
          onCancel={cancelNote}
        />
      )}
    </div>
  )
}
