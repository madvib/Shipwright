// Persistent SSE notification listener for the Ship MCP connection.
// Handles cache invalidation on resource changes and event delivery.

import type { QueryClient } from '@tanstack/react-query'
import type { McpClient } from '#/lib/mcp-client'
import { mcpKeys } from '#/lib/query-keys'
import type { EventEnvelope } from '#/features/studio/events/useEventStream'

const RECONNECT_DELAY_MS = 3_000

interface ListenerRefs {
  listenerActiveRef: { current: boolean }
  clientRef: { current: McpClient | null }
}

export function startNotificationListener(
  client: McpClient,
  queryClient: QueryClient,
  refs: ListenerRefs,
): void {
  if (refs.listenerActiveRef.current) return
  refs.listenerActiveRef.current = true

  void (async () => {
    while (refs.listenerActiveRef.current && refs.clientRef.current === client) {
      try {
        await client.startNotificationListener((method, params) => {
          if (method === 'notifications/resources/list_changed') {
            void queryClient.invalidateQueries({ queryKey: mcpKeys.all })
          } else if (method === 'ship/event' && params != null) {
            const envelope = params as EventEnvelope
            queryClient.setQueryData(mcpKeys.events(), (prev: EventEnvelope[] | undefined) => {
              return [envelope, ...(prev ?? [])].slice(0, 200)
            })
          }
        })
        // Stream closed normally — fallback invalidation
        if (refs.listenerActiveRef.current && refs.clientRef.current === client) {
          void queryClient.invalidateQueries({ queryKey: mcpKeys.all })
        }
      } catch {
        if (refs.listenerActiveRef.current && refs.clientRef.current === client) {
          void queryClient.invalidateQueries({ queryKey: mcpKeys.all })
        }
      }
      if (refs.listenerActiveRef.current && refs.clientRef.current === client) {
        await new Promise((r) => setTimeout(r, RECONNECT_DELAY_MS))
      }
    }
  })()
}
