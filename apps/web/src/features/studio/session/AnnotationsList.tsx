// Annotations tab content for the sidebar. Shows a list of all annotations
// with remove buttons and an export-to-agent button.

import { Download, X } from 'lucide-react'
import type { Annotation } from './types'

interface AnnotationsListProps {
  annotations: Annotation[]
  onRemove: (id: string) => void
  onExport: () => void
}

export function AnnotationsList({ annotations, onRemove, onExport }: AnnotationsListProps) {
  if (annotations.length === 0) {
    return (
      <p className="text-[10px] text-muted-foreground px-1 py-6 text-center">
        No annotations yet. Use the Annotate tool on the canvas.
      </p>
    )
  }

  return (
    <div className="space-y-1">
      {annotations.map((ann, i) => (
        <div
          key={ann.id}
          className="flex items-start gap-1.5 rounded px-1.5 py-1 hover:bg-muted/30 group"
        >
          <span className="flex items-center justify-center size-4 rounded-full bg-primary/15 text-primary text-[9px] font-bold shrink-0 mt-px">
            {i + 1}
          </span>
          <div className="flex-1 min-w-0">
            <p className="text-[10px] text-foreground leading-tight">
              {ann.type === 'action' ? `[${ann.action}] ${ann.text}` : ann.note}
            </p>
            <time className="text-[9px] text-muted-foreground/60">
              {new Date(ann.timestamp).toLocaleTimeString()}
            </time>
          </div>
          <button
            onClick={() => onRemove(ann.id)}
            className="shrink-0 p-0.5 rounded text-muted-foreground/0 group-hover:text-muted-foreground hover:text-foreground transition"
            aria-label="Remove annotation"
          >
            <X className="size-2.5" />
          </button>
        </div>
      ))}

      <button
        onClick={onExport}
        className="flex items-center gap-1 w-full rounded px-1.5 py-1.5 text-[10px] font-medium text-muted-foreground hover:bg-muted/50 hover:text-foreground transition mt-1"
      >
        <Download className="size-3" />
        Export to agent
      </button>
    </div>
  )
}
