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

export interface SessionEntry {
  id: string
  workspace_id: string
  workspace_branch: string
  status: string
  started_at: string | null
  ended_at: string | null
  agent_id: string | null
  primary_provider: string | null
  goal: string | null
  summary: string | null
  tool_call_count: number
}

export interface UseDaemonReturn {
  connected: boolean
  workspaces: Workspace[]
  agents: AgentEntry[]
  sessions: SessionEntry[]
  jobs: JobRecord[]
  error: Error | null
}

// ---- Query keys ----

export const daemonKeys = {
  workspaces: ['daemon', 'workspaces'] as const,
  agents: ['daemon', 'agents'] as const,
  sessions: (wsId?: string) => ['daemon', 'sessions', wsId] as const,
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

async function fetchSessions(wsId?: string): Promise<SessionEntry[]> {
  const url = wsId
    ? `${DAEMON_BASE_URL}/api/runtime/sessions?workspace_id=${encodeURIComponent(wsId)}`
    : `${DAEMON_BASE_URL}/api/runtime/sessions`
  const res = await fetch(url)
  if (!res.ok) throw new Error(`daemon: sessions ${res.status}`)
  const body = await res.json() as { ok: boolean; data: { sessions: SessionEntry[] } }
  return body.data.sessions
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

  const activeWsId = (workspacesQuery.data ?? []).find((w) => w.status === 'active')?.branch
  const sessionsQuery = useQuery<SessionEntry[], Error>({
    queryKey: daemonKeys.sessions(activeWsId),
    queryFn: () => fetchSessions(activeWsId),
    refetchInterval: 10000,
    retry: false,
    enabled: !!activeWsId,
  })

  const jobsQuery = useQuery<JobRecord[], Error>({
    queryKey: daemonKeys.jobs,
    queryFn: fetchJobs,
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
    sessions: sessionsQuery.data ?? [],
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
