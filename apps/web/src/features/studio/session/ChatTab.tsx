// Chat tab content: filtered studio.* event history (oldest first) + compose area.

import { useRef, useEffect } from 'react'
import { MessageSquare } from 'lucide-react'
import { useEventStream } from '../events/useEventStream'
import { ChatDraftArea } from './ChatDraftArea'
import type { StagedAnnotation } from './types'

interface Props {
  stagedAnnotations: StagedAnnotation[]
  onSend: (text: string) => Promise<void>
  onRemoveAnnotation: (id: string) => void
  onUploadFiles: (files: FileList) => void
  disabled?: boolean
}

export function ChatTab({
  stagedAnnotations,
  onSend,
  onRemoveAnnotation,
  onUploadFiles,
  disabled = false,
}: Props) {
  const { events } = useEventStream()
  const bottomRef = useRef<HTMLDivElement>(null)

  // Filter to studio.* events, reverse to oldest-first
  const studioEvents = [...events].filter((e) => e.event_type.startsWith('studio.')).reverse()

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [studioEvents.length])

  return (
    <div className="flex flex-col flex-1 min-h-0">
      <div className="flex-1 overflow-y-auto min-h-0 px-3 py-2 space-y-2">
        {studioEvents.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full gap-2 py-8 text-center">
            <MessageSquare className="size-6 text-muted-foreground/30" />
            <p className="text-[11px] text-muted-foreground/50">No messages yet</p>
            <p className="text-[10px] text-muted-foreground/30">
              Stage annotations on the canvas, then send to agent
            </p>
          </div>
        ) : (
          studioEvents.map((e) => {
            let payload: Record<string, unknown> | null = null
            if (e.payload_json) {
              try { payload = JSON.parse(e.payload_json) as Record<string, unknown> } catch { /* skip */ }
            }

            const ts = new Date(e.created_at).toLocaleTimeString(undefined, {
              hour: '2-digit',
              minute: '2-digit',
            })

            const summary = payload && typeof payload['summary'] === 'string' ? payload['summary'] : null
            const annotations = payload && Array.isArray(payload['annotations']) ? payload['annotations'] : []

            return (
              <div key={e.id} className="rounded-lg border border-border/30 bg-muted/10 p-2 space-y-1">
                <div className="flex items-center gap-2">
                  <span className="text-[10px] font-mono text-primary/70 truncate flex-1">{e.event_type}</span>
                  <span className="text-[9px] text-muted-foreground/50 shrink-0">{ts}</span>
                </div>
                {summary && (
                  <p className="text-[11px] text-foreground/80 leading-snug">{summary}</p>
                )}
                {annotations.length > 0 && (
                  <p className="text-[10px] text-muted-foreground/60">
                    {annotations.length} annotation{annotations.length !== 1 ? 's' : ''}
                  </p>
                )}
              </div>
            )
          })
        )}
        <div ref={bottomRef} />
      </div>

      <ChatDraftArea
        stagedAnnotations={stagedAnnotations}
        onSend={onSend}
        onRemoveAnnotation={onRemoveAnnotation}
        onUploadFiles={onUploadFiles}
        disabled={disabled}
      />
    </div>
  )
}
