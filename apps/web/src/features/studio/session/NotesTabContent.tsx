// Notes tab content: TODO checklist and annotations list.

import { SessionTodo } from './SessionTodo'
import { AnnotationsList } from './AnnotationsList'
import type { Annotation } from './types'

interface NotesTabContentProps {
  isConnected: boolean
  annotations: Annotation[]
  onRemoveAnnotation: (id: string) => void
  onExportAnnotations: () => void
}

export function NotesTabContent({
  isConnected, annotations, onRemoveAnnotation, onExportAnnotations,
}: NotesTabContentProps) {
  return (
    <div className="space-y-3">
      <div>
        <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">TODO</p>
        {isConnected ? <SessionTodo /> : (
          <p className="text-[10px] text-muted-foreground px-1">Connect CLI for TODO management</p>
        )}
      </div>
      <div>
        <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">Annotations</p>
        <AnnotationsList annotations={annotations} onRemove={onRemoveAnnotation} onExport={onExportAnnotations} />
      </div>
    </div>
  )
}
