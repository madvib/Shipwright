// Single staged annotation card with thumbnail (if image) or type icon + text.
// Used in ChatDraftArea above the compose input.

import { MousePointerClick, Square, Zap, X } from 'lucide-react'
import type { StagedAnnotation } from './types'

const TYPE_META = {
  click: { icon: MousePointerClick, color: 'text-sky-500', bg: 'bg-sky-500/10', label: 'Click' },
  box: { icon: Square, color: 'text-violet-500', bg: 'bg-violet-500/10', label: 'Box' },
  action: { icon: Zap, color: 'text-amber-500', bg: 'bg-amber-500/10', label: 'Action' },
} as const

interface Props {
  staged: StagedAnnotation
  onRemove: (id: string) => void
}

export function AnnotationCard({ staged, onRemove }: Props) {
  const { ann } = staged
  const meta = TYPE_META[ann.type]
  const TypeIcon = meta.icon

  const preview =
    ann.type === 'click' ? (ann.note || ann.text)
    : ann.type === 'box' ? ann.note
    : ann.text

  return (
    <div className="flex items-center gap-1.5 px-2 py-1 rounded-md border border-border/40 bg-muted/20 group">
      <span className={`flex items-center justify-center size-5 rounded shrink-0 ${meta.bg}`}>
        <TypeIcon className={`size-3 ${meta.color}`} />
      </span>
      <span className="flex-1 text-[11px] text-muted-foreground truncate min-w-0">
        <span className="font-medium text-foreground/70">{meta.label}</span>
        {preview && <span className="ml-1 opacity-70">{preview}</span>}
      </span>
      <button
        onClick={() => onRemove(staged.ann.id)}
        className="shrink-0 rounded p-0.5 hover:bg-muted/50 hover:text-destructive transition opacity-60 group-hover:opacity-100"
        title="Remove annotation"
        aria-label="Remove annotation"
      >
        <X className="size-3 text-muted-foreground" />
      </button>
    </div>
  )
}
