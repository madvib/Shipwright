// Merged agent hook: combines MCP pull data with draft overlays.
// This is the primary read path for agent data in the UI.

import { useMemo } from 'react'
import { usePullAgents } from '#/features/studio/mcp-queries'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { useAgentDrafts } from './useAgentDrafts'
import { pullAgentToResolved } from './pull-adapter'
import type { ResolvedAgentProfile } from './types'

const CACHE_KEY = 'ship-agents-pull-cache'

function loadCachedAgents(): ResolvedAgentProfile[] {
  try {
    const raw = typeof window !== 'undefined'
      ? window.localStorage.getItem(CACHE_KEY)
      : null
    if (!raw) return []
    return JSON.parse(raw) as ResolvedAgentProfile[]
  } catch {
    return []
  }
}

function cacheAgents(agents: ResolvedAgentProfile[]) {
  try {
    window.localStorage.setItem(CACHE_KEY, JSON.stringify(agents))
  } catch { /* storage full or unavailable */ }
}

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

  // Convert pull data to resolved format
  const pulledAgents = useMemo(() => {
    if (!pullQuery.data?.agents) return []
    return pullQuery.data.agents.map(pullAgentToResolved)
  }, [pullQuery.data])

  // Cache pulled agents for offline use
  useMemo(() => {
    if (pulledAgents.length > 0) cacheAgents(pulledAgents)
  }, [pulledAgents])

  // Use pulled agents when available, fall back to cache when disconnected
  const baseAgents = useMemo(() => {
    if (pulledAgents.length > 0) return pulledAgents
    if (!isConnected) return loadCachedAgents()
    return []
  }, [pulledAgents, isConnected])

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
