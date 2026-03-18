import { useEffect, useRef, useState, useSyncExternalStore } from 'react'
import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useAuth } from '#/lib/components/protected-route'
import { studioKeys } from '#/lib/query-keys'
import { fetchApi } from '#/lib/api-errors'
import type { SyncStatusValue } from '#/features/studio/SyncStatus'

const LOCAL_STORAGE_KEY = 'ship-studio-v1'
const DEBOUNCE_MS = 2000

interface ServerWorkspace {
  id: string
  name: string
  branch: string
  status: string
  created_at: number
}

function getSnapshot(): string | null {
  return window.localStorage.getItem(LOCAL_STORAGE_KEY)
}

function subscribeStorage(cb: () => void): () => void {
  const handler = (e: StorageEvent) => {
    if (e.key === LOCAL_STORAGE_KEY) cb()
  }
  window.addEventListener('storage', handler)
  return () => window.removeEventListener('storage', handler)
}

/**
 * Sync library state to the server when authenticated.
 * Observes localStorage (written by useLibrary via StorageEvent)
 * and debounces a POST to /api/workspaces on changes.
 *
 * Falls back to localStorage-only when not authenticated.
 * Mount in the studio layout to enable server persistence.
 */
export function useLibrarySync() {
  const { isAuthenticated, isPending: authPending } = useAuth()
  const queryClient = useQueryClient()
  const syncedRef = useRef(false)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const initialRef = useRef(true)
  const [syncStatus, setSyncStatus] = useState<SyncStatusValue>('idle')

  const snapshot = useSyncExternalStore(subscribeStorage, getSnapshot, () => null)

  // Fetch workspaces when authenticated
  const { data: workspaces } = useQuery({
    queryKey: studioKeys.workspaces(),
    queryFn: () =>
      fetchApi<{ workspaces: ServerWorkspace[] }>('/api/workspaces', {
        credentials: 'include',
      }),
    enabled: isAuthenticated && !authPending,
  })

  // Mark synced once workspaces load or immediately if unauthenticated
  useEffect(() => {
    if (authPending || syncedRef.current) return
    if (!isAuthenticated) {
      syncedRef.current = true
      return
    }
    if (workspaces) syncedRef.current = true
  }, [authPending, isAuthenticated, workspaces])

  // Debounced POST when authenticated and library state changes
  useEffect(() => {
    // Skip initial snapshot — only sync on subsequent changes
    if (initialRef.current) {
      initialRef.current = false
      return
    }
    if (!isAuthenticated || authPending || !snapshot) return

    if (debounceRef.current) clearTimeout(debounceRef.current)

    setSyncStatus('saving')

    let modeName = 'untitled'
    try {
      const parsed = JSON.parse(snapshot) as { modeName?: string }
      if (parsed.modeName) modeName = parsed.modeName
    } catch { /* use default */ }

    debounceRef.current = setTimeout(() => {
      void fetchApi<{ workspace: ServerWorkspace }>('/api/workspaces', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ name: modeName }),
      })
        .then(() => {
          queryClient.invalidateQueries({ queryKey: studioKeys.workspaces() })
          setSyncStatus('saved')
        })
        .catch(() => {
          // Server sync failed — localStorage is the fallback. Do not surface error.
          setSyncStatus('idle')
        })
    }, DEBOUNCE_MS)

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current)
    }
  }, [snapshot, isAuthenticated, authPending, queryClient])

  return {
    isSynced: isAuthenticated && syncedRef.current,
    serverWorkspaces: workspaces?.workspaces ?? [],
    syncStatus,
  }
}
