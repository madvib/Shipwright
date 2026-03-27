import { useState, useCallback, useEffect, useRef } from 'react'
import { useSkillsLibrary } from './useSkillsLibrary'
import type { LibrarySkill } from './useSkillsLibrary'
import type { Skill } from '@ship/ui'
import { newSkillTemplate } from './skill-frontmatter'

const IDE_STATE_KEY = 'ship-skills-ide-v1'
export const SKILL_MD = 'SKILL.md'

/** Composite tab ID: `skillId::filePath` */
export function makeTabId(skillId: string, filePath: string): string {
  return `${skillId}::${filePath}`
}

export function parseTabId(tabId: string): { skillId: string; filePath: string } {
  const idx = tabId.indexOf('::')
  // Backward compat: bare skill IDs become skillId::SKILL.md
  if (idx === -1) return { skillId: tabId, filePath: SKILL_MD }
  return { skillId: tabId.slice(0, idx), filePath: tabId.slice(idx + 2) }
}

interface IDEState {
  openTabIds: string[]
  activeTabId: string | null
  expandedFolders: Set<string>
  unsavedIds: Set<string>
  searchQuery: string
  previewTab: 'metadata' | 'output' | 'used-by'
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
    // Migrate old bare-skillId tab IDs to composite format
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

function persistState(state: PersistedIDEState) {
  try {
    window.localStorage.setItem(IDE_STATE_KEY, JSON.stringify(state))
  } catch { /* ignore */ }
}

export function useSkillsIDE() {
  const { skills: librarySkills, isLoading, isConnected } = useSkillsLibrary()
  const skills: Skill[] = librarySkills
  const persisted = useRef(loadPersistedState())

  const [openTabIds, setOpenTabIds] = useState<string[]>(persisted.current.openTabIds ?? [])
  const [activeTabId, setActiveTabId] = useState<string | null>(persisted.current.activeTabId ?? null)
  const [expandedFolders, setExpandedFolders] = useState<Set<string>>(
    new Set(persisted.current.expandedFolders ?? []),
  )
  const [unsavedIds, setUnsavedIds] = useState<Set<string>>(new Set())
  const [searchQuery, setSearchQuery] = useState('')
  const [previewTab, setPreviewTab] = useState<'metadata' | 'output' | 'used-by'>('metadata')
  const [previewOpen, setPreviewOpen] = useState(true)

  // Local content buffers for unsaved edits (drafts)
  const [contentBuffers, setContentBuffers] = useState<Record<string, string>>({})

  // Persist layout state
  useEffect(() => {
    persistState({
      openTabIds,
      activeTabId,
      expandedFolders: Array.from(expandedFolders),
    })
  }, [openTabIds, activeTabId, expandedFolders])

  // If active tab's skill doesn't exist, clear it
  useEffect(() => {
    if (!activeTabId) return
    const { skillId } = parseTabId(activeTabId)
    if (!skills.some((s) => s.id === skillId)) {
      // Find first open tab whose skill still exists
      const fallback = openTabIds.find((tid) => {
        const { skillId: sid } = parseTabId(tid)
        return skills.some((s) => s.id === sid)
      })
      setActiveTabId(fallback ?? null)
    }
  }, [skills, activeTabId, openTabIds])

  // Derive active skill and content from composite tab ID
  const activeTab = activeTabId ? parseTabId(activeTabId) : null
  const activeSkill = activeTab ? (skills.find((s) => s.id === activeTab.skillId) ?? null) : null

  const getFileContent = useCallback((skillId: string, filePath: string): string => {
    const skill = skills.find((s) => s.id === skillId)
    if (filePath === SKILL_MD) return skill?.content ?? ''
    return ''
  }, [skills])

  const activeContent = activeTabId
    ? (contentBuffers[activeTabId] ?? (activeTab ? getFileContent(activeTab.skillId, activeTab.filePath) : ''))
    : ''

  const openFile = useCallback((skillId: string, filePath: string) => {
    const tabId = makeTabId(skillId, filePath)
    setOpenTabIds((prev) => (prev.includes(tabId) ? prev : [...prev, tabId]))
    setActiveTabId(tabId)
  }, [])

  const openSkill = useCallback((id: string) => {
    openFile(id, SKILL_MD)
    setExpandedFolders((prev) => {
      if (prev.has(id)) return prev
      return new Set(prev).add(id)
    })
  }, [openFile])

  const closeTab = useCallback((tabId: string) => {
    setOpenTabIds((prev) => {
      const next = prev.filter((t) => t !== tabId)
      if (activeTabId === tabId) {
        const idx = prev.indexOf(tabId)
        const newActive = next[Math.min(idx, next.length - 1)] ?? null
        setActiveTabId(newActive)
      }
      return next
    })
    setUnsavedIds((prev) => { const n = new Set(prev); n.delete(tabId); return n })
    setContentBuffers((prev) => { const n = { ...prev }; delete n[tabId]; return n })
  }, [activeTabId])

  const updateContent = useCallback((tabId: string, content: string) => {
    setContentBuffers((prev) => ({ ...prev, [tabId]: content }))
    setUnsavedIds((prev) => new Set(prev).add(tabId))
  }, [])

  // Save stages content as a draft (marks unsaved cleared).
  // Actual persistence to CLI happens via push.
  const saveSkill = useCallback((id: string) => {
    const content = contentBuffers[id]
    if (content === undefined) return
    // Draft is stored in contentBuffers; clearing unsaved marker means
    // the edit is acknowledged but not yet pushed to CLI.
    setUnsavedIds((prev) => { const n = new Set(prev); n.delete(id); return n })
  }, [contentBuffers])

  const saveAll = useCallback(() => {
    if (unsavedIds.size === 0) return
    setUnsavedIds(new Set())
  }, [unsavedIds])

  const toggleFolder = useCallback((id: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  const createSkill = useCallback((idOrUndefined?: string) => {
    const ts = Date.now()
    const id = idOrUndefined ?? `new-skill-${ts}`
    const name = id.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
    const content = newSkillTemplate(name, id)
    // New skill creation is a local draft only — not yet pushed
    setContentBuffers((prev) => ({ ...prev, [id]: content }))
    setExpandedFolders((prev) => new Set(prev).add(id))
    openSkill(id)
    setUnsavedIds((prev) => new Set(prev).add(id))
  }, [openSkill])

  const deleteSkill = useCallback((id: string) => {
    closeTab(id)
  }, [closeTab])

  const filteredSkills = searchQuery
    ? skills.filter((s) => {
        const q = searchQuery.toLowerCase()
        return s.name.toLowerCase().includes(q) || s.id.toLowerCase().includes(q)
      })
    : skills

  /** Get the LibrarySkill metadata (origin, usedBy) for a skill ID. */
  const getLibrarySkill = useCallback((id: string): LibrarySkill | undefined => {
    return librarySkills.find((s) => s.id === id)
  }, [librarySkills])

  const state: IDEState = {
    openTabIds,
    activeTabId,
    expandedFolders,
    unsavedIds,
    searchQuery,
    previewTab,
    previewOpen,
  }

  return {
    skills,
    filteredSkills,
    state,
    activeSkill,
    activeContent,
    isLoading,
    isConnected,
    getLibrarySkill,
    openSkill,
    openFile,
    closeTab,
    setActiveTabId,
    updateContent,
    saveSkill,
    saveAll,
    toggleFolder,
    setSearchQuery,
    createSkill,
    deleteSkill,
    setPreviewTab,
    setPreviewOpen,
  }
}
