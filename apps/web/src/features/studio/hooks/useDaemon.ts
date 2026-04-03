// Hook for connecting to the shipd daemon REST/SSE API at port 51742.
// Provides live workspace and agent state without going through MCP.

import { useEffect, useRef } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { mcpKeys } from '#/lib/query-keys'
import type { Workspace } from '@ship/ui'
import type { EventEnvelope } from '#/features/studio/events/useEventStream'

// ---- Types derived from shipd runtime_api.rs response shapes ----

export interface AgentEntry {
  agent_id: string
  label: string
  capabilities: string[]
  status: string
}

export interface UseDaemonReturn {
  connected: boolean
  workspaces: Workspace[]
  agents: AgentEntry[]
  error: Error | null
}

// ---- Query keys ----

export const daemonKeys = {
  workspaces: ['daemon', 'workspaces'] as const,
  agents: ['daemon', 'agents'] as const,
}

// ---- Constants ----

const SSE_RECONNECT_BASE_MS = 1000
const SSE_RECONNECT_MAX_MS = 30000
const EVENT_RING_BUFFER_SIZE = 200

/** Prefixes that indicate test-leaked workspaces from integration tests. */
const TEST_BRANCH_PREFIXES = [
  'feature/evt-',
  'feature/rebuild-src-',
  'feature/test-',
  'test/',
]

function isTestWorkspace(ws: Workspace): boolean {
  return TEST_BRANCH_PREFIXES.some((p) => ws.branch.startsWith(p))
}

// ---- Fetchers ----

async function fetchWorkspaces(): Promise<Workspace[]> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/workspaces`)
  if (!res.ok) throw new Error(`daemon: workspaces ${res.status}`)
  const body = await res.json() as { ok: boolean; data: { workspaces: Workspace[] } }
  return body.data.workspaces
}

async function fetchAgents(): Promise<AgentEntry[]> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/agents`)
  if (!res.ok) throw new Error(`daemon: agents ${res.status}`)
  const body = await res.json() as { ok: boolean; data: { agents: AgentEntry[] } }
  return body.data.agents
}

// ---- Hook ----

export function useDaemon(): UseDaemonReturn {
  const queryClient = useQueryClient()
  const retryDelay = useRef(SSE_RECONNECT_BASE_MS)

  const workspacesQuery = useQuery<Workspace[], Error>({
    queryKey: daemonKeys.workspaces,
    queryFn: fetchWorkspaces,
    refetchInterval: 5000,
    retry: false,
  })

  const agentsQuery = useQuery<AgentEntry[], Error>({
    queryKey: daemonKeys.agents,
    queryFn: fetchAgents,
    refetchInterval: 5000,
    retry: false,
  })

  // SSE stream — invalidate workspace queries on workspace.* events,
  // write all events to the event ring buffer for useEventStream.
  // Reconnects with exponential backoff on error.
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
          // Append to event ring buffer
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

  const fetchError = workspacesQuery.error ?? agentsQuery.error ?? null
  const connected = !workspacesQuery.isError && workspacesQuery.data !== undefined

  const workspaces = (workspacesQuery.data ?? []).filter((ws) => !isTestWorkspace(ws))

  return {
    connected,
    workspaces,
    agents: agentsQuery.data ?? [],
    error: fetchError,
  }
}
