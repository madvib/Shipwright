import { useState, useCallback, useEffect, useRef, useMemo } from 'react'
import { useSkillsLibrary } from './useSkillsLibrary'
import type { LibrarySkill } from './useSkillsLibrary'
import { newSkillTemplate } from './skill-frontmatter'
import { useSaveSkillFile } from '#/features/studio/mcp-queries'
import { idbGet, idbSet } from '#/lib/idb-cache'
import {
  SKILL_MD,
  makeTabId,
  parseTabId,
  deriveUnsavedIds,
  deriveLocalFiles,
  type SkillDraft,
} from './skill-drafts'

// Re-export tab-ID utilities so existing component imports keep working.
export { SKILL_MD, makeTabId, parseTabId } from './skill-drafts'

const IDE_STATE_KEY = 'ship-skills-ide-v1'
const DRAFTS_IDB_KEY = 'ship-skills-ide-drafts'
const DRAFTS_SAVE_DELAY = 800

interface IDEState {
  openTabIds: string[]
  activeTabId: string | null
  expandedFolders: Set<string>
  unsavedIds: Set<string>
  searchQuery: string
  previewTab: string
  previewOpen: boolean
}

interface PersistedIDEState {
  openTabIds: string[]
  activeTabId: string | null
  expandedFolders: string[]
}

function loadPersistedState(): Partial<PersistedIDEState> {
  try {
    const raw = window.localStorage.getItem(IDE_STATE_KEY)
    if (!raw) return {}
    const parsed = JSON.parse(raw)
    if (parsed.openTabIds) {
      parsed.openTabIds = parsed.openTabIds.map((id: string) =>
        id.includes('::') ? id : makeTabId(id, SKILL_MD),
      )
    }
    if (parsed.activeTabId && !parsed.activeTabId.includes('::')) {
      parsed.activeTabId = makeTabId(parsed.activeTabId, SKILL_MD)
    }
    return parsed
  } catch { /* ignore */ }
  return {}
}

/** Resolve file content from pull data for a given skill + file path. */
function resolveOriginal(skills: LibrarySkill[], skillId: string, filePath: string): string {
  const skill = skills.find((s) => s.id === skillId)
  if (!skill) return ''
  if (filePath === SKILL_MD) return skill.content ?? ''
  if (skill.referenceDocs?.[filePath]) return skill.referenceDocs[filePath]
  if (filePath === 'assets/vars.json' && skill.varsSchema) {
    return JSON.stringify(skill.varsSchema, null, 2)
  }
  if (filePath === 'evals/evals.json' && skill.evals) {
    return JSON.stringify(skill.evals, null, 2)
  }
  return ''
}

