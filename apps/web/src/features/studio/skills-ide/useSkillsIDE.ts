import { useState, useCallback, useEffect, useRef } from 'react'
import { useLibrary } from '#/features/compiler/useLibrary'
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
  const { library, updateLibrary } = useLibrary()
  const skills = library.skills ?? []
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

  // Local content buffers for unsaved edits
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

  const saveSkill = useCallback((id: string) => {
    const content = contentBuffers[id]
    if (content === undefined) return
    const updated = skills.map((s) => (s.id === id ? { ...s, content } : s))
    updateLibrary({ skills: updated })
    setUnsavedIds((prev) => { const n = new Set(prev); n.delete(id); return n })
  }, [contentBuffers, skills, updateLibrary])

  const saveAll = useCallback(() => {
    if (unsavedIds.size === 0) return
    const updated = skills.map((s) => {
      const buf = contentBuffers[s.id]
      return buf !== undefined ? { ...s, content: buf } : s
    })
    updateLibrary({ skills: updated })
    setUnsavedIds(new Set())
  }, [contentBuffers, skills, updateLibrary, unsavedIds])

  const toggleFolder = useCallback((id: string) => {
    setExpandedFolders((prev) => {
      const next = new Set(prev)
      if (next.has(id)) next.delete(id)
      else next.add(id)
      return next
    })
  }, [])

  const createSkill = useCallback(() => {
    const ts = Date.now()
    const id = `new-skill-${ts}`
    const name = 'New Skill'
    const content = newSkillTemplate(name, id)
    const skill: Skill = {
      id,
      name,
      description: null,
      content,
      source: 'custom',
    }
    updateLibrary({ skills: [...skills, skill] })
    setExpandedFolders((prev) => new Set(prev).add(id))
    openSkill(id)
    setContentBuffers((prev) => ({ ...prev, [id]: content }))
  }, [skills, updateLibrary, openSkill])

  const deleteSkill = useCallback((id: string) => {
    updateLibrary({ skills: skills.filter((s) => s.id !== id) })
    closeTab(id)
  }, [skills, updateLibrary, closeTab])

  const filteredSkills = searchQuery
    ? skills.filter((s) => {
        const q = searchQuery.toLowerCase()
        return s.name.toLowerCase().includes(q) || s.id.toLowerCase().includes(q)
      })
    : skills

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
