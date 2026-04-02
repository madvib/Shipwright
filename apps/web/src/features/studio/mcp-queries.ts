// MCP query stubs — these previously used the MCP bridge to read/write
// agent and skill config files. Stubbed to return empty data until daemon
// endpoints for project config are available.

import { useQuery, useMutation } from '@tanstack/react-query'
import { mcpKeys } from '#/lib/query-keys'
import type {
  PullResponse,
  PullSkill,
  ListAgentsResponse,
  TransferBundle,
} from '@ship/ui'

// ── Queries ─────────────────────────────────────────────────────────────

// TODO: daemon endpoint
/** Fetch local agent IDs from CLI (.ship/agents/). */
export function useLocalAgentIds() {
  return useQuery({
    queryKey: mcpKeys.agentList(),
    queryFn: async (): Promise<ListAgentsResponse> => {
      return { agents: [] } as ListAgentsResponse
    },
    staleTime: 5_000,
  })
}

// TODO: daemon endpoint
/** Pull all resolved agents from CLI (.ship/). */
export function usePullAgents() {
  return useQuery({
    queryKey: mcpKeys.pull(),
    queryFn: async (): Promise<PullResponse> => {
      return { agents: [], skills: [] } as unknown as PullResponse
    },
    staleTime: 5_000,
  })
}

// TODO: daemon endpoint
/** Fetch all project skills from .ship/skills/. */
export function useProjectSkills() {
  return useQuery({
    queryKey: mcpKeys.projectSkills(),
    queryFn: async (): Promise<PullSkill[]> => {
      return []
    },
    staleTime: 5_000,
  })
}

// ── Mutations (no-op stubs) ────────────────────────────────────────────

// TODO: daemon endpoint
/** Push a transfer bundle to CLI (.ship/). */
export function usePushBundle() {
  return useMutation({
    mutationFn: async (_bundle: TransferBundle): Promise<string> => {
      console.warn('usePushBundle: stubbed — daemon endpoint not yet available')
      return ''
    },
  })
}

// TODO: daemon endpoint
/** Write a single skill file to disk via CLI. */
export function useSaveSkillFile() {
  return useMutation({
    mutationFn: async (_args: { skillId: string; filePath: string; content: string }): Promise<string> => {
      console.warn('useSaveSkillFile: stubbed — daemon endpoint not yet available')
      return ''
    },
  })
}

// TODO: daemon endpoint
/** Get merged var values for a skill. */
export function useSkillVars(skillId: string | null) {
  return useQuery({
    queryKey: mcpKeys.skillVars(skillId ?? ''),
    queryFn: async () => {
      return {} as Record<string, unknown>
    },
    enabled: skillId != null,
    staleTime: 5_000,
  })
}

// TODO: daemon endpoint
/** Set a single skill variable value. */
export function useSetSkillVar() {
  return useMutation({
    mutationFn: async (_args: { skillId: string; key: string; valueJson: string }) => {
      console.warn('useSetSkillVar: stubbed — daemon endpoint not yet available')
      return ''
    },
  })
}

// TODO: daemon endpoint
/** Delete a single skill file from disk via CLI. */
export function useDeleteSkillFile() {
  return useMutation({
    mutationFn: async (_args: { skillId: string; filePath: string }) => {
      console.warn('useDeleteSkillFile: stubbed — daemon endpoint not yet available')
      return ''
    },
  })
}
