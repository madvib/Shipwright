import { useEffect, useRef, useState, useSyncExternalStore } from 'react'
import { useAuth } from '#/lib/components/protected-route'
import { fetchApi } from '#/lib/api-errors'

const LOCAL_STORAGE_KEY = 'ship-studio-v1'
const LIBRARY_ID_KEY = 'ship-studio-library-id'
const DEBOUNCE_MS = 2000

export type SyncStatus = 'idle' | 'saving' | 'saved' | 'error'

interface ServerLibrary {
  id: string
  org_id: string
  user_id: string
  name: string
  slug: string
  data: unknown
  created_at: string
  updated_at: string
}

function getSnapshot(): string | null {
  return window.localStorage.getItem(LOCAL_STORAGE_KEY)
}

function subscribe(cb: () => void): () => void {
  const handler = (e: StorageEvent) => { if (e.key === LOCAL_STORAGE_KEY) cb() }
  window.addEventListener('storage', handler)
  return () => window.removeEventListener('storage', handler)
}

function parseSnapshot(raw: string): { name: string; data: unknown } {
  let parsed: Record<string, unknown> = {}
  try { parsed = JSON.parse(raw) as Record<string, unknown> } catch { /* default */ }
  return {
    name: typeof parsed.modeName === 'string' ? parsed.modeName : 'untitled',
    data: parsed,
  }
}

async function createLibrary(name: string, data: unknown): Promise<ServerLibrary> {
  const { library } = await fetchApi<{ library: ServerLibrary }>('/api/libraries', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify({ name, data }),
  })
  return library
}

/**
 * Sync library to the server when authenticated. Falls back to localStorage-only
 * when unauthenticated or the API is unreachable (silent fallback — no errors shown).
 *
 * On mount: GET /api/libraries. Server wins if data exists (seeds localStorage).
 * If empty, push local content up. After that, local changes are the source of truth.
 */
export function useLibrarySync() {
  const { isAuthenticated, isPending: authPending } = useAuth()
  const [syncStatus, setSyncStatus] = useState<SyncStatus>('idle')
  const libraryIdRef = useRef<string | null>(null)
  const initialSyncDoneRef = useRef(false)
  const isFirstSnapshotRef = useRef(true)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const snapshot = useSyncExternalStore(subscribe, getSnapshot, () => null)

  // Restore persisted library id on mount
  useEffect(() => {
    const stored = window.localStorage.getItem(LIBRARY_ID_KEY)
    if (stored) libraryIdRef.current = stored
  }, [])

  // Initial server sync: server wins if data exists, else push local up
  useEffect(() => {
    if (authPending || !isAuthenticated || initialSyncDoneRef.current) return
    initialSyncDoneRef.current = true

    void (async () => {
      try {
        const { libraries } = await fetchApi<{ libraries: ServerLibrary[] }>(
          '/api/libraries', { credentials: 'include' },
        )

        if (libraries.length > 0) {
          const serverLib = libraries[0]
          libraryIdRef.current = serverLib.id
          window.localStorage.setItem(LIBRARY_ID_KEY, serverLib.id)
          const value = JSON.stringify(serverLib.data)
          window.localStorage.setItem(LOCAL_STORAGE_KEY, value)
          window.dispatchEvent(new StorageEvent('storage', { key: LOCAL_STORAGE_KEY, newValue: value }))
        } else {
          const raw = window.localStorage.getItem(LOCAL_STORAGE_KEY)
          if (!raw) return
          const { name, data } = parseSnapshot(raw)
          const lib = await createLibrary(name, data)
          libraryIdRef.current = lib.id
          window.localStorage.setItem(LIBRARY_ID_KEY, lib.id)
        }
      } catch {
        // API unreachable — localStorage is authoritative
        const stored = window.localStorage.getItem(LIBRARY_ID_KEY)
        if (stored) libraryIdRef.current = stored
      }
    })()
  }, [authPending, isAuthenticated])

  // Debounced PUT on library changes when authenticated
  useEffect(() => {
    if (isFirstSnapshotRef.current) { isFirstSnapshotRef.current = false; return }
    if (!isAuthenticated || authPending || !snapshot) return

    if (debounceRef.current) clearTimeout(debounceRef.current)
    setSyncStatus('saving')

    debounceRef.current = setTimeout(() => {
      const libId = libraryIdRef.current
      if (!libId) { setSyncStatus('idle'); return }

      const { name, data } = parseSnapshot(snapshot)

      void fetchApi<{ library: ServerLibrary }>(`/api/libraries/${libId}`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ name, data }),
      })
        .then(() => { setSyncStatus('saved') })
        .catch(async (err: unknown) => {
          if ((err as { status?: number }).status === 404) {
            try {
              const lib = await createLibrary(name, data)
              libraryIdRef.current = lib.id
              window.localStorage.setItem(LIBRARY_ID_KEY, lib.id)
              setSyncStatus('saved')
            } catch { setSyncStatus('idle') }
          } else {
            setSyncStatus('idle')
          }
        })
    }, DEBOUNCE_MS)

    return () => { if (debounceRef.current) clearTimeout(debounceRef.current) }
  }, [snapshot, isAuthenticated, authPending])

  return { syncStatus }
}
