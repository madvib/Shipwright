// Unified agent store: localStorage + server sync via useSyncExternalStore.

import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from 'react'
import { toast } from 'sonner'
import { useAuth } from '#/lib/components/protected-route'
import { fetchAgents, createAgentApi, updateAgentApi, deleteAgentApi } from './agent-api'
import type { ResolvedAgentProfile } from './types'

const STORAGE_KEY = 'ship-agents-v2'
const DEBOUNCE_MS = 2000

export interface AgentSyncStatus {
  status: 'syncing' | 'synced' | 'error' | 'offline'
  lastSyncedAt?: number
}

interface StoreState { agents: ResolvedAgentProfile[]; activeId: string | null }

function emptyState(): StoreState {
  return { agents: [], activeId: null }
}

function loadState(): StoreState {
  try {
    const raw = typeof window !== 'undefined'
      ? window.localStorage.getItem(STORAGE_KEY)
      : null
    if (!raw) return emptyState()
    const parsed = JSON.parse(raw) as StoreState
    // Validate shape: if agents don't have profile.id, data is stale -- drop it
    if (parsed.agents?.length > 0 && !parsed.agents[0].profile?.id) {
      return emptyState()
    }
    return parsed
  } catch {
    return emptyState()
  }
}

function saveState(state: StoreState): void {
  try {
    const value = JSON.stringify(state)
    window.localStorage.setItem(STORAGE_KEY, value)
    window.dispatchEvent(
      new StorageEvent('storage', { key: STORAGE_KEY, newValue: value }),
    )
  } catch { /* storage full or unavailable */ }
}

function getSnapshot(): string {
  return window.localStorage.getItem(STORAGE_KEY) ?? JSON.stringify(emptyState())
}

function getServerSnapshot(): string {
  return JSON.stringify(emptyState())
}

function subscribe(cb: () => void): () => void {
  const handler = (e: StorageEvent) => {
    if (e.key === STORAGE_KEY) cb()
  }
  window.addEventListener('storage', handler)
  return () => window.removeEventListener('storage', handler)
}

function generateId(): string {
  const bytes = new Uint8Array(16)
  crypto.getRandomValues(bytes)
  return Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('')
}

export function makeAgent(partial?: Partial<ResolvedAgentProfile>): ResolvedAgentProfile {
  const pp = partial?.profile
  return {
    profile: {
      id: pp?.id || generateId(),
      name: pp?.name ?? 'New Agent',
      description: pp?.description ?? '',
      providers: pp?.providers ?? ['claude'],
      version: pp?.version ?? '0.1.0',
    },
    skills: partial?.skills ?? [],
    mcpServers: partial?.mcpServers ?? [],
    permissions: partial?.permissions ?? { preset: 'ship-standard' },
    hooks: partial?.hooks ?? [],
    rules: partial?.rules ?? [],
  }
}

function mergeAgents(
  local: ResolvedAgentProfile[],
  server: ResolvedAgentProfile[],
): ResolvedAgentProfile[] {
  const merged = new Map<string, ResolvedAgentProfile>()

  // Server wins for ID conflicts
  for (const a of server) merged.set(a.profile.id, a)
  // Local-only entries are preserved
  for (const a of local) {
    if (!merged.has(a.profile.id)) merged.set(a.profile.id, a)
  }

  return Array.from(merged.values())
}

