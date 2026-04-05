// Hook for reading daemon state via TanStack Query.
// SSE connection is handled by DaemonEventListener (mounted once in root).

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import type { Workspace, JobRecord } from '@ship/ui'

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
  jobs: JobRecord[]
  error: Error | null
}

// ---- Query keys ----

export const daemonKeys = {
  workspaces: ['daemon', 'workspaces'] as const,
  agents: ['daemon', 'agents'] as const,
  jobs: ['daemon', 'jobs'] as const,
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

async function fetchJobs(): Promise<JobRecord[]> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/jobs`)
  if (!res.ok) throw new Error(`daemon: jobs ${res.status}`)
  const body = await res.json() as { ok: boolean; data: { jobs: JobRecord[] } }
  return body.data.jobs
}

export interface CreateJobInput {
  slug: string
  agent: string
  branch: string
  spec_path: string
  depends_on: string[] | null
}

async function postCreateJob(input: CreateJobInput): Promise<string> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/jobs`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(input),
  })
  if (!res.ok) throw new Error(`daemon: create job ${res.status}`)
  const body = await res.json() as { ok: boolean; data: { job_id: string } }
  return body.data.job_id
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

  const jobsQuery = useQuery<JobRecord[], Error>({
    queryKey: daemonKeys.jobs,
    queryFn: fetchJobs,
    refetchInterval: 5000,
    retry: false,
  })

  const fetchError = workspacesQuery.error ?? agentsQuery.error ?? jobsQuery.error ?? null
  const connected = !workspacesQuery.isError && workspacesQuery.data !== undefined

  const workspaces = (workspacesQuery.data ?? []).filter((ws) => !isTestWorkspace(ws))

  return {
    connected,
    workspaces,
    agents: agentsQuery.data ?? [],
    jobs: jobsQuery.data ?? [],
    error: fetchError,
  }
}

export interface ViewEntry {
  name: string
}

async function fetchViews(): Promise<ViewEntry[]> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/runtime/views`)
  if (!res.ok) return []
  const body = await res.json() as { ok: boolean; data: { views: ViewEntry[] } }
  return body.data.views
}

export function useViews() {
  return useQuery<ViewEntry[], Error>({
    queryKey: ['daemon', 'views'],
    queryFn: fetchViews,
    staleTime: 30_000,
    retry: false,
  })
}

export function useCreateJob() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: postCreateJob,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: daemonKeys.jobs })
    },
  })
}
