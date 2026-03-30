// Dev-only event debug panel. Only rendered when import.meta.env.DEV is true.
// Toggle with Ctrl+Shift+E. Shows the live ship/event stream from the MCP server.

import { useState, useEffect, useCallback } from 'react'
import { Badge } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Activity, ChevronDown, ChevronRight, X } from 'lucide-react'
import { useEventStream } from './useEventStream'
import type { EventEnvelope } from './useEventStream'

// ── Event row ──────────────────────────────────────────────────────────────

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
        <span className="text-muted-foreground text-[10px] font-mono shrink-0 mt-0.5 w-16">{ts}</span>
        <span className="text-[11px] font-mono text-foreground/90 flex-1 truncate">{envelope.event_type}</span>
        {envelope.entity_id && (
          <span className="text-[10px] text-muted-foreground truncate max-w-[100px]">{envelope.entity_id}</span>
        )}
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
            {JSON.stringify(
              {
                id: envelope.id,
                workspace_id: envelope.workspace_id,
                payload: payload,
              },
              null,
              2,
            )}
          </pre>
        </div>
      )}
    </div>
  )
}

// ── Panel ──────────────────────────────────────────────────────────────────

export function EventDebugPanel() {
  const { events, isConnected, clearEvents } = useEventStream()
  const [visible, setVisible] = useState(false)
  const [filter, setFilter] = useState('')

  // Keyboard shortcut: Ctrl+Shift+E
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.ctrlKey && e.shiftKey && e.key === 'E') {
      e.preventDefault()
      setVisible((p) => !p)
    }
  }, [])

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  if (!visible) {
    return (
      <button
        className="fixed bottom-4 right-4 z-50 flex items-center gap-1.5 rounded-full bg-card border border-border px-3 py-1.5 text-[11px] text-muted-foreground shadow-lg hover:text-foreground transition-colors"
        onClick={() => setVisible(true)}
        title="Open event debug panel (Ctrl+Shift+E)"
      >
        <Activity className="size-3" />
        <span>Events{events.length > 0 ? ` (${events.length})` : ''}</span>
      </button>
    )
  }

  const filtered = filter
    ? events.filter((e) => e.event_type.startsWith(filter))
    : events

  return (
    <div className="fixed bottom-0 right-0 z-50 w-[480px] max-h-[50vh] flex flex-col border border-border bg-card shadow-xl rounded-tl-lg overflow-hidden">
      {/* Header */}
      <div className="flex items-center gap-2 px-3 py-2 border-b border-border shrink-0">
        <Activity className="size-3.5 text-primary" />
        <span className="text-[11px] font-semibold flex-1">Event Stream</span>
        <span className={`size-1.5 rounded-full shrink-0 ${isConnected ? 'bg-green-500' : 'bg-muted'}`} />
        <span className="text-[10px] text-muted-foreground">{isConnected ? 'connected' : 'disconnected'}</span>
        <Button variant="ghost" size="sm" className="h-5 px-1.5 text-[10px]" onClick={clearEvents}>
          Clear
        </Button>
        <button onClick={() => setVisible(false)} className="text-muted-foreground hover:text-foreground transition-colors">
          <X className="size-3.5" />
        </button>
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
      <div className="flex-1 overflow-y-auto">
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
        {filtered.length} of {events.length} events · Ctrl+Shift+E to toggle
      </div>
    </div>
  )
}
