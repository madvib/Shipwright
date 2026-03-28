// State management for annotations on the session canvas.
// Manages the annotation list, active annotation editing, and persistence.

import { useState, useCallback } from 'react'
import type { Annotation } from './types'

let nextId = 1
function genId(): string {
  return `ann-${nextId++}-${Date.now()}`
}

export function useAnnotations() {
  const [annotations, setAnnotations] = useState<Annotation[]>([])
  const [activeId, setActiveId] = useState<string | null>(null)
  const [annotationMode, setAnnotationMode] = useState(false)

  const addClickAnnotation = useCallback(
    (selector: string, text: string, note: string, x: number, y: number) => {
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
      setAnnotations((prev) => [...prev, ann])
      return ann.id
    },
    [],
  )

  const addBoxAnnotation = useCallback(
    (rect: [number, number, number, number], elements: string[], note: string) => {
      const ann: Annotation = {
        type: 'box',
        id: genId(),
        rect,
        elements,
        note,
        timestamp: new Date().toISOString(),
      }
      setAnnotations((prev) => [...prev, ann])
      return ann.id
    },
    [],
  )

  const removeAnnotation = useCallback((id: string) => {
    setAnnotations((prev) => prev.filter((a) => a.id !== id))
    setActiveId((prev) => (prev === id ? null : prev))
  }, [])

  const clearAnnotations = useCallback(() => {
    setAnnotations([])
    setActiveId(null)
  }, [])

  const toExportJSON = useCallback(() => {
    return annotations.map((a) => {
      if (a.type === 'click') {
        return {
          type: 'click',
          selector: a.selector,
          text: a.text,
          note: a.note,
          timestamp: a.timestamp,
        }
      }
      return {
        type: 'box',
        rect: a.rect,
        elements: a.elements,
        note: a.note,
        timestamp: a.timestamp,
      }
    })
  }, [annotations])

  return {
    annotations,
    activeId,
    setActiveId,
    annotationMode,
    setAnnotationMode,
    addClickAnnotation,
    addBoxAnnotation,
    removeAnnotation,
    clearAnnotations,
    toExportJSON,
  }
}
