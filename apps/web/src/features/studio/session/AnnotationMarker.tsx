// Renders a single annotation marker on the overlay.
// Click annotations show a numbered pin, box annotations show a colored rect.

import { X } from 'lucide-react'
import type { Annotation } from './types'

interface AnnotationMarkerProps {
  annotation: Annotation
  index: number
  isActive: boolean
  onClick: () => void
  onRemove: () => void
}

export function AnnotationMarker({ annotation, index, isActive, onClick, onRemove }: AnnotationMarkerProps) {
  if (annotation.type === 'click') {
    return (
      <div
        className="absolute z-10 group"
        style={{ left: annotation.x, top: annotation.y, transform: 'translate(-50%, -50%)' }}
      >
        <button
          onClick={onClick}
          className={`flex items-center justify-center size-6 rounded-full text-[10px] font-bold shadow-md transition-all ${
            isActive
              ? 'bg-primary text-primary-foreground scale-125'
              : 'bg-primary/80 text-primary-foreground hover:scale-110'
          }`}
        >
          {index + 1}
        </button>

        {isActive && (
          <div className="absolute top-8 left-1/2 -translate-x-1/2 w-56 rounded-lg border border-border bg-popover p-3 shadow-xl z-20">
            <div className="flex items-start justify-between gap-2 mb-1">
              <span className="text-[10px] font-mono text-muted-foreground truncate max-w-[160px]">
                {annotation.selector}
              </span>
              <button onClick={onRemove} className="shrink-0 text-muted-foreground hover:text-foreground">
                <X className="size-3" />
              </button>
            </div>
            <p className="text-xs text-foreground">{annotation.note}</p>
            <time className="text-[10px] text-muted-foreground mt-1 block">
              {new Date(annotation.timestamp).toLocaleTimeString()}
            </time>
          </div>
        )}
      </div>
    )
  }

  // Box annotation
  const [x, y, w, h] = annotation.rect
  return (
    <div className="absolute z-10" style={{ left: x, top: y, width: w, height: h }}>
      <button
        onClick={onClick}
        className={`w-full h-full border-2 rounded transition-colors ${
          isActive
            ? 'border-primary bg-primary/20'
            : 'border-primary/50 bg-primary/10 hover:border-primary hover:bg-primary/15'
        }`}
      />

      {/* Number badge in top-left */}
      <span
        className={`absolute -top-2.5 -left-2.5 flex items-center justify-center size-5 rounded-full text-[9px] font-bold shadow ${
          isActive ? 'bg-primary text-primary-foreground' : 'bg-primary/80 text-primary-foreground'
        }`}
      >
        {index + 1}
      </span>

      {isActive && (
        <div className="absolute top-full left-0 mt-2 w-56 rounded-lg border border-border bg-popover p-3 shadow-xl z-20">
          <div className="flex items-start justify-between gap-2 mb-1">
            <span className="text-[10px] text-muted-foreground">
              {annotation.elements.length} element{annotation.elements.length !== 1 ? 's' : ''}
            </span>
            <button onClick={onRemove} className="shrink-0 text-muted-foreground hover:text-foreground">
              <X className="size-3" />
            </button>
          </div>
          <p className="text-xs text-foreground">{annotation.note}</p>
          <time className="text-[10px] text-muted-foreground mt-1 block">
            {new Date(annotation.timestamp).toLocaleTimeString()}
          </time>
        </div>
      )}
    </div>
  )
}
