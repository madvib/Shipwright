import { useQuery } from '@tanstack/react-query';
import { commands, type McpRegistryEntry } from '@/bindings';

interface UseMcpRegistrySearchOptions {
  query: string;
  enabled?: boolean;
  limit?: number;
}

export function useMcpRegistrySearch({
  query,
  enabled = true,
  limit = 20,
}: UseMcpRegistrySearchOptions) {
  const normalizedQuery = query.trim();
  return useQuery({
    queryKey: ['mcp-registry', normalizedQuery.toLowerCase(), limit],
    queryFn: async () => {
      if (normalizedQuery.length < 2) return [] as McpRegistryEntry[];
      const result = await commands.searchMcpRegistryCmd(normalizedQuery, limit);
      if (result.status === 'error') throw new Error(result.error);
      return result.data;
    },
    enabled: enabled && normalizedQuery.length >= 2,
    staleTime: 120_000,
  });
}