export function useSkillsIDE() {
  const { skills: librarySkills, isLoading, isConnected } = useSkillsLibrary()
  const persisted = useRef(loadPersistedState())
  const saveMutation = useSaveSkillFile()

  const [openTabIds, setOpenTabIds] = useState<string[]>(persisted.current.openTabIds ?? [])
  const [activeTabId, setActiveTabId] = useState<string | null>(persisted.current.activeTabId ?? null)
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(
    new Set(persisted.current.expandedFolders ?? []),
  )
  const [searchQuery, setSearchQuery] = useState('')
  const [previewTab, setPreviewTab] = useState('vars')
  const [previewOpen, setPreviewOpen] = useState(true)

  // Single drafts map replaces contentBuffers + unsavedIds + localFiles
  const [drafts, setDrafts] = useState<Record<string, SkillDraft>>({})
  const draftsLoadedRef = useRef(false)
  const draftsSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Load drafts from IndexedDB on mount
  useEffect(() => {
    let cancelled = false
    async function load() {
      try {
        const stored = await idbGet<Record<string, SkillDraft>>(DRAFTS_IDB_KEY)
        if (stored && !cancelled) setDrafts(stored)
      } catch { /* IDB unavailable */ }
      draftsLoadedRef.current = true
    }
    void load()
    return () => { cancelled = true }
  }, [])

  // Debounced persist drafts to IndexedDB
  useEffect(() => {
    if (!draftsLoadedRef.current) return
    if (draftsSaveTimerRef.current) clearTimeout(draftsSaveTimerRef.current)
    draftsSaveTimerRef.current = setTimeout(() => {
      idbSet(DRAFTS_IDB_KEY, drafts).catch(() => {})
    }, DRAFTS_SAVE_DELAY)
    return () => { if (draftsSaveTimerRef.current) clearTimeout(draftsSaveTimerRef.current) }
  }, [drafts])

  const unsavedIds = useMemo(() => deriveUnsavedIds(drafts), [drafts])
  const localFiles = useMemo(() => deriveLocalFiles(drafts), [drafts])

  const skills: LibrarySkill[] = useMemo(() => {
    if (Object.keys(localFiles).length === 0) return librarySkills
    return librarySkills.map((s) => {
      const added = localFiles[s.id]
      if (!added?.length) return s
      return { ...s, files: Array.from(new Set([...s.files, ...added])).sort() }
    })
  }, [librarySkills, localFiles])

  // Persist layout state to localStorage
  useEffect(() => {
    try {
      window.localStorage.setItem(IDE_STATE_KEY, JSON.stringify({
        openTabIds, activeTabId, expandedFolders: Array.from(expandedFolders),
      }))
    } catch { /* ignore */ }
  }, [openTabIds, activeTabId, expandedFolders])

  // Clear active tab if its skill was removed
  useEffect(() => {
    if (!activeTabId) return
    const { skillId } = parseTabId(activeTabId)
    if (!skills.some((s) => s.id === skillId)) {
      const fallback = openTabIds.find((tid) => skills.some((s) => s.id === parseTabId(tid).skillId))
      setActiveTabId(fallback ?? null)
    }
  }, [skills, activeTabId, openTabIds])

  const activeTab = activeTabId ? parseTabId(activeTabId) : null
  const activeSkill = activeTab ? (skills.find((s) => s.id === activeTab.skillId) ?? null) : null

  const activeContent = activeTabId
    ? (drafts[activeTabId]?.content ?? (activeTab ? resolveOriginal(librarySkills, activeTab.skillId, activeTab.filePath) : ''))
    : ''

  const openFile = useCallback((skillId: string, filePath: string) => {
    const tabId = makeTabId(skillId, filePath)
    setOpenTabIds((prev) => (prev.includes(tabId) ? prev : [...prev, tabId]))
    setActiveTabId(tabId)
  }, [])

  const addFile = useCallback((skillId: string, filePath: string, content: string) => {
    const tabId = makeTabId(skillId, filePath)
    setDrafts((prev) => ({ ...prev, [tabId]: { content, originalContent: content, isNew: true } }))
    setExpandedFolders((prev) => new Set(prev).add(skillId))
    openFile(skillId, filePath)
  }, [openFile])

  const openSkill = useCallback((id: string) => {
    openFile(id, SKILL_MD)
    setExpandedFolders((prev) => prev.has(id) ? prev : new Set(prev).add(id))
  }, [openFile])

  const closeTab = useCallback((tabId: string) => {
    setOpenTabIds((prev) => {
      const next = prev.filter((t) => t !== tabId)
      if (activeTabId === tabId) {
        const idx = prev.indexOf(tabId)
        setActiveTabId(next[Math.min(idx, next.length - 1)] ?? null)
      }
      return next
    })
    setDrafts((prev) => {
      if (!(tabId in prev)) return prev
      const next = { ...prev }
      delete next[tabId]
      return next
    })
  }, [activeTabId])

  const updateContent = useCallback((tabId: string, content: string) => {
    setDrafts((prev) => {
      const existing = prev[tabId]
      if (existing) return { ...prev, [tabId]: { ...existing, content } }
      const { skillId, filePath } = parseTabId(tabId)
      const original = resolveOriginal(librarySkills, skillId, filePath)
      return { ...prev, [tabId]: { content, originalContent: original, isNew: false } }
    })
  }, [librarySkills])

  /** Mark a draft as saved after successful MCP write. */
  const markSaved = useCallback((tabId: string) => {
    setDrafts((prev) => {
      const d = prev[tabId]
      if (!d) return prev
      return { ...prev, [tabId]: { ...d, originalContent: d.content, isNew: false } }
    })
  }, [])

  const saveSkill = useCallback((tabId: string) => {
    const draft = drafts[tabId]
    if (!draft) return
    const { skillId, filePath } = parseTabId(tabId)
    saveMutation.mutate(
      { skillId, filePath, content: draft.content },
      { onSuccess: () => markSaved(tabId) },
    )
  }, [drafts, saveMutation, markSaved])

  const saveAll = useCallback(() => {
    for (const tabId of unsavedIds) {
      const draft = drafts[tabId]
      if (!draft) continue
      const { skillId, filePath } = parseTabId(tabId)
      saveMutation.mutate(
        { skillId, filePath, content: draft.content },
        { onSuccess: () => markSaved(tabId) },
      )
    }
  }, [unsavedIds, drafts, saveMutation, markSaved])

  const toggleFolder = useCallback((id: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id); else next.add(id)
      return next
    })
  }, [])

  const collapseAll = useCallback(() => {
    if (activeTabId) {
      setExpandedFolders(new Set([parseTabId(activeTabId).skillId]))
    } else {
      setExpandedFolders(new Set())
    }
  }, [activeTabId])

  const createSkill = useCallback((idOrUndefined?: string) => {
    const id = idOrUndefined ?? `new-skill-${Date.now()}`
    const name = id.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
    const content = newSkillTemplate(name, id)
    const tabId = makeTabId(id, SKILL_MD)
    setDrafts((prev) => ({ ...prev, [tabId]: { content, originalContent: content, isNew: true } }))
    setExpandedFolders((prev) => new Set(prev).add(id))
    openSkill(id)
  }, [openSkill])

  const deleteSkill = useCallback((id: string) => { closeTab(id) }, [closeTab])

  const filteredSkills = searchQuery
    ? skills.filter((s) => {
        const q = searchQuery.toLowerCase()
        return s.name.toLowerCase().includes(q) || s.id.toLowerCase().includes(q)
      })
    : skills

  const getLibrarySkill = useCallback((id: string): LibrarySkill | undefined => {
    return skills.find((s) => s.id === id)
  }, [skills])

  const state: IDEState = {
    openTabIds, activeTabId, expandedFolders, unsavedIds,
    searchQuery, previewTab, previewOpen,
  }

  return {
    skills, filteredSkills, state, activeSkill, activeContent,
    isLoading, isConnected, isSaving: saveMutation.isPending,
    getLibrarySkill, openSkill, openFile, addFile, closeTab,
    setActiveTabId, updateContent, saveSkill, saveAll,
    toggleFolder, collapseAll, setSearchQuery, createSkill,
    deleteSkill, setPreviewTab, setPreviewOpen,
  }
}
