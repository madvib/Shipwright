import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useLocalMcpContext } from './LocalMcpContext'
import { mcpKeys } from '#/lib/query-keys'
import type {
  PullResponse,
  ListAgentsResponse,
  TransferBundle,
} from '@ship/ui'

// ── Helpers ─────────────────────────────────────────────────────────────

function useMcpCallTool() {
  const mcp = useLocalMcpContext()
  if (!mcp) throw new Error('useMcpCallTool requires LocalMcpProvider')
  return { callTool: mcp.callTool, status: mcp.status }
}

// ── Queries ─────────────────────────────────────────────────────────────

/** Fetch local agent IDs from CLI (.ship/agents/). */
export function useLocalAgentIds() {
  const { callTool, status } = useMcpCallTool()

  return useQuery({
    queryKey: mcpKeys.agentList(),
    queryFn: async (): Promise<ListAgentsResponse> => {
      const raw = await callTool('list_local_agents')
      return JSON.parse(raw) as ListAgentsResponse
    },
    enabled: status === 'connected',
    refetchInterval: 10_000,
    staleTime: 5_000,
  })
}

/** Pull all resolved agents from CLI (.ship/). */
export function usePullAgents() {
  const { callTool } = useMcpCallTool()

  return useQuery({
    queryKey: mcpKeys.pull(),
    queryFn: async (): Promise<PullResponse> => {
      const raw = await callTool('pull_agents')
      return JSON.parse(raw) as PullResponse
    },
    enabled: false, // manual-only via refetch()
    staleTime: 0,
  })
}

// ── Mutations ───────────────────────────────────────────────────────────

/** Push a transfer bundle to CLI (.ship/). */
export function usePushBundle() {
  const { callTool } = useMcpCallTool()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (bundle: TransferBundle): Promise<string> => {
      return callTool('push_bundle', { bundle: JSON.stringify(bundle) })
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.agents() })
    },
  })
}
