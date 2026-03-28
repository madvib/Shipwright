// Hook that provides session files from .ship-session/ via MCP.
// Falls back to placeholder data when CLI is not connected.

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useCallback } from 'react'
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
        const entries = JSON.parse(raw) as Array<{ path: string; size: number; modified: string; type: string }>
        return entries.map((f) => ({
          name: f.path.split('/').pop() ?? f.path,
          path: f.path,
          type: classifyFile(f.path),
          size: f.size,
          modifiedAt: new Date(f.modified).getTime(),
        }))
      } catch {
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

const TODO_PATH = 'todo.md'

export function useSessionTodo() {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'
  const queryClient = useQueryClient()

  const query = useQuery({
    queryKey: sessionKeys.fileContent(TODO_PATH),
    queryFn: async (): Promise<string | null> => {
      if (!mcp) return null
      try {
        const raw = await mcp.callTool('read_session_file', { path: TODO_PATH })
        if (raw.startsWith('Error:')) return null
        return raw
      } catch {
        return null
      }
    },
    enabled: isConnected,
    staleTime: 3_000,
    refetchInterval: 10_000,
  })

  const writeMutation = useMutation({
    mutationFn: async (content: string) => {
      if (!mcp) throw new Error('Not connected')
      await mcp.callTool('write_session_file', { path: TODO_PATH, content })
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: sessionKeys.fileContent(TODO_PATH) })
    },
  })

  const writeTodo = useCallback(
    (content: string) => writeMutation.mutate(content),
    [writeMutation],
  )

  return {
    content: query.data ?? null,
    exists: query.data != null,
    isLoading: query.isLoading,
    writeTodo,
    isSaving: writeMutation.isPending,
  }
}
