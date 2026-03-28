// Hook that reads .ship-session/diff.txt via MCP for the diff viewer.
// Returns the raw diff text or null if unavailable.

import { useQuery } from '@tanstack/react-query'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { sessionKeys } from './query-keys'

const DIFF_PATH = 'diff.txt'

export function useDiffContent() {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'

  const query = useQuery({
    queryKey: sessionKeys.diff(),
    queryFn: async (): Promise<string | null> => {
      if (!mcp) return null
      try {
        const raw = await mcp.callTool('read_session_file', { path: DIFF_PATH })
        if (raw.startsWith('Error:')) return null
        return raw
      } catch {
        return null
      }
    },
    enabled: isConnected,
    staleTime: 5_000,
    refetchInterval: 10_000,
  })

  return {
    diffText: query.data ?? null,
    isLoading: query.isLoading,
  }
}
