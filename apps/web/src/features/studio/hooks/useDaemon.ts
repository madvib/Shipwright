// Hook for connecting to the shipd daemon REST/SSE API at port 51742.
// Provides live workspace and agent state without going through MCP.

import { useEffect } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import type { Workspace } from '@ship/ui'

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

const daemonKeys = {
  workspaces: ['daemon', 'workspaces'] as const,
  agents: ['daemon', 'agents'] as const,
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

  // SSE stream — invalidate workspace queries on workspace.* events.
  // On error: close and don't reconnect; the 5s poll fallback handles recovery.
  useEffect(() => {
    const es = new EventSource(`${DAEMON_BASE_URL}/api/runtime/events`)

    es.addEventListener('ship.event', (e: MessageEvent) => {
      try {
        const envelope = JSON.parse(e.data as string) as { event_type?: string }
        if (envelope.event_type?.startsWith('workspace.')) {
          void queryClient.invalidateQueries({ queryKey: daemonKeys.workspaces })
        }
      } catch {
        // malformed payload — ignore
      }
    })

    es.onerror = () => {
      es.close()
    }

    return () => {
      es.close()
    }
  }, [queryClient])

  const fetchError = workspacesQuery.error ?? agentsQuery.error ?? null
  const connected = !workspacesQuery.isError && workspacesQuery.data !== undefined

  return {
    connected,
    workspaces: workspacesQuery.data ?? [],
    agents: agentsQuery.data ?? [],
    error: fetchError,
  }
}
