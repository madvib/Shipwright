// Session annotation types — UI-only state for the annotation system.
// These are local to the session feature and not derived from any API schema.

export interface ClickAnnotation {
  type: 'click'
  id: string
  selector: string
  text: string
  note: string
  timestamp: string
  x: number
  y: number
}

export interface BoxAnnotation {
  type: 'box'
  id: string
  rect: [number, number, number, number]
  elements: string[]
  note: string
  timestamp: string
}

export interface ActionAnnotation {
  type: 'action'
  id: string
  action: string
  text: string
  timestamp: string
}

export type Annotation = ClickAnnotation | BoxAnnotation | ActionAnnotation

export interface SessionFile {
  name: string
  path: string
  type: 'html' | 'image' | 'markdown' | 'other'
  size: number
  modifiedAt: number
}

export type UploadResult =
  | { success: true }
  | { success: false; error: string }

// Main content area display mode
export type ViewMode = 'canvas' | 'diff' | 'artifact'

// A staged annotation: an Annotation plus its file context.
// Annotations live in local React state until the user sends — no server calls per annotation.
// id is a stable client-side key used for deletion/deduplication.
export interface StagedAnnotation {
  filePath: string
  id: string
  ann: Annotation
}
