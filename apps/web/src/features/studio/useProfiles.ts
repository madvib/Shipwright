import { useState, useCallback, useEffect, useRef } from 'react'
import { useCompiler } from '#/features/compiler/useCompiler'
import { DEFAULT_PERMISSIONS } from '#/features/compiler/types'
import type { McpServerConfig, Skill, Permissions, ProjectLibrary } from '#/features/compiler/types'

// Icon field stores a TechStack key (e.g. 'react', 'typescript').
// Rendered as a brand-color lettermark tile. Swap for real SVG once simple-icons is installed.

export interface Profile {
  id: string
  name: string
  persona: string
  icon: string         // TechStack key, e.g. 'react'
  accentColor: string  // hex — auto-derived from tech stack brand color
  selectedProviders: string[]
  skills: Skill[]
  mcpServers: McpServerConfig[]
  rules: string[]      // bullet lines → compiled to RULES.md
  permissions: Permissions
}

export function makeProfile(name = 'New Profile'): Profile {
  return {
    id: `profile-${Date.now()}`,
    name,
    persona: '',
    icon: 'custom',
    accentColor: '#f59e0b',
    selectedProviders: ['claude'],
    skills: [],
    mcpServers: [],
    rules: [],
    permissions: DEFAULT_PERMISSIONS,
  }
}

/** Build a ProjectLibrary from a Profile for the WASM compiler */
export function profileToLibrary(profile: Profile): ProjectLibrary {
  const rules = profile.rules.length > 0
    ? [{ file_name: 'RULES.md', content: profile.rules.map((r) => `- ${r}`).join('\n') }]
    : []
  return {
    modes: [{
      id: profile.id,
      name: profile.name,
      description: profile.persona || null,
      skills: profile.skills.map((s) => s.id),
      mcp_servers: profile.mcpServers.map((s) => s.name),
      rules: rules.map((r) => r.file_name),
      active_tools: profile.selectedProviders,
    }],
    active_agent: profile.name,
    mcp_servers: profile.mcpServers,
    skills: profile.skills,
    rules,
    permissions: profile.permissions,
    agent_profiles: [],
    claude_team_agents: [],
    env: {},
    available_models: [],
  }
}

const STORAGE_KEY = 'ship-profiles-v1'

interface StoredState {
  profiles: Profile[]
  activeId: string | null
}

function loadStored(): StoredState {
  try {
    const raw = typeof window !== 'undefined' ? window.localStorage.getItem(STORAGE_KEY) : null
    if (raw) return JSON.parse(raw) as StoredState
  } catch { /* ignore */ }
  const initial = makeProfile('web-lane')
  return { profiles: [initial], activeId: initial.id }
}

export function useProfiles() {
  const stored = useRef(loadStored())
  const [profiles, setProfiles] = useState<Profile[]>(stored.current.profiles)
  const [activeId, setActiveId] = useState<string | null>(stored.current.activeId)
  const { state: compileState, compile } = useCompiler()

  const active = profiles.find((p) => p.id === activeId) ?? profiles[0] ?? null

  // Persist
  useEffect(() => {
    try {
      window.localStorage.setItem(STORAGE_KEY, JSON.stringify({ profiles, activeId }))
    } catch { /* ignore */ }
  }, [profiles, activeId])

  // Auto-compile active profile
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  useEffect(() => {
    if (!active) return
    if (timerRef.current) clearTimeout(timerRef.current)
    timerRef.current = setTimeout(() => void compile(profileToLibrary(active)), 600)
    return () => { if (timerRef.current) clearTimeout(timerRef.current) }
  }, [active, compile])

  const addProfile = useCallback(() => {
    const p = makeProfile()
    setProfiles((prev) => [...prev, p])
    setActiveId(p.id)
    return p.id
  }, [])

  const updateProfile = useCallback((id: string, patch: Partial<Profile>) => {
    setProfiles((prev) => prev.map((p) => (p.id === id ? { ...p, ...patch } : p)))
  }, [])

  const removeProfile = useCallback((id: string) => {
    setProfiles((prev) => {
      const next = prev.filter((p) => p.id !== id)
      if (activeId === id) setActiveId(next[0]?.id ?? null)
      return next
    })
  }, [activeId])

  return {
    profiles,
    activeId,
    active,
    compileState,
    setActiveId,
    addProfile,
    updateProfile,
    removeProfile,
  }
}
