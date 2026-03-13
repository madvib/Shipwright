import { useMemo } from 'react';
import { useQuery } from '@tanstack/react-query';
import { commands, type ProviderInfo, type Skill } from '@/bindings';
import { listMcpServersCmd } from '@/lib/platform/tauri/commands';

export interface AgentLocalMcpOption {
  id: string;
  label: string;
}

interface UseAgentAssetInventoryOptions {
  enabled?: boolean;
  includeProviders?: boolean;
  includeMcpServers?: boolean;
  includeSkills?: boolean;
  skillScope?: 'project' | 'user' | null;
}

interface AgentAssetInventory {
  providers: ProviderInfo[];
  mcpServers: AgentLocalMcpOption[];
  skills: Skill[];
  loading: boolean;
  error: string | null;
  providersPending: boolean;
  providersError: boolean;
  refreshProviders: () => void;
}

function normalizeMcpOptions(input: Awaited<ReturnType<typeof listMcpServersCmd>>): AgentLocalMcpOption[] {
  return input
    .map((server) => {
      const id = (server.id ?? '').trim();
      const label = (server.name ?? id).trim();
      return { id, label };
    })
    .filter((server) => server.id.length > 0)
    .sort((left, right) => left.id.localeCompare(right.id));
}

export function useAgentAssetInventory({
  enabled = true,
  includeProviders = true,
  includeMcpServers = true,
  includeSkills = true,
  skillScope = null,
}: UseAgentAssetInventoryOptions = {}): AgentAssetInventory {
  const providersQuery = useQuery({
    queryKey: ['providers'],
    queryFn: async () => {
      const res = await commands.listProvidersCmd();
      if (res.status === 'error') throw new Error(res.error);
      return res.data;
    },
    enabled: enabled && includeProviders,
    staleTime: 60_000,
  });

  const mcpServersQuery = useQuery({
    queryKey: ['local-mcp-servers'],
    queryFn: async () => normalizeMcpOptions(await listMcpServersCmd()),
    enabled: enabled && includeMcpServers,
    staleTime: 60_000,
  });

  const skillsQuery = useQuery({
    queryKey: ['skills', skillScope ?? 'all'],
    queryFn: async () => {
      const res = await commands.listSkillsCmd(skillScope);
      if (res.status === 'error') throw new Error(res.error);
      return res.data.sort((left, right) => left.id.localeCompare(right.id));
    },
    enabled: enabled && includeSkills,
    staleTime: 60_000,
  });

  const loading = providersQuery.isLoading || mcpServersQuery.isLoading || skillsQuery.isLoading;
  const error = providersQuery.error ?? mcpServersQuery.error ?? skillsQuery.error ?? null;

  return useMemo(
    () => ({
      providers: providersQuery.data ?? [],
      mcpServers: mcpServersQuery.data ?? [],
      skills: skillsQuery.data ?? [],
      loading,
      error: error ? String(error) : null,
      providersPending: providersQuery.isPending,
      providersError: providersQuery.isError,
      refreshProviders: () => {
        void providersQuery.refetch();
      },
    }),
    [
      providersQuery.data,
      providersQuery.isPending,
      providersQuery.isError,
      providersQuery.refetch,
      mcpServersQuery.data,
      skillsQuery.data,
      loading,
      error,
    ],
  );
}
