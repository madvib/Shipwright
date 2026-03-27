// Skills library: aggregates skills from MCP pull data into a deduplicated view.
// Skills come from pulled agents (PullAgent.skills). Each skill tracks which
// agents reference it and its origin (project vs library).

import { useMemo, useState, useEffect } from 'react'
import { usePullAgents, useProjectSkills } from '#/features/studio/mcp-queries'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { idbGet, idbSet, migrateFromLocalStorage } from '#/lib/idb-cache'
import type { Skill, PullSkill, JsonValue } from '@ship/ui'

const CACHE_KEY = 'ship-skills-library-cache-v2'

export interface LibrarySkill extends Skill {
  /** 'project' = from .ship/, 'library' = from ~/.ship/ */
  origin: 'project' | 'library'
  /** Agent IDs that reference this skill */
  usedBy: string[]
  /** Canonical storage key from stable-id frontmatter */
  stableId: string | null
  /** Tags from frontmatter */
  tags: string[]
  /** Authors from frontmatter */
  authors: string[]
  /** Raw vars.json schema, null if no vars */
  varsSchema: JsonValue | null
  /** All files in the skill directory */
  files: string[]
  /** Reference doc content keyed by relative path */
  referenceDocs: Record<string, string>
  /** Raw evals.json content, null if no evals */
  evals: JsonValue | null
}

export interface UseSkillsLibraryReturn {
  skills: LibrarySkill[]
  isLoading: boolean
  isConnected: boolean
}

/** Convert a PullSkill to a LibrarySkill. */
function pullToLibrarySkill(
  ps: PullSkill,
  origin: 'project' | 'library',
  usedBy: string[],
): LibrarySkill {
  const source = (ps.source === 'custom' || ps.source === 'builtin' ||
           ps.source === 'ai-generated' || ps.source === 'community' ||
           ps.source === 'imported')
    ? ps.source
    : 'custom' as const
  return {
    id: ps.id,
    name: ps.name,
    description: ps.description ?? null,
    content: ps.content,
    source,
    vars: {},
    origin,
    usedBy,
    stableId: ps.stable_id ?? null,
    tags: ps.tags ?? [],
    authors: ps.authors ?? [],
    varsSchema: ps.vars_schema ?? null,
    files: ps.files ?? [],
    referenceDocs: (ps.reference_docs ?? {}) as Record<string, string>,
    evals: ps.evals ?? null,
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
        map.set(ps.id, pullToLibrarySkill(
          ps,
          agentOrigin as 'project' | 'library',
          [agent.id],
        ))
      }
    }
  }

  return Array.from(map.values())
}

/** Merge project skills into an agent-derived skill list. Deduplicates by ID. */
export function mergeProjectSkills(
  agentSkills: LibrarySkill[],
  projectPullSkills: PullSkill[],
): LibrarySkill[] {
  const map = new Map<string, LibrarySkill>()
  for (const s of agentSkills) map.set(s.id, s)
  for (const ps of projectPullSkills) {
    if (!map.has(ps.id)) {
      map.set(ps.id, pullToLibrarySkill(ps, 'project', []))
    }
  }
  return Array.from(map.values())
}

export function useSkillsLibrary(): UseSkillsLibraryReturn {
  const mcp = useLocalMcpContext()
  const isConnected = mcp?.status === 'connected'
  const pullQuery = usePullAgents()
  const projectQuery = useProjectSkills()
  const [cachedSkills, setCachedSkills] = useState<LibrarySkill[]>([])

  // Load cache from IndexedDB on mount (migrate from localStorage if needed)
  useEffect(() => {
    let cancelled = false
    async function load() {
      try {
        const migrated = await migrateFromLocalStorage<LibrarySkill[]>(CACHE_KEY)
        if (migrated && !cancelled) { setCachedSkills(migrated); return }
        const data = await idbGet<LibrarySkill[]>(CACHE_KEY)
        if (data && !cancelled) setCachedSkills(data)
      } catch { /* IDB unavailable */ }
    }
    void load()
    return () => { cancelled = true }
  }, [])

  const pulledSkills = useMemo(() => {
    if (!pullQuery.data?.agents) return []
    const agentInputs = pullQuery.data.agents.map((a) => ({
      id: a.profile.id,
      skills: a.skills,
      source: a.source,
    }))
    return aggregateSkills(agentInputs)
  }, [pullQuery.data])

  // Cache pulled skills to IndexedDB
  useEffect(() => {
    if (pulledSkills.length > 0) {
      setCachedSkills(pulledSkills)
      idbSet(CACHE_KEY, pulledSkills).catch(() => {})
    }
  }, [pulledSkills])

  // Merge project skills (from list_project_skills) into the agent-derived list
  const skills = useMemo(() => {
    const base = pulledSkills.length > 0 ? pulledSkills : (!isConnected ? cachedSkills : [])
    const projectSkills = projectQuery.data ?? []
    if (projectSkills.length === 0) return base
    return mergeProjectSkills(base, projectSkills)
  }, [pulledSkills, isConnected, cachedSkills, projectQuery.data])

  return {
    skills,
    isLoading: pullQuery.isLoading,
    isConnected: isConnected ?? false,
  }
}
