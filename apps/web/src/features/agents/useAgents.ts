// Merged agent hook: combines MCP pull data with draft overlays.
// This is the primary read path for agent data in the UI.

import { useMemo, useState, useEffect } from 'react'
import { usePullAgents } from '#/features/studio/mcp-queries'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { useAgentDrafts } from './useAgentDrafts'
import { pullAgentToResolved } from './pull-adapter'
import { idbGet, idbSet, migrateFromLocalStorage } from '#/lib/idb-cache'
import type { ResolvedAgentProfile } from './types'

const CACHE_KEY = 'ship-agents-pull-cache'

export interface UseAgentsReturn {
  agents: ResolvedAgentProfile[]
  isLoading: boolean
  isConnected: boolean
  getAgent: (id: string) => ResolvedAgentProfile | undefined
}

export function useAgents(): UseAgentsReturn {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'
  const pullQuery = usePullAgents()
  const { drafts } = useAgentDrafts()
  const [cachedAgents, setCachedAgents] = useState<ResolvedAgentProfile[]>([])

  // Load cache from IndexedDB on mount (migrate from localStorage if needed)
  useEffect(() => {
    let cancelled = false
    async function load() {
      try {
        const migrated = await migrateFromLocalStorage<ResolvedAgentProfile[]>(CACHE_KEY)
        if (migrated && !cancelled) { setCachedAgents(migrated); return }
        const data = await idbGet<ResolvedAgentProfile[]>(CACHE_KEY)
        if (data && !cancelled) setCachedAgents(data)
      } catch { /* IDB unavailable */ }
    }
    void load()
    return () => { cancelled = true }
  }, [])

  // Convert pull data to resolved format
  const pulledAgents = useMemo(() => {
    if (!pullQuery.data?.agents) return []
    return pullQuery.data.agents.map(pullAgentToResolved)
  }, [pullQuery.data])

  // Cache pulled agents to IndexedDB
  useEffect(() => {
    if (pulledAgents.length > 0) {
      setCachedAgents(pulledAgents)
      idbSet(CACHE_KEY, pulledAgents).catch(() => {})
    }
  }, [pulledAgents])

  // Use pulled agents when available, fall back to cache when disconnected
  const baseAgents = useMemo(() => {
    if (pulledAgents.length > 0) return pulledAgents
    if (!isConnected) return cachedAgents
    return []
  }, [pulledAgents, isConnected, cachedAgents])

  // Merge drafts on top of base agents
  const agents = useMemo(() => {
    const draftKeys = Object.keys(drafts)
    if (draftKeys.length === 0) return baseAgents
    return baseAgents.map((agent) => {
      const draft = drafts[agent.profile.id]
      if (!draft) return agent
      return mergeAgentWithDraft(agent, draft)
    })
  }, [baseAgents, drafts])

  const getAgent = useMemo(() => {
    const map = new Map(agents.map((a) => [a.profile.id, a]))
    return (id: string) => map.get(id)
  }, [agents])

  return {
    agents,
    isLoading: pullQuery.isLoading,
    isConnected,
    getAgent,
  }
}

function mergeAgentWithDraft(
  agent: ResolvedAgentProfile,
  draft: Partial<ResolvedAgentProfile>,
): ResolvedAgentProfile {
  const merged = { ...agent, ...draft }
  // Deep merge profile if present in draft
  if (draft.profile) {
    merged.profile = { ...agent.profile, ...draft.profile }
  }
  return merged as ResolvedAgentProfile
}
