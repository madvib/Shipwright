// Hook for reading daemon state via TanStack Query.
// SSE connection is handled by DaemonEventListener (mounted once in root).

import { useQuery } from '@tanstack/react-query'
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

export const daemonKeys = {
  workspaces: ['daemon', 'workspaces'] as const,
  agents: ['daemon', 'agents'] as const,
}

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
