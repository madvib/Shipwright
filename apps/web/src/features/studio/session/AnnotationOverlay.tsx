// Transparent overlay that captures clicks for annotations.
// Uses onPointerDown so scrolling still works underneath. Only one
// comment input is open at a time; clicking elsewhere closes it.

import { useState, useCallback, useRef, useEffect, type RefObject } from 'react'
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
  const [iframeScroll, setIframeScroll] = useState({ top: 0, left: 0 })

  useEffect(() => {
    const iframe = iframeRef.current
    if (!iframe) return
    const attach = () => {
      const doc = iframe.contentDocument
      if (!doc) return
      const handler = () => setIframeScroll({ top: doc.documentElement.scrollTop, left: doc.documentElement.scrollLeft })
      doc.addEventListener('scroll', handler, { passive: true })
      return () => doc.removeEventListener('scroll', handler)
    }
    const cleanup = attach()
    iframe.addEventListener('load', attach)
    return () => { cleanup?.(); iframe.removeEventListener('load', attach) }
  }, [iframeRef])

  const getOverlayPos = useCallback((clientX: number, clientY: number) => {
    const overlay = overlayRef.current
    if (!overlay) return { x: 0, y: 0 }
    const rect = overlay.getBoundingClientRect()
    return { x: clientX - rect.left, y: clientY - rect.top }
  }, [])

  const getIframeScroll = useCallback(() => {
    const doc = iframeRef.current?.contentDocument
    return { top: doc?.documentElement.scrollTop ?? 0, left: doc?.documentElement.scrollLeft ?? 0 }
  }, [iframeRef])

  const toContentPos = useCallback((pos: { x: number; y: number }) => {
    const s = getIframeScroll()
    return { x: pos.x + s.left, y: pos.y + s.top }
  }, [getIframeScroll])

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
        // elementFromPoint needs viewport-relative coords (end.x/y is correct)
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
        // Store content-relative coordinates so markers follow scroll
        const contentPos = toContentPos(end)
        setPending({ type: 'click', x: contentPos.x, y: contentPos.y, selector, elementText: text })
      } else {
        // Store content-relative rect coordinates
        const scroll = getIframeScroll()
        const x = Math.min(dragStart.x, end.x) + scroll.left
        const y = Math.min(dragStart.y, end.y) + scroll.top
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
    [annotationMode, dragStart, getOverlayPos, iframeRef, toContentPos, getIframeScroll],
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
            scrollTop={iframeScroll.top}
            scrollLeft={iframeScroll.left}
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
          pending={{ ...pending, x: pending.x - iframeScroll.left, y: pending.y - iframeScroll.top }}
          onConfirm={confirmNote}
          onCancel={cancelNote}
        />
      )}
    </div>
  )
}
