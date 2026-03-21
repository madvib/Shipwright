// Unified agent store: localStorage + server sync via useSyncExternalStore.

import { useCallback, useEffect, useRef, useState, useSyncExternalStore } from 'react'
import { toast } from 'sonner'
import { useAuth } from '#/lib/components/protected-route'
import { fetchAgents, createAgentApi, updateAgentApi, deleteAgentApi } from './agent-api'
import type { AgentProfile } from './types'
import { DEFAULT_SETTINGS } from './types'
import { DEFAULT_PERMISSIONS } from '@ship/ui'
import { hasMigrated, migrateFromV1, finalizeMigration } from './migrate-storage'

const STORAGE_KEY = 'ship-agents-v2'
const DEBOUNCE_MS = 2000

export interface AgentSyncStatus {
  status: 'syncing' | 'synced' | 'error' | 'offline'
  lastSyncedAt?: number
}

interface StoreState { agents: AgentProfile[]; activeId: string | null }

function emptyState(): StoreState {
  return { agents: [], activeId: null }
}

function loadState(): StoreState {
  try {
    const raw = typeof window !== 'undefined'
      ? window.localStorage.getItem(STORAGE_KEY)
      : null
    if (raw) return JSON.parse(raw) as StoreState

    // No V2 data — attempt one-time migration from V1 keys
    if (!hasMigrated()) {
      const migrated = migrateFromV1()
      if (migrated.agents.length > 0) {
        const state: StoreState = { agents: migrated.agents, activeId: migrated.activeId }
        saveState(state)
        finalizeMigration()
        return state
      }
      finalizeMigration()
    }
    return emptyState()
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

export function makeAgent(partial?: Partial<AgentProfile>): AgentProfile {
  return {
    id: partial?.id ?? generateId(),
    name: partial?.name ?? 'New Agent',
    description: partial?.description ?? '',
    providers: partial?.providers ?? ['claude'],
    version: partial?.version ?? '0.1.0',
    skills: partial?.skills ?? [],
    mcpServers: partial?.mcpServers ?? [],
    subagents: partial?.subagents ?? [],
    permissions: partial?.permissions ?? { ...DEFAULT_PERMISSIONS },
    permissionPreset: partial?.permissionPreset ?? 'ship-guarded',
    settings: partial?.settings ?? { ...DEFAULT_SETTINGS },
    hooks: partial?.hooks ?? [],
    rules: partial?.rules ?? [],
    mcpToolStates: partial?.mcpToolStates ?? {},
    maxTurns: partial?.maxTurns,
    providerSettings: partial?.providerSettings,
  }
}

function mergeAgents(
  local: AgentProfile[],
  server: AgentProfile[],
): AgentProfile[] {
  const merged = new Map<string, AgentProfile>()

  // Server wins for ID conflicts
  for (const a of server) merged.set(a.id, a)
  // Local-only entries are preserved
  for (const a of local) {
    if (!merged.has(a.id)) merged.set(a.id, a)
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

  const getAgent = useCallback((id: string): AgentProfile | undefined => {
    return state.agents.find((a) => a.id === id)
  }, [state.agents])

  const createAgent = useCallback(
    (partial?: Partial<AgentProfile>): string => {
      const agent = makeAgent(partial)
      setState((prev) => ({
        agents: [...prev.agents, agent],
        activeId: agent.id,
      }))
      return agent.id
    },
    [setState],
  )

  const updateAgent = useCallback(
    (id: string, patch: Partial<AgentProfile>) => {
      setState((prev) => {
        const exists = prev.agents.some((a) => a.id === id)
        if (exists) {
          return {
            ...prev,
            agents: prev.agents.map((a) =>
              a.id === id ? { ...a, ...patch } : a,
            ),
          }
        }
        // Upsert: create agent with the patch if it doesn't exist
        const agent = makeAgent({ id, ...patch })
        return { agents: [...prev.agents, agent], activeId: prev.activeId ?? agent.id }
      })
    },
    [setState],
  )

  const deleteAgent = useCallback(
    (id: string) => {
      setState((prev) => {
        const agents = prev.agents.filter((a) => a.id !== id)
        const activeId = prev.activeId === id
          ? (agents[0]?.id ?? null)
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

        saveState({ agents: merged, activeId: local.activeId ?? merged[0]?.id ?? null })
        lastSyncedRef.current = JSON.stringify(merged)

        // Push local-only agents to server
        const serverIds = new Set(serverAgents.map((a) => a.id))
        const localOnly = merged.filter((a) => !serverIds.has(a.id))
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

async function syncToServer(agents: AgentProfile[]): Promise<void> {
  const serverAgents = await fetchAgents()
  const serverMap = new Map(serverAgents.map((a) => [a.id, a]))
  const localMap = new Map(agents.map((a) => [a.id, a]))

  const ops: Promise<void>[] = []

  // Create or update local agents on server
  for (const agent of agents) {
    if (serverMap.has(agent.id)) {
      ops.push(updateAgentApi(agent.id, agent))
    } else {
      ops.push(createAgentApi(agent))
    }
  }

  // Delete server agents not present locally
  for (const sa of serverAgents) {
    if (!localMap.has(sa.id)) {
      ops.push(deleteAgentApi(sa.id))
    }
  }

  await Promise.all(ops)
}
