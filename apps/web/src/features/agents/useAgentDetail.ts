import { useState, useCallback, useEffect } from 'react'
import type { Skill } from '@ship/ui'
import type {
  AgentProfile,
  ToolPermission,
  AgentSettings,
} from './types'
import { DEFAULT_SETTINGS } from './types'

const STORAGE_KEY = 'ship-agent-profiles-v1'

function loadProfiles(): Record<string, AgentProfile> {
  try {
    const raw = typeof window !== 'undefined'
      ? window.localStorage.getItem(STORAGE_KEY)
      : null
    if (!raw) return {}
    return JSON.parse(raw) as Record<string, AgentProfile>
  } catch {
    return {}
  }
}

function saveProfiles(profiles: Record<string, AgentProfile>) {
  try {
    window.localStorage.setItem(STORAGE_KEY, JSON.stringify(profiles))
  } catch { /* ignore */ }
}

export function useAgentDetail(agentId: string) {
  const [profile, setProfile] = useState<AgentProfile>(() => {
    const stored = loadProfiles()
    return stored[agentId] ?? {
      id: agentId,
      name: agentId,
      description: '',
      providers: [],
      version: '0.1.0',
      skills: [],
      mcpServers: [],
      subagents: [],
      permissions: {
        tools: { allow: [], deny: [] },
        filesystem: { allow: [], deny: [] },
        commands: { allow: [], deny: [] },
        network: { policy: 'none', allow_hosts: [] },
        agent: { require_confirmation: [] },
      },
      permissionPreset: 'ship-guarded',
      settings: { ...DEFAULT_SETTINGS },
      hooks: [],
      rules: [],
      mcpToolStates: {},
    }
  })

  // Persist on change
  useEffect(() => {
    const profiles = loadProfiles()
    profiles[profile.id] = profile
    saveProfiles(profiles)
  }, [profile])

  const updateProfile = useCallback((patch: Partial<AgentProfile>) => {
    setProfile((prev) => ({ ...prev, ...patch }))
  }, [])

  const removeSkill = useCallback((skillId: string) => {
    setProfile((prev) => ({
      ...prev,
      skills: prev.skills.filter((s) => s.id !== skillId),
    }))
  }, [])

  const addSkill = useCallback((skill: Skill) => {
    setProfile((prev) => {
      if (prev.skills.some((s) => s.id === skill.id)) return prev
      return { ...prev, skills: [...prev.skills, skill] }
    })
  }, [])

  const removeServer = useCallback((name: string) => {
    setProfile((prev) => ({
      ...prev,
      mcpServers: prev.mcpServers.filter((s) => s.name !== name),
    }))
  }, [])

  const removeSubagent = useCallback((id: string) => {
    setProfile((prev) => ({
      ...prev,
      subagents: prev.subagents.filter((s) => s.id !== id),
    }))
  }, [])

  const setPermissionPreset = useCallback((preset: string) => {
    setProfile((prev) => ({ ...prev, permissionPreset: preset }))
  }, [])

  const updateSettings = useCallback((patch: Partial<AgentSettings>) => {
    setProfile((prev) => ({
      ...prev,
      settings: { ...prev.settings, ...patch },
    }))
  }, [])

  const setToolPermission = useCallback(
    (serverName: string, toolName: string, permission: ToolPermission) => {
      setProfile((prev) => {
        const serverTools = prev.mcpToolStates[serverName] ?? {}
        return {
          ...prev,
          mcpToolStates: {
            ...prev.mcpToolStates,
            [serverName]: { ...serverTools, [toolName]: permission },
          },
        }
      })
    },
    [],
  )

  const setGroupPermission = useCallback(
    (serverName: string, toolNames: string[], permission: ToolPermission) => {
      setProfile((prev) => {
        const serverTools = { ...(prev.mcpToolStates[serverName] ?? {}) }
        for (const name of toolNames) {
          serverTools[name] = permission
        }
        return {
          ...prev,
          mcpToolStates: { ...prev.mcpToolStates, [serverName]: serverTools },
        }
      })
    },
    [],
  )

  return {
    profile,
    updateProfile,
    removeSkill,
    addSkill,
    removeServer,
    removeSubagent,
    setPermissionPreset,
    updateSettings,
    setToolPermission,
    setGroupPermission,
  }
}
