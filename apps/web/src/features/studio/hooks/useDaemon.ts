// Hook for connecting to the shipd daemon REST/SSE API at port 51742.
// Provides live workspace and agent state without going through MCP.

import { useEffect } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'

// ---- Types derived from shipd runtime_api.rs response shapes ----

export interface WorkspaceEntry {
  id: string
  branch: string
  workspace_type: string
  status: string
  active_agent?: string
  worktree_path?: string
  providers: string[]
  mcp_servers: string[]
  skills: string[]
  is_worktree: boolean
}

export interface AgentEntry {
  agent_id: string
  label: string
  capabilities: string[]
  status: string
}

export interface UseDaemonReturn {
  connected: boolean
  workspaces: WorkspaceEntry[]
  agents: AgentEntry[]
  error: Error | null
}

// ---- Query keys ----

const daemonKeys = {
  workspaces: ['daemon', 'workspaces'] as const,
  agents: ['daemon', 'agents'] as const,
}

// ---- Fetchers ----

async function fetchWorkspaces(): Promise<WorkspaceEntry[]> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/workspaces`)
  if (!res.ok) throw new Error(`daemon: workspaces ${res.status}`)
  const body = await res.json() as { ok: boolean; data: { workspaces: WorkspaceEntry[] } }
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

  const workspacesQuery = useQuery<WorkspaceEntry[], Error>({
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

  // SSE stream — invalidate all daemon queries on any event.
  useEffect(() => {
    let es: EventSource | null = null
    let closed = false

    function open() {
      if (closed) return
      es = new EventSource(`${DAEMON_BASE_URL}/api/runtime/events`)
      es.addEventListener('ship.event', () => {
        void queryClient.invalidateQueries({ queryKey: ['daemon'] })
      })
      es.onerror = () => {
        es?.close()
        es = null
        // Backoff before reconnecting — daemon may not be running.
        if (!closed) setTimeout(open, 5000)
      }
    }

    open()

    return () => {
      closed = true
      es?.close()
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
