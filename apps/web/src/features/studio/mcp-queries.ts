import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useLocalMcpContext } from './LocalMcpContext'
import { mcpKeys } from '#/lib/query-keys'
import type {
  PullResponse,
  PullSkill,
  ListAgentsResponse,
  TransferBundle,
} from '@ship/ui'

// ── Helpers ─────────────────────────────────────────────────────────────

function useMcpCallTool() {
  const mcp = useLocalMcpContext()
  if (!mcp) {
    return {
      callTool: () => Promise.reject(new Error('No MCP connection')),
      status: 'disconnected' as const,
    }
  }
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
    staleTime: 5_000,
  })
}

/** Pull all resolved agents from CLI (.ship/). Auto-refetches when connected. */
export function usePullAgents() {
  const { callTool, status } = useMcpCallTool()
  const isConnected = status === 'connected'

  return useQuery({
    queryKey: mcpKeys.pull(),
    queryFn: async (): Promise<PullResponse> => {
      const raw = await callTool('pull_agents')
      return JSON.parse(raw) as PullResponse
    },
    enabled: isConnected,
    staleTime: 5_000,
  })
}

/** Fetch all project skills from .ship/skills/ regardless of agent references. */
export function useProjectSkills() {
  const { callTool, status } = useMcpCallTool()

  return useQuery({
    queryKey: mcpKeys.projectSkills(),
    queryFn: async (): Promise<PullSkill[]> => {
      const raw = await callTool('list_project_skills')
      return JSON.parse(raw) as PullSkill[]
    },
    enabled: status === 'connected',
    staleTime: 5_000,
  })
}

// ── Mutations ───────────────────────────────────────────────────────────

/** Push a transfer bundle to CLI (.ship/). Invalidates pull query on success. */
export function usePushBundle() {
  const { callTool } = useMcpCallTool()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (bundle: TransferBundle): Promise<string> => {
      return callTool('push_bundle', { bundle: JSON.stringify(bundle) })
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.pull() })
      void queryClient.invalidateQueries({ queryKey: mcpKeys.agents() })
    },
  })
}

/** Write a single skill file to disk via CLI. Invalidates pull on success. */
export function useSaveSkillFile() {
  const { callTool } = useMcpCallTool()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({
      skillId,
      filePath,
      content,
    }: {
      skillId: string
      filePath: string
      content: string
    }): Promise<string> => {
      return callTool('write_skill_file', {
        skill_id: skillId,
        file_path: filePath,
        content,
      })
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.pull() })
    },
  })
}

/** Delete a single skill file from disk via CLI. Invalidates all MCP queries on success. */
export function useDeleteSkillFile() {
  const { callTool } = useMcpCallTool()
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async ({ skillId, filePath }: { skillId: string; filePath: string }) => {
      return callTool('delete_skill_file', { skill_id: skillId, file_path: filePath })
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: mcpKeys.all })
    },
  })
}
