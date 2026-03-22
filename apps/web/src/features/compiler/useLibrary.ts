import { useState, useCallback, useEffect, useRef } from 'react'
import { toast } from 'sonner'
import { useCompiler } from '#/features/compiler/useCompiler'
import { DEFAULT_LIBRARY } from '#/features/compiler/types'
import type { ProjectLibrary } from '#/features/compiler/types'
import type { McpServerConfig, Skill } from '@ship/ui'

const STORAGE_KEY = 'ship-studio-v1'

function loadStored(): { library: ProjectLibrary; modeName: string; selectedProviders: string[] } | null {
  try {
    const raw = typeof window !== 'undefined' ? window.localStorage.getItem(STORAGE_KEY) : null
    if (!raw) return null
    return JSON.parse(raw) as { library: ProjectLibrary; modeName: string; selectedProviders: string[] }
  } catch (err) {
    console.warn('[useLibrary] Failed to parse stored library from localStorage:', err)
    return null
  }
}

export function useLibrary() {
  const stored = useRef(loadStored())
  const [library, setLibrary] = useState<ProjectLibrary>(stored.current?.library ?? DEFAULT_LIBRARY)
  const [modeName, setModeName] = useState(stored.current?.modeName ?? 'untitled-mode')
  const [selectedProviders, setSelectedProviders] = useState<string[]>(
    stored.current?.selectedProviders ?? ['claude', 'gemini', 'codex'],
  )
  const { state, compile } = useCompiler()

  // Persist to localStorage and notify sync listeners
  useEffect(() => {
    try {
      const value = JSON.stringify({ library, modeName, selectedProviders })
      window.localStorage.setItem(STORAGE_KEY, value)
      window.dispatchEvent(new StorageEvent('storage', { key: STORAGE_KEY, newValue: value }))
    } catch (err) {
      console.warn('[useLibrary] Failed to persist library to localStorage:', err)
    }
  }, [library, modeName, selectedProviders])

  // Auto-compile on change (debounced 600ms)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current)
    timerRef.current = setTimeout(() => {
      const effectiveLibrary = {
        ...library,
        modes: [{
          id: 'default',
          name: modeName || 'default',
          description: '',
          target_agents: selectedProviders,
          mcp_servers: [],
          skills: [],
          rules: [],
        }],
        active_agent: 'default',
      }
      void compile(effectiveLibrary)
    }, 600)
    return () => { if (timerRef.current) clearTimeout(timerRef.current) }
  }, [library, selectedProviders, modeName, compile])

  const updateLibrary = useCallback((patch: Partial<ProjectLibrary>) => {
    setLibrary((prev) => ({ ...prev, ...patch }))
  }, [])

  const handleImport = useCallback((imported: ProjectLibrary) => {
    setLibrary((prev) => {
      const existingMcpNames = new Set((prev.mcp_servers ?? []).map((s) => s.name))
      const existingSkillIds = new Set((prev.skills ?? []).map((s) => s.id))
      const existingRuleNames = new Set((prev.rules ?? []).map((r) => r.file_name))
      return {
        ...prev,
        mcp_servers: [
          ...(prev.mcp_servers ?? []),
          ...(imported.mcp_servers ?? []).filter((s) => !existingMcpNames.has(s.name)),
        ],
        skills: [
          ...(prev.skills ?? []),
          ...(imported.skills ?? []).filter((s) => !existingSkillIds.has(s.id)),
        ],
        rules: [
          ...(prev.rules ?? []),
          ...(imported.rules ?? []).filter((r) => !existingRuleNames.has(r.file_name)),
        ],
      }
    })
  }, [])

  const addMcpServer = useCallback((config: McpServerConfig) => {
    setLibrary((prev) => {
      if ((prev.mcp_servers ?? []).some((s) => s.name === config.name)) return prev
      return { ...prev, mcp_servers: [...(prev.mcp_servers ?? []), config] }
    })
  }, [])

  const addSkill = useCallback((skill: Skill) => {
    setLibrary((prev) => {
      if ((prev.skills ?? []).some((s) => s.id === skill.id)) return prev
      return { ...prev, skills: [...(prev.skills ?? []), skill] }
    })
  }, [])

  const toggleProvider = useCallback((id: string) => {
    setSelectedProviders((prev) =>
      prev.includes(id) ? prev.filter((p) => p !== id) : [...prev, id],
    )
  }, [])

  const clearLibrary = useCallback(() => {
    setLibrary(DEFAULT_LIBRARY)
    toast.success('Library cleared')
  }, [])

  return {
    library,
    modeName,
    selectedProviders,
    compileState: state,
    updateLibrary,
    setModeName,
    handleImport,
    addMcpServer,
    addSkill,
    toggleProvider,
    clearLibrary,
  }
}
