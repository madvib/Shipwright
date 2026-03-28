// Hook that provides session files from .ship-session/ via MCP.
// Falls back to placeholder data when CLI is not connected.

import { useQuery } from '@tanstack/react-query'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { sessionKeys } from './query-keys'
import type { SessionFile } from './types'

function classifyFile(name: string): SessionFile['type'] {
  if (/\.html?$/i.test(name)) return 'html'
  if (/\.(png|jpe?g|gif|webp|svg)$/i.test(name)) return 'image'
  if (/\.md$/i.test(name)) return 'markdown'
  return 'other'
}

export function useSessionFiles() {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'

  const query = useQuery({
    queryKey: sessionKeys.files(),
    queryFn: async (): Promise<SessionFile[]> => {
      if (!mcp) return []
      try {
        const raw = await mcp.callTool('list_session_files')
        const parsed = JSON.parse(raw) as { files: Array<{ name: string; path: string; modified_at: number }> }
        return parsed.files.map((f) => ({
          name: f.name,
          path: f.path,
          type: classifyFile(f.name),
          modifiedAt: f.modified_at,
        }))
      } catch {
        // Tool may not exist yet — return empty
        return []
      }
    },
    enabled: isConnected,
    staleTime: 3_000,
    refetchInterval: 5_000,
  })

  return {
    files: query.data ?? [],
    isLoading: query.isLoading,
    isConnected: isConnected ?? false,
  }
}

export function useSessionFileContent(filePath: string | null) {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'

  return useQuery({
    queryKey: sessionKeys.fileContent(filePath ?? ''),
    queryFn: async (): Promise<string> => {
      if (!mcp || !filePath) return ''
      try {
        const raw = await mcp.callTool('read_session_file', { path: filePath })
        return raw
      } catch {
        return ''
      }
    },
    enabled: isConnected && filePath != null,
    staleTime: 3_000,
    refetchInterval: 5_000,
  })
}
