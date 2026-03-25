import { useState, useCallback, useEffect, useRef } from 'react'
import { toast } from 'sonner'
import { useCompiler } from '#/features/compiler/useCompiler'
import { DEFAULT_LIBRARY, PROVIDERS } from '#/features/compiler/types'
import type { ProjectLibrary } from '#/features/compiler/types'
import type { McpServerConfig, Skill } from '@ship/ui'
import { idbGet, idbSet, migrateFromLocalStorage } from '#/lib/idb-cache'

const STORAGE_KEY = 'ship-studio-v1'

/** Derive provider IDs from the canonical PROVIDERS list. */
const DEFAULT_PROVIDER_IDS = PROVIDERS.map((p) => p.id)

interface StoredLibrary {
  library: ProjectLibrary
  modeName: string
  selectedProviders: string[]
}

export function useLibrary() {
  const [library, setLibrary] = useState<ProjectLibrary>(DEFAULT_LIBRARY)
  const [modeName, setModeName] = useState('untitled-mode')
  const [selectedProviders, setSelectedProviders] = useState<string[]>(DEFAULT_PROVIDER_IDS)
  const [loaded, setLoaded] = useState(false)
  const { state, compile } = useCompiler()

  // Load from IndexedDB on mount (migrate from localStorage if needed)
  useEffect(() => {
    let cancelled = false
    async function load() {
      try {
        const migrated = await migrateFromLocalStorage<StoredLibrary>(STORAGE_KEY)
        if (migrated && !cancelled) {
          setLibrary(migrated.library ?? DEFAULT_LIBRARY)
          setModeName(migrated.modeName ?? 'untitled-mode')
          setSelectedProviders(migrated.selectedProviders ?? DEFAULT_PROVIDER_IDS)
          setLoaded(true)
          return
        }
        const data = await idbGet<StoredLibrary>(STORAGE_KEY)
        if (data && !cancelled) {
          setLibrary(data.library ?? DEFAULT_LIBRARY)
          setModeName(data.modeName ?? 'untitled-mode')
          setSelectedProviders(data.selectedProviders ?? DEFAULT_PROVIDER_IDS)
        }
      } catch { /* IDB unavailable */ }
      if (!cancelled) setLoaded(true)
    }
    void load()
    return () => { cancelled = true }
  }, [])

  // Persist to IndexedDB on change (skip initial render before load completes)
  useEffect(() => {
    if (!loaded) return
    idbSet(STORAGE_KEY, { library, modeName, selectedProviders } as StoredLibrary).catch(() => {})
  }, [library, modeName, selectedProviders, loaded])

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
