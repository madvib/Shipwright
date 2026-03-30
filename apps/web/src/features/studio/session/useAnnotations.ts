// Per-document annotation state.
// Annotations are keyed by filePath. The hook takes the active file path and
// returns only that file's annotations for the overlay — all other consumers
// see the full staged list via allStaged.

import { useState, useCallback } from 'react'
import type { Annotation, StagedAnnotation } from './types'

let seq = 1
function genId(): string {
  return `ann-${Date.now().toString(36)}-${(seq++).toString(36)}`
}

export function useAnnotations(activeFilePath: string | null) {
  const [staged, setStaged] = useState<StagedAnnotation[]>([])
  const [activeId, setActiveId] = useState<string | null>(null)
  const [annotationMode, setAnnotationMode] = useState(false)

  // Only the active file's annotations — passed to AnnotationOverlay
  const annotations = staged
    .filter((s) => s.filePath === activeFilePath)
    .map((s) => s.ann)

  const toggleActiveId = useCallback((id: string) => {
    setActiveId((prev) => (prev === id ? null : id))
  }, [])

  const dismissActive = useCallback(() => setActiveId(null), [])

  const addClickAnnotation = useCallback(
    (selector: string, text: string, note: string, x: number, y: number) => {
      const filePath = activeFilePath ?? 'general'
      const ann: Annotation = {
        type: 'click',
        id: genId(),
        selector,
        text,
        note,
        timestamp: new Date().toISOString(),
        x,
        y,
      }
      setStaged((prev) => [...prev, { filePath, id: ann.id, ann }])
      return ann.id
    },
    [activeFilePath],
  )

  const addBoxAnnotation = useCallback(
    (rect: [number, number, number, number], elements: string[], note: string) => {
      const filePath = activeFilePath ?? 'general'
      const ann: Annotation = {
        type: 'box',
        id: genId(),
        rect,
        elements,
        note,
        timestamp: new Date().toISOString(),
      }
      setStaged((prev) => [...prev, { filePath, id: ann.id, ann }])
      return ann.id
    },
    [activeFilePath],
  )

  const addActionAnnotation = useCallback(
    (action: string, text: string) => {
      const filePath = activeFilePath ?? 'general'
      const ann: Annotation = {
        type: 'action',
        id: genId(),
        action,
        text,
        timestamp: new Date().toISOString(),
      }
      setStaged((prev) => [...prev, { filePath, id: ann.id, ann }])
      return ann.id
    },
    [activeFilePath],
  )

  const removeAnnotation = useCallback((id: string) => {
    setStaged((prev) => prev.filter((s) => s.ann.id !== id))
    setActiveId((prev) => (prev === id ? null : prev))
  }, [])

  // Clears only the active file's annotations (canvas "Clear" button)
  const clearAnnotations = useCallback(() => {
    setStaged((prev) => prev.filter((s) => s.filePath !== activeFilePath))
    setActiveId(null)
  }, [activeFilePath])

  // Clears all staged annotations across all files (post-send)
  const clearAllAnnotations = useCallback(() => {
    setStaged([])
    setActiveId(null)
  }, [])

  const toExportJSON = useCallback(() => {
    return staged.map(({ ann: a }) => {
      if (a.type === 'click') {
        return { type: 'click', selector: a.selector, text: a.text, note: a.note, timestamp: a.timestamp }
      }
      if (a.type === 'action') {
        return { type: 'action', action: a.action, text: a.text, timestamp: a.timestamp }
      }
      return { type: 'box', rect: a.rect, elements: a.elements, note: a.note, timestamp: a.timestamp }
    })
  }, [staged])

  return {
    annotations,
    allStaged: staged,
    stagedCount: staged.length,
    activeId,
    annotationMode,
    setAnnotationMode,
    toggleActiveId,
    dismissActive,
    addClickAnnotation,
    addBoxAnnotation,
    addActionAnnotation,
    removeAnnotation,
    clearAnnotations,
    clearAllAnnotations,
    toExportJSON,
  }
}
