// Hook that reads diff content from two sources:
// 1. get_git_diff MCP tool (preferred, live working tree diff)
// 2. .ship-session/diff.txt fallback (manual or agent-generated)

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

      // Try get_git_diff first
      try {
        const raw = await mcp.callTool('get_git_diff')
        if (raw && !raw.startsWith('Error:') && raw.trim().length > 0) {
          return raw
        }
      } catch {
        // Tool may not exist -- fall through
      }

      // Fall back to diff.txt
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
