// Panel showing all staged annotations grouped by document.
// Used in the sidebar Files tab.

import { MousePointerClick, Square, Zap, Trash2 } from 'lucide-react'
import type { StagedAnnotation } from './types'

const TYPE_META = {
  click: { icon: MousePointerClick, color: 'text-sky-500', bg: 'bg-sky-500/10', label: 'Click' },
  box: { icon: Square, color: 'text-violet-500', bg: 'bg-violet-500/10', label: 'Box' },
  action: { icon: Zap, color: 'text-amber-500', bg: 'bg-amber-500/10', label: 'Action' },
} as const

interface Props {
  staged: StagedAnnotation[]
  onNavigate: (filePath: string, annotationId: string) => void
  onDelete: (annotationId: string) => void
  onClearAll: () => void
}

export function StagedAnnotationsPanel({ staged, onNavigate, onDelete, onClearAll }: Props) {
  const groups = staged.reduce<Map<string, StagedAnnotation[]>>((acc, s) => {
    if (!acc.has(s.filePath)) acc.set(s.filePath, [])
    acc.get(s.filePath)!.push(s)
    return acc
  }, new Map())

  return (
    <div className="mt-3 border-t border-border/40 pt-3">
      <div className="flex items-center justify-between mb-1.5">
        <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/60">
          Staged · {staged.length}
        </span>
        <button
          onClick={onClearAll}
          className="text-[10px] text-muted-foreground/50 hover:text-destructive transition"
          title="Clear all annotations"
        >
          Clear all
        </button>
      </div>

      <div className="space-y-2">
        {Array.from(groups.entries()).map(([filePath, items]) => (
          <div key={filePath}>
            <p className="px-1 text-[10px] text-muted-foreground/50 font-medium truncate mb-0.5">
              {filePath.split('/').pop() ?? filePath}
            </p>
            {items.map((s) => {
              const meta = TYPE_META[s.ann.type]
              const TypeIcon = meta.icon
              const preview =
                s.ann.type === 'click' ? (s.ann.note || s.ann.text)
                : s.ann.type === 'box' ? s.ann.note
                : s.ann.text
              return (
                <div
                  key={s.ann.id}
                  className="group flex items-center gap-1.5 px-1.5 py-1 rounded-md hover:bg-muted/30 cursor-pointer transition"
                  onClick={() => onNavigate(s.filePath, s.ann.id)}
                >
                  <span className={`flex items-center justify-center size-4 rounded shrink-0 ${meta.bg}`}>
                    <TypeIcon className={`size-2.5 ${meta.color}`} />
                  </span>
                  <span className="flex-1 text-[11px] text-muted-foreground truncate">
                    {preview || meta.label}
                  </span>
                  <button
                    onClick={(e) => { e.stopPropagation(); onDelete(s.ann.id) }}
                    className="opacity-0 group-hover:opacity-100 rounded p-0.5 hover:text-destructive transition"
                    title="Remove"
                  >
                    <Trash2 className="size-3 text-muted-foreground" />
                  </button>
                </div>
              )
            })}
          </div>
        ))}
      </div>
    </div>
  )
}
