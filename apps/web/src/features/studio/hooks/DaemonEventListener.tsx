// Singleton SSE listener for daemon events.
// Mount once in the app root. Writes events to the TanStack Query cache.

import { useEffect, useRef } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { mcpKeys } from '#/lib/query-keys'
import { daemonKeys } from './useDaemon'
import type { EventEnvelope } from '#/features/studio/events/useEventStream'

const SSE_RECONNECT_BASE_MS = 1000
const SSE_RECONNECT_MAX_MS = 30000
const EVENT_RING_BUFFER_SIZE = 200

export function DaemonEventListener() {
  const queryClient = useQueryClient()
  const retryDelay = useRef(SSE_RECONNECT_BASE_MS)

  useEffect(() => {
    let es: EventSource | null = null
    let timer: ReturnType<typeof setTimeout> | null = null
    let cancelled = false

    function connect() {
      if (cancelled) return
      es = new EventSource(`${DAEMON_BASE_URL}/api/runtime/events`)

      es.addEventListener('ship.event', (e: MessageEvent) => {
        try {
          const envelope = JSON.parse(e.data as string) as EventEnvelope
          if (envelope.event_type?.startsWith('workspace.')) {
            void queryClient.invalidateQueries({ queryKey: daemonKeys.workspaces })
          }
          queryClient.setQueryData<EventEnvelope[]>(mcpKeys.events(), (prev) => {
            const next = [envelope, ...(prev ?? [])]
            return next.length > EVENT_RING_BUFFER_SIZE ? next.slice(0, EVENT_RING_BUFFER_SIZE) : next
          })
        } catch {
          // malformed payload — ignore
        }
      })

      es.onopen = () => {
        retryDelay.current = SSE_RECONNECT_BASE_MS
      }

      es.onerror = () => {
        es?.close()
        es = null
        if (cancelled) return
        timer = setTimeout(connect, retryDelay.current)
        retryDelay.current = Math.min(retryDelay.current * 2, SSE_RECONNECT_MAX_MS)
      }
    }

    connect()

    return () => {
      cancelled = true
      if (timer) clearTimeout(timer)
      es?.close()
    }
  }, [queryClient])

  return null
}
