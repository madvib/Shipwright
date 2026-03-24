// Agent store: localStorage persistence for agent creation and local state.
// MCP pull is now the source of truth for existing agents.
// This store handles agent creation and the legacy localStorage cache.

import { useCallback, useSyncExternalStore } from 'react'
import type { ResolvedAgentProfile } from './types'

const STORAGE_KEY = 'ship-agents-v2'

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
    if (!Array.isArray(parsed.agents)) return emptyState()
    const valid = parsed.agents.filter((a) => a?.profile?.id)
    return { agents: valid, activeId: parsed.activeId ?? valid[0]?.profile?.id ?? null }
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

function slugify(name: string): string {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '') || `agent-${Date.now()}`
}

export function makeAgent(partial?: Partial<ResolvedAgentProfile>): ResolvedAgentProfile {
  const pp = partial?.profile
  const name = pp?.name ?? 'New Agent'
  return {
    profile: {
      id: pp?.id || slugify(name),
      name,
      description: pp?.description ?? '',
      providers: pp?.providers ?? ['claude'],
      version: pp?.version ?? '0.1.0',
    },
    skills: partial?.skills ?? [],
    mcpServers: partial?.mcpServers ?? [],
    permissions: partial?.permissions ?? { preset: 'ship-standard' },
    hooks: partial?.hooks ?? [],
    rules: partial?.rules ?? [],
    model: partial?.model ?? null,
    env: partial?.env ?? null,
    availableModels: partial?.availableModels ?? null,
    agentLimits: partial?.agentLimits ?? null,
    providerSettings: partial?.providerSettings ?? {},
    toolPermissions: partial?.toolPermissions ?? {},
    source: partial?.source ?? 'project',
  }
}

export function useAgentStore() {
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

  return {
    agents: state.agents,
    activeId: state.activeId,
    setActiveId,
    getAgent,
    createAgent,
    updateAgent,
    deleteAgent,
  }
}
