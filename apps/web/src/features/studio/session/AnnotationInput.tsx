// Floating input for adding a note to a pending annotation (click or box).
// Only one instance is rendered at a time by the overlay.

import { useState, useCallback } from 'react'
import { X } from 'lucide-react'

interface PendingInput {
  type: 'click' | 'box'
  x: number
  y: number
  selector?: string
  elementText?: string
  rect?: [number, number, number, number]
  elements?: string[]
}

interface AnnotationInputProps {
  pending: PendingInput
  onConfirm: (noteText: string) => void
  onCancel: () => void
}

export function AnnotationInput({ pending, onConfirm, onCancel }: AnnotationInputProps) {
  const [noteText, setNoteText] = useState('')

  const handleConfirm = useCallback(() => {
    if (!noteText.trim()) return
    onConfirm(noteText)
    setNoteText('')
  }, [noteText, onConfirm])

  const handleCancel = useCallback(() => {
    setNoteText('')
    onCancel()
  }, [onCancel])

  return (
    <div
      data-annotation-input
      className="absolute z-30 w-64 rounded-lg border border-border bg-popover p-3 shadow-xl pointer-events-auto"
      style={{ left: Math.min(pending.x, 200), top: pending.y + 12 }}
      onPointerDown={(e) => e.stopPropagation()}
    >
      <div className="flex items-center justify-between mb-1.5">
        {pending.type === 'click' && pending.selector && (
          <p className="text-[10px] font-mono text-muted-foreground truncate flex-1">
            {pending.selector}
          </p>
        )}
        {pending.type === 'box' && (
          <p className="text-[10px] text-muted-foreground flex-1">Box selection</p>
        )}
        <button
          onClick={handleCancel}
          className="shrink-0 p-0.5 rounded text-muted-foreground hover:text-foreground"
          aria-label="Close annotation input"
        >
          <X className="size-3" />
        </button>
      </div>
      <textarea
        autoFocus
        value={noteText}
        onChange={(e) => setNoteText(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault()
            handleConfirm()
          }
          if (e.key === 'Escape') handleCancel()
        }}
        placeholder="Add note about this element..."
        className="w-full rounded-md border border-border bg-background px-2.5 py-1.5 text-xs text-foreground placeholder:text-muted-foreground resize-none focus:outline-none focus:ring-1 focus:ring-primary"
        rows={2}
      />
      <div className="flex justify-end gap-1.5 mt-2">
        <button
          onClick={handleCancel}
          className="rounded px-2 py-1 text-[11px] text-muted-foreground hover:text-foreground"
        >
          Cancel
        </button>
        <button
          onClick={handleConfirm}
          disabled={!noteText.trim()}
          className="rounded bg-primary px-2.5 py-1 text-[11px] font-medium text-primary-foreground disabled:opacity-40"
        >
          Add
        </button>
      </div>
    </div>
  )
}
