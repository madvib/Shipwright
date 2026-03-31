// Hook that provides session files from .ship-session/ via MCP.
// Falls back to placeholder data when CLI is not connected.

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useCallback } from 'react'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { sessionKeys } from './query-keys'
import type { SessionFile, UploadResult } from './types'

const MAX_UPLOAD_BYTES = 10 * 1024 * 1024 // 10 MB

function classifyFile(name: string): SessionFile['type'] {
  if (/\.html?$/i.test(name)) return 'html'
  if (/\.(png|jpe?g|gif|webp|svg)$/i.test(name)) return 'image'
  if (/\.md$/i.test(name)) return 'markdown'
  if (/\.url$/i.test(name)) return 'url'
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

export function useDeleteSessionFile() {
  const mcp = useLocalMcpContext()
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (path: string) => {
      if (!mcp) throw new Error('Not connected')
      return mcp.callTool('delete_session_file', { path })
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: sessionKeys.files() })
    },
  })
}

export function useUploadSessionFile() {
  const mcp = useLocalMcpContext()
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (file: File): Promise<UploadResult> => {
      if (!mcp) throw new Error('Not connected')
      if (file.size > MAX_UPLOAD_BYTES) {
        return { success: false, error: `File too large (${(file.size / 1024 / 1024).toFixed(1)} MB). Max 10 MB.` }
      }
      const isImage = file.type.startsWith('image/')
      let content: string
      if (isImage) {
        const buffer = await file.arrayBuffer()
        const bytes = new Uint8Array(buffer)
        let binary = ''
        for (let i = 0; i < bytes.length; i++) binary += String.fromCharCode(bytes[i])
        const base64 = btoa(binary)
        content = `data:${file.type};base64,${base64}`
      } else {
        content = await file.text()
      }
      await mcp.callTool('write_session_file', { path: file.name, content })
      return { success: true }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: sessionKeys.files() })
    },
  })
}
