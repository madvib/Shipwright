import { useSyncExternalStore } from 'react'

const LOCAL_STORAGE_KEY = 'ship-studio-v1'

export type SyncStatus = 'idle' | 'saving' | 'saved' | 'error'

function getSnapshot(): string | null {
  return window.localStorage.getItem(LOCAL_STORAGE_KEY)
}

function subscribe(cb: () => void): () => void {
  const handler = (e: StorageEvent) => { if (e.key === LOCAL_STORAGE_KEY) cb() }
  window.addEventListener('storage', handler)
  return () => window.removeEventListener('storage', handler)
}

/**
 * Library data lives in localStorage only.
 * Server sync (profiles, libraries tables) has been removed.
 * This hook provides the same API surface so callers do not need to change.
 */
export function useLibrarySync() {
  useSyncExternalStore(subscribe, getSnapshot, () => null)
  const syncStatus: SyncStatus = 'idle'
  return { syncStatus }
}
