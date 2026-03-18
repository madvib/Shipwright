import { useEffect, useRef, useCallback } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { useAuth } from '#/lib/components/protected-route'
import { studioKeys } from '#/lib/query-keys'
import { fetchApi } from '#/lib/api-errors'
import type { ProjectLibrary } from '#/features/compiler/types'

const LOCAL_STORAGE_KEY = 'ship-studio-v1'

interface StoredLibrary {
  library: ProjectLibrary
  modeName: string
  selectedProviders: string[]
}

interface ServerWorkspace {
  id: string
  name: string
  branch: string
  status: string
  created_at: number
}

/**
 * Sync library state to the server when authenticated.
 * Falls back to localStorage-only when not authenticated.
 *
 * This hook does NOT replace useLibrary — it layers on top of it.
 * Call it from the studio layout to enable server persistence.
 */
export function useLibrarySync(current: StoredLibrary) {
  const { isAuthenticated, isPending: authPending } = useAuth()
  const queryClient = useQueryClient()
  const syncedRef = useRef(false)

  // Fetch workspaces when authenticated
  const { data: workspaces } = useQuery({
    queryKey: studioKeys.workspaces(),
    queryFn: () => fetchApi<{ workspaces: ServerWorkspace[] }>('/api/workspaces', {
      credentials: 'include',
    }),
    enabled: isAuthenticated && !authPending,
  })

  // Save workspace mutation
  const saveMutation = useMutation({
    mutationFn: (payload: { name: string; branch?: string }) =>
      fetchApi<{ workspace: ServerWorkspace }>('/api/workspaces', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify(payload),
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: studioKeys.workspaces() })
    },
  })

  // On mount, when authenticated and workspaces loaded, mark as synced.
  // Future: merge server state with local state here.
  useEffect(() => {
    if (authPending || syncedRef.current) return
    if (!isAuthenticated) {
      syncedRef.current = true
      return
    }
    if (workspaces) {
      syncedRef.current = true
    }
  }, [authPending, isAuthenticated, workspaces])

  // Auto-save to localStorage always (baseline persistence)
  useEffect(() => {
    try {
      window.localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(current))
    } catch { /* quota exceeded — ignore */ }
  }, [current])

  const saveToServer = useCallback(
    (name: string) => {
      if (!isAuthenticated) return
      saveMutation.mutate({ name })
    },
    [isAuthenticated, saveMutation],
  )

  return {
    isSyncing: saveMutation.isPending,
    isSynced: isAuthenticated && syncedRef.current,
    serverWorkspaces: workspaces?.workspaces ?? [],
    saveToServer,
  }
}
