// ── V1 -> V2 storage migration ───────────────────────────────────────────────
// One-time migration from the old dual-storage system (useProfiles + useAgentDetail)
// to the unified ship-agents-v2 format. Idempotent: guarded by a migration flag.

import type { AgentProfile, HookConfig } from './types'
import { DEFAULT_SETTINGS } from './types'
import { DEFAULT_PERMISSIONS } from '@ship/ui'
import type { SkillSource } from '@ship/ui'

// ── Old V1 types (for deserialization only) ──────────────────────────────────

interface V1Profile {
  id: string
  name: string
  persona?: string
  icon?: string
  accentColor?: string
  selectedProviders?: string[]
  skills?: Array<{ id: string; name: string; content: string; source: string }>
  mcpServers?: Array<Record<string, unknown>>
  rules?: string[]
  permissions?: Record<string, unknown>
}

interface V1StoredState {
  profiles: V1Profile[]
  activeId: string | null
}

// ── Constants ────────────────────────────────────────────────────────────────

const V1_PROFILES_KEY = 'ship-profiles-v1'
const V1_AGENT_DETAIL_KEY = 'ship-agent-profiles-v1'
const MIGRATION_FLAG = 'ship-agents-migrated'

// ── Public API ───────────────────────────────────────────────────────────────

/**
 * Returns true if migration has already run. Callers should skip migration
 * when this returns true to avoid redundant work.
 */
export function hasMigrated(): boolean {
  if (typeof window === 'undefined') return true
  return window.localStorage.getItem(MIGRATION_FLAG) === '1'
}

/**
 * Reads old V1 localStorage keys, merges them into AgentProfile[], and returns
 * the result. Returns an empty array when no V1 data exists.
 *
 * Does NOT write to localStorage — the caller (useAgentStore) handles persistence.
 */
export function migrateFromV1(): { agents: AgentProfile[]; activeId: string | null } {
  if (typeof window === 'undefined') return { agents: [], activeId: null }

  const profileAgents = readV1Profiles()
  const detailAgents = readV1AgentDetails()

  // Merge: detail data wins for matching IDs since it has richer fields
  const merged = new Map<string, AgentProfile>()
  for (const a of profileAgents) merged.set(a.id, a)
  for (const a of detailAgents) merged.set(a.id, a)

  const agents = Array.from(merged.values())
  const activeId = readV1ActiveId() ?? agents[0]?.id ?? null
  return { agents, activeId }
}

/**
 * Delete old V1 keys and set the migration flag. Call after successfully
 * saving the migrated data to ship-agents-v2.
 */
export function finalizeMigration(): void {
  if (typeof window === 'undefined') return
  try {
    window.localStorage.removeItem(V1_PROFILES_KEY)
    window.localStorage.removeItem(V1_AGENT_DETAIL_KEY)
    window.localStorage.setItem(MIGRATION_FLAG, '1')
  } catch { /* storage unavailable */ }
}

// ── Internals ────────────────────────────────────────────────────────────────

function readV1Profiles(): AgentProfile[] {
  try {
    const raw = window.localStorage.getItem(V1_PROFILES_KEY)
    if (!raw) return []
    const stored = JSON.parse(raw) as V1StoredState
    if (!Array.isArray(stored.profiles)) return []
    return stored.profiles.map(convertV1Profile)
  } catch {
    return []
  }
}

function readV1ActiveId(): string | null {
  try {
    const raw = window.localStorage.getItem(V1_PROFILES_KEY)
    if (!raw) return null
    const stored = JSON.parse(raw) as V1StoredState
    return stored.activeId ?? null
  } catch {
    return null
  }
}

function readV1AgentDetails(): AgentProfile[] {
  try {
    const raw = window.localStorage.getItem(V1_AGENT_DETAIL_KEY)
    if (!raw) return []
    const stored = JSON.parse(raw) as Record<string, AgentProfile>
    if (typeof stored !== 'object' || stored === null) return []
    return Object.values(stored).filter(isValidAgentProfile)
  } catch {
    return []
  }
}

function convertV1Profile(p: V1Profile): AgentProfile {
  return {
    id: p.id,
    name: p.name,
    description: p.persona ?? '',
    providers: p.selectedProviders ?? ['claude'],
    version: '0.1.0',
    skills: Array.isArray(p.skills) ? p.skills.map((s) => ({
      id: s.id,
      name: s.name,
      content: s.content ?? '',
      source: (s.source ?? 'custom') as SkillSource,
    })) : [],
    mcpServers: Array.isArray(p.mcpServers) ? p.mcpServers as AgentProfile['mcpServers'] : [],
    subagents: [],
    permissions: isPermissionsLike(p.permissions)
      ? p.permissions as AgentProfile['permissions']
      : { ...DEFAULT_PERMISSIONS },
    permissionPreset: 'ship-guarded',
    settings: { ...DEFAULT_SETTINGS },
    hooks: [] as HookConfig[],
    rules: Array.isArray(p.rules)
      ? p.rules.map((r, i) => ({ file_name: `rule-${i}.md`, content: r }))
      : [],
    mcpToolStates: {},
  }
}

function isValidAgentProfile(v: unknown): v is AgentProfile {
  if (typeof v !== 'object' || v === null) return false
  const obj = v as Record<string, unknown>
  return typeof obj.id === 'string' && typeof obj.name === 'string'
}

function isPermissionsLike(v: unknown): boolean {
  if (typeof v !== 'object' || v === null) return false
  const obj = v as Record<string, unknown>
  return 'tools' in obj && 'filesystem' in obj
}
