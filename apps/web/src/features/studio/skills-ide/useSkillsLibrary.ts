// Skills library: aggregates skills from MCP pull data into a deduplicated view.
// Skills come from pulled agents (PullAgent.skills). Each skill tracks which
// agents reference it and its origin (project vs library).

import { useMemo } from 'react'
import { usePullAgents } from '#/features/studio/mcp-queries'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import type { Skill, PullSkill } from '@ship/ui'

const CACHE_KEY = 'ship-skills-library-cache'

export interface LibrarySkill extends Skill {
  /** 'project' = from .ship/, 'library' = from ~/.ship/ */
  origin: 'project' | 'library'
  /** Agent IDs that reference this skill */
  usedBy: string[]
}

export interface UseSkillsLibraryReturn {
  skills: LibrarySkill[]
  isLoading: boolean
  isConnected: boolean
}

function loadCachedSkills(): LibrarySkill[] {
  try {
    const raw = typeof window !== 'undefined'
      ? window.localStorage.getItem(CACHE_KEY)
      : null
    if (!raw) return []
    return JSON.parse(raw) as LibrarySkill[]
  } catch {
    return []
  }
}

function cacheSkills(skills: LibrarySkill[]) {
  try {
    window.localStorage.setItem(CACHE_KEY, JSON.stringify(skills))
  } catch { /* storage full or unavailable */ }
}

/** Convert a PullSkill to the Skill shape used by the IDE. */
function pullSkillToSkill(ps: PullSkill): Skill {
  return {
    id: ps.id,
    name: ps.name,
    description: ps.description ?? null,
    content: ps.content,
    source: (ps.source === 'custom' || ps.source === 'builtin' ||
             ps.source === 'ai-generated' || ps.source === 'community' ||
             ps.source === 'imported')
      ? ps.source
      : 'custom',
  }
}

/**
 * Aggregate and deduplicate skills from all pulled agents.
 * Returns a stable array of LibrarySkill with usage and origin metadata.
 */
export function aggregateSkills(
  agents: { id: string; skills: PullSkill[]; source?: string }[],
): LibrarySkill[] {
  const map = new Map<string, LibrarySkill>()

  for (const agent of agents) {
    const agentOrigin = agent.source === 'library' ? 'library' : 'project'

    for (const ps of agent.skills) {
      const existing = map.get(ps.id)
      if (existing) {
        if (!existing.usedBy.includes(agent.id)) {
          existing.usedBy.push(agent.id)
        }
      } else {
        const skill = pullSkillToSkill(ps)
        map.set(ps.id, {
          ...skill,
          origin: agentOrigin as 'project' | 'library',
          usedBy: [agent.id],
        })
      }
    }
  }

  return Array.from(map.values())
}

export function useSkillsLibrary(): UseSkillsLibraryReturn {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'
  const pullQuery = usePullAgents()

  const pulledSkills = useMemo(() => {
    if (!pullQuery.data?.agents) return []
    const agentInputs = pullQuery.data.agents.map((a) => ({
      id: a.profile.id,
      skills: a.skills,
      source: a.source,
    }))
    return aggregateSkills(agentInputs)
  }, [pullQuery.data])

  // Cache pulled skills for offline use
  useMemo(() => {
    if (pulledSkills.length > 0) cacheSkills(pulledSkills)
  }, [pulledSkills])

  const skills = useMemo(() => {
    if (pulledSkills.length > 0) return pulledSkills
    if (!isConnected) return loadCachedSkills()
    return []
  }, [pulledSkills, isConnected])

  return {
    skills,
    isLoading: pullQuery.isLoading,
    isConnected: isConnected ?? false,
  }
}
