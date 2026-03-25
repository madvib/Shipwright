import { useState, useCallback, useEffect, useRef } from 'react'
import { useSkillsLibrary } from './useSkillsLibrary'
import type { LibrarySkill } from './useSkillsLibrary'
import type { Skill } from '@ship/ui'
import { newSkillTemplate } from './skill-frontmatter'

const IDE_STATE_KEY = 'ship-skills-ide-v1'

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
    if (raw) return JSON.parse(raw)
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

  // If active tab doesn't exist in skills, clear it
  useEffect(() => {
    if (activeTabId && !skills.some((s) => s.id === activeTabId)) {
      setActiveTabId(openTabIds.find((id) => skills.some((s) => s.id === id)) ?? null)
    }
  }, [skills, activeTabId, openTabIds])

  const activeSkill = skills.find((s) => s.id === activeTabId) ?? null
  const activeContent = activeTabId
    ? (contentBuffers[activeTabId] ?? activeSkill?.content ?? '')
    : ''

  const openSkill = useCallback((id: string) => {
    setOpenTabIds((prev) => (prev.includes(id) ? prev : [...prev, id]))
    setActiveTabId(id)
  }, [])

  const closeTab = useCallback((id: string) => {
    setOpenTabIds((prev) => {
      const next = prev.filter((t) => t !== id)
      if (activeTabId === id) {
        const idx = prev.indexOf(id)
        const newActive = next[Math.min(idx, next.length - 1)] ?? null
        setActiveTabId(newActive)
      }
      return next
    })
    setUnsavedIds((prev) => { const n = new Set(prev); n.delete(id); return n })
    setContentBuffers((prev) => { const n = { ...prev }; delete n[id]; return n })
  }, [activeTabId])

  const updateContent = useCallback((id: string, content: string) => {
    setContentBuffers((prev) => ({ ...prev, [id]: content }))
    setUnsavedIds((prev) => new Set(prev).add(id))
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