export function useAgentStore() {
  const { isAuthenticated, isPending: authPending } = useAuth()
  const initialSyncDoneRef = useRef(false)
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const isFirstSnapshotRef = useRef(true)
  const lastSyncedRef = useRef<string | null>(null)
  const [syncStatus, setSyncStatus] = useState<AgentSyncStatus>({
    status: isAuthenticated ? 'synced' : 'offline',
  })

  const raw = useSyncExternalStore(subscribe, getSnapshot, getServerSnapshot)
  const state: StoreState = (() => {
    try { return JSON.parse(raw) as StoreState } catch { return emptyState() }
  })()

  const setState = useCallback((updater: (prev: StoreState) => StoreState) => {
    const current = loadState()
    const next = updater(current)
    saveState(next)
  }, [])

  const setActiveId = useCallback((id: string | null) => {
    setState((prev) => ({ ...prev, activeId: id }))
  }, [setState])

  const getAgent = useCallback((id: string): ResolvedAgentProfile | undefined => {
    return state.agents.find((a) => a.profile.id === id)
  }, [state.agents])

  const createAgent = useCallback(
    (partial?: Partial<ResolvedAgentProfile>): string => {
      const agent = makeAgent(partial)
      setState((prev) => ({
        agents: [...prev.agents, agent],
        activeId: agent.profile.id,
      }))
      return agent.profile.id
    },
    [setState],
  )

  const updateAgent = useCallback(
    (id: string, patch: Partial<ResolvedAgentProfile>) => {
      setState((prev) => {
        const exists = prev.agents.some((a) => a.profile.id === id)
        if (exists) {
          return {
            ...prev,
            agents: prev.agents.map((a) =>
              a.profile.id === id ? { ...a, ...patch } : a,
            ),
          }
        }
        // Upsert: create agent with the patch if it doesn't exist
        const agent = makeAgent({ profile: { id, name: id }, ...patch })
        return { agents: [...prev.agents, agent], activeId: prev.activeId ?? agent.profile.id }
      })
    },
    [setState],
  )

  const deleteAgent = useCallback(
    (id: string) => {
      setState((prev) => {
        const agents = prev.agents.filter((a) => a.profile.id !== id)
        const activeId = prev.activeId === id
          ? (agents[0]?.profile.id ?? null)
          : prev.activeId
        return { agents, activeId }
      })
    },
    [setState],
  )

  useEffect(() => {
    if (authPending) return
    if (!isAuthenticated) {
      setSyncStatus({ status: 'offline' })
      return
    }
    if (initialSyncDoneRef.current) return
    initialSyncDoneRef.current = true

    setSyncStatus({ status: 'syncing' })

    void (async () => {
      try {
        const serverAgents = await fetchAgents()
        const local = loadState()
        const merged = mergeAgents(local.agents, serverAgents)

        saveState({ agents: merged, activeId: local.activeId ?? merged[0]?.profile.id ?? null })
        lastSyncedRef.current = JSON.stringify(merged)

        // Push local-only agents to server
        const serverIds = new Set(serverAgents.map((a) => a.profile.id))
        const localOnly = merged.filter((a) => !serverIds.has(a.profile.id))
        await Promise.all(localOnly.map((a) => createAgentApi(a)))

        setSyncStatus({ status: 'synced', lastSyncedAt: Date.now() })
      } catch {
        // API unreachable -- localStorage is authoritative
        setSyncStatus({ status: 'error' })
        toast.error('Agent sync failed', { description: 'Changes saved locally.' })
      }
    })()
  }, [authPending, isAuthenticated])

  useEffect(() => {
    if (isFirstSnapshotRef.current) {
      isFirstSnapshotRef.current = false
      return
    }
    if (!isAuthenticated || authPending) return

    if (debounceRef.current) clearTimeout(debounceRef.current)

    setSyncStatus((prev) => ({ ...prev, status: 'syncing' }))

    debounceRef.current = setTimeout(() => {
      const current = loadState()
      const serialized = JSON.stringify(current.agents)

      // Skip if unchanged since last sync
      if (serialized === lastSyncedRef.current) {
        setSyncStatus((prev) => ({ ...prev, status: 'synced' }))
        return
      }
      lastSyncedRef.current = serialized

      void syncToServer(current.agents)
        .then(() => {
          setSyncStatus({ status: 'synced', lastSyncedAt: Date.now() })
        })
        .catch(() => {
          setSyncStatus((prev) => ({ ...prev, status: 'error' }))
          toast.error('Agent sync failed', { description: 'Changes saved locally.' })
        })
    }, DEBOUNCE_MS)

    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current)
    }
  }, [raw, isAuthenticated, authPending])

  return {
    agents: state.agents,
    activeId: state.activeId,
    syncStatus,
    setActiveId,
    getAgent,
    createAgent,
    updateAgent,
    deleteAgent,
  }
}

async function syncToServer(agents: ResolvedAgentProfile[]): Promise<void> {
  const serverAgents = await fetchAgents()
  const serverMap = new Map(serverAgents.map((a) => [a.profile.id, a]))
  const localMap = new Map(agents.map((a) => [a.profile.id, a]))

  const ops: Promise<void>[] = []

  // Create or update local agents on server
  for (const agent of agents) {
    if (serverMap.has(agent.profile.id)) {
      ops.push(updateAgentApi(agent.profile.id, agent))
    } else {
      ops.push(createAgentApi(agent))
    }
  }

  // Delete server agents not present locally
  for (const sa of serverAgents) {
    if (!localMap.has(sa.profile.id)) {
      ops.push(deleteAgentApi(sa.profile.id))
    }
  }

  await Promise.all(ops)
}
