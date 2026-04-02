// Hook that provides session files from .ship-session/ via the shipd daemon.

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useCallback } from 'react'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { useDaemon } from '#/features/studio/hooks/useDaemon'
import { sessionKeys } from './query-keys'
import type { SessionFile, UploadResult } from './types'

const MAX_UPLOAD_BYTES = 10 * 1024 * 1024 // 10 MB

function useActiveWorkspaceId(): string {
  const { workspaces } = useDaemon()
  return workspaces.find((w) => w.status === 'active')?.branch ?? 'v0.2.0'
}

export function useSessionFiles() {
  const wsId = useActiveWorkspaceId()

  const query = useQuery({
    queryKey: sessionKeys.files(),
    queryFn: async (): Promise<SessionFile[]> => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files`)
      if (!res.ok) throw new Error(`daemon: session-files ${res.status}`)
      const body = (await res.json()) as { ok: boolean; data: { files: Array<{ name: string; path: string; size: number; type: string }> } }
      return body.data.files.map((f) => ({
        name: f.name,
        path: f.path,
        type: f.type as SessionFile['type'],
        size: f.size,
        modifiedAt: Date.now(),
      }))
    },
    staleTime: 3_000,
    refetchInterval: 5_000,
  })

  return {
    files: query.data ?? [],
    isLoading: query.isLoading,
  }
}

export function useSessionFileContent(filePath: string | null) {
  const wsId = useActiveWorkspaceId()

  return useQuery({
    queryKey: sessionKeys.fileContent(filePath ?? ''),
    queryFn: async (): Promise<string> => {
      if (!filePath) return ''
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(filePath)}`)
      if (!res.ok) throw new Error(`daemon: read session file ${res.status}`)
      const body = (await res.json()) as { ok: boolean; data: { content: string } }
      return body.data.content
    },
    enabled: filePath != null,
    staleTime: 3_000,
    refetchInterval: 5_000,
  })
}

const TODO_PATH = 'todo.md'

export function useSessionTodo() {
  const wsId = useActiveWorkspaceId()
  const queryClient = useQueryClient()

  const query = useQuery({
    queryKey: sessionKeys.fileContent(TODO_PATH),
    queryFn: async (): Promise<string | null> => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(TODO_PATH)}`)
      if (!res.ok) return null
      const body = (await res.json()) as { ok: boolean; data: { content: string } }
      return body.data.content
    },
    staleTime: 3_000,
    refetchInterval: 10_000,
  })

  const writeMutation = useMutation({
    mutationFn: async (content: string) => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(TODO_PATH)}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      })
      if (!res.ok) throw new Error(`daemon: write todo ${res.status}`)
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
  const wsId = useActiveWorkspaceId()
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (path: string) => {
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(path)}`, {
        method: 'DELETE',
      })
      if (!res.ok) throw new Error(`daemon: delete session file ${res.status}`)
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: sessionKeys.files() })
    },
  })
}

export function useUploadSessionFile() {
  const wsId = useActiveWorkspaceId()
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: async (file: File): Promise<UploadResult> => {
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
      const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(file.name)}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ content }),
      })
      if (!res.ok) throw new Error(`daemon: upload session file ${res.status}`)
      return { success: true }
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: sessionKeys.files() })
    },
  })
}
