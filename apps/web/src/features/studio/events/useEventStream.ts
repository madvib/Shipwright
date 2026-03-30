// Hook for consuming the ship/event notification stream.
// Events are written to the TanStack Query cache by useLocalMcp's SSE listener.
// This hook provides a read view + clearEvents().

import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { mcpKeys } from '#/lib/query-keys'

/** Shape of a ship/event MCP notification payload. */
export interface EventEnvelope {
  id: string
  event_type: string
  entity_id: string | null
  actor: string | null
  payload_json: string | null
  workspace_id: string | null
  created_at: string
}

export interface UseEventStreamReturn {
  events: EventEnvelope[]
  isConnected: boolean
  clearEvents: () => void
}

/**
 * Returns the live ship/event stream from the MCP notification channel.
 * Events are populated by the SSE listener in useLocalMcp — this hook only reads.
 * Ring buffer: max 200 events, most-recent-first.
 */
export function useEventStream(): UseEventStreamReturn {
  const mcp = useLocalMcpContext()
  const queryClient = useQueryClient()

  const { data } = useQuery<EventEnvelope[]>({
    queryKey: mcpKeys.events(),
    // Query data is written externally by the SSE listener.
    // Return empty array as initial value; never refetch automatically.
    queryFn: () => [],
    staleTime: Infinity,
    gcTime: Infinity,
  })

  const clearEvents = () => {
    queryClient.setQueryData(mcpKeys.events(), [])
  }

  return {
    events: data ?? [],
    isConnected: mcp?.status === 'connected',
    clearEvents,
  }
}
