// Inner event stream panel — no FAB shell, no fixed positioning.
// Extracted from EventDebugPanel for embedding in the right drawer.

import { useState } from 'react'
import { Badge } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Activity, ChevronDown, ChevronRight } from 'lucide-react'
import { useEventStream } from './useEventStream'
import type { EventEnvelope } from './useEventStream'

function EventRow({ envelope }: { envelope: EventEnvelope }) {
  const [expanded, setExpanded] = useState(false)

  const ts = new Date(envelope.created_at).toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  })

  let payload: unknown = null
  if (envelope.payload_json) {
    try { payload = JSON.parse(envelope.payload_json) } catch { payload = envelope.payload_json }
  }

  return (
    <div className="border-b border-border/30 last:border-0">
      <button
        className="w-full flex items-start gap-2 px-3 py-1.5 hover:bg-muted/40 transition-colors text-left"
        onClick={() => setExpanded((p) => !p)}
      >
        <span className="text-muted-foreground text-[10px] font-mono shrink-0 mt-0.5 w-14">{ts}</span>
        <span className="text-[11px] font-mono text-foreground/90 flex-1 truncate">{envelope.event_type}</span>
        {envelope.actor && (
          <Badge variant="outline" className="text-[9px] px-1 py-0 shrink-0">{envelope.actor}</Badge>
        )}
        <span className="shrink-0 text-muted-foreground">
          {expanded ? <ChevronDown className="size-3" /> : <ChevronRight className="size-3" />}
        </span>
      </button>
      {expanded && (
        <div className="px-3 pb-2">
          <pre className="text-[10px] font-mono text-foreground/70 bg-muted/30 rounded p-2 overflow-x-auto whitespace-pre-wrap break-all">
            {JSON.stringify({ id: envelope.id, workspace_id: envelope.workspace_id, payload }, null, 2)}
          </pre>
        </div>
      )}
    </div>
  )
}

export function EventStreamPanel() {
  const { events, isConnected, clearEvents } = useEventStream()
  const [filter, setFilter] = useState('')

  const filtered = filter ? events.filter((e) => e.event_type.startsWith(filter)) : events

  return (
    <div className="flex flex-col flex-1 min-h-0">
      {/* Sub-header */}
      <div className="flex items-center gap-2 px-3 py-1.5 border-b border-border/50 shrink-0">
        <Activity className="size-3 text-primary shrink-0" />
        <span className={`size-1.5 rounded-full shrink-0 ${isConnected ? 'bg-green-500' : 'bg-muted'}`} />
        <span className="text-[10px] text-muted-foreground flex-1">{isConnected ? 'connected' : 'disconnected'}</span>
        <Button variant="ghost" size="sm" className="h-5 px-1.5 text-[10px]" onClick={clearEvents}>
          Clear
        </Button>
      </div>

      {/* Filter */}
      <div className="px-3 py-1.5 border-b border-border/50 shrink-0">
        <Input
          placeholder="Filter by event_type prefix…"
          value={filter}
          onChange={(e) => setFilter(e.target.value)}
          className="h-6 text-[11px]"
        />
      </div>

      {/* Event list */}
      <div className="flex-1 overflow-y-auto min-h-0">
        {filtered.length === 0 ? (
          <div className="py-6 text-center text-[11px] text-muted-foreground">
            {events.length === 0 ? 'Waiting for events…' : 'No events match filter.'}
          </div>
        ) : (
          filtered.map((e) => <EventRow key={e.id} envelope={e} />)
        )}
      </div>

      {/* Footer */}
      <div className="px-3 py-1 border-t border-border/50 shrink-0 text-[10px] text-muted-foreground">
        {filtered.length} of {events.length} events
      </div>
    </div>
  )
}
