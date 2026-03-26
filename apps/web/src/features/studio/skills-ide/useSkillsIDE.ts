import { useState, useCallback, useEffect, useRef } from 'react'
import { useLibrary } from '#/features/compiler/useLibrary'
import type { Skill } from '@ship/ui'
import { newSkillTemplate, parseFrontmatter } from './skill-frontmatter'

const IDE_STATE_KEY = 'ship-skills-ide-v1'
const SKILL_FILES_KEY = 'ship-skills-files-v1'

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

/** Extra files per skill (everything except SKILL.md which lives in skill.content). */
function loadSkillFiles(): Record<string, Record<string, string>> {
  try {
    const raw = window.localStorage.getItem(SKILL_FILES_KEY)
    if (raw) return JSON.parse(raw)
  } catch { /* ignore */ }
  return {}
}

function persistSkillFiles(files: Record<string, Record<string, string>>) {
  try {
    window.localStorage.setItem(SKILL_FILES_KEY, JSON.stringify(files))
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

  // Local content buffers for unsaved edits (keyed by composite tab ID)
  const [contentBuffers, setContentBuffers] = useState<Record<string, string>>({})

  // Extra files per skill (not SKILL.md — those live in skill.content)
  const [skillFiles, setSkillFiles] = useState<Record<string, Record<string, string>>>(loadSkillFiles)

  // Persist layout state
  useEffect(() => {
    persistState({
      openTabIds,
      activeTabId,
      expandedFolders: Array.from(expandedFolders),
    })
  }, [openTabIds, activeTabId, expandedFolders])

  // Persist skill files
  useEffect(() => {
    persistSkillFiles(skillFiles)
  }, [skillFiles])

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
    return skillFiles[skillId]?.[filePath] ?? ''
  }, [skills, skillFiles])

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

  const saveSkill = useCallback((tabId: string) => {
    const content = contentBuffers[tabId]
    if (content === undefined) return

    const { skillId, filePath } = parseTabId(tabId)

    if (filePath === SKILL_MD) {
      const fm = parseFrontmatter(content)
      const updated = skills.map((s) => {
        if (s.id !== skillId) return s
        return {
          ...s,
          content,
          name: fm.name || s.name,
          description: fm.description || s.description,
        }
      })
      updateLibrary({ skills: updated })
    } else {
      // Save extra file
      setSkillFiles((prev) => ({
        ...prev,
        [skillId]: { ...prev[skillId], [filePath]: content },
      }))
    }
    setUnsavedIds((prev) => { const n = new Set(prev); n.delete(tabId); return n })
  }, [contentBuffers, skills, updateLibrary])

  const saveAll = useCallback(() => {
    if (unsavedIds.size === 0) return

    // Group unsaved tabs by type
    const skillUpdates = new Map<string, string>()
    const fileUpdates = new Map<string, Record<string, string>>()

    for (const tabId of unsavedIds) {
      const buf = contentBuffers[tabId]
      if (buf === undefined) continue
      const { skillId, filePath } = parseTabId(tabId)
      if (filePath === SKILL_MD) {
        skillUpdates.set(skillId, buf)
      } else {
        const existing = fileUpdates.get(skillId) ?? {}
        existing[filePath] = buf
        fileUpdates.set(skillId, existing)
      }
    }

    // Save SKILL.md updates to library
    if (skillUpdates.size > 0) {
      const updated = skills.map((s) => {
        const buf = skillUpdates.get(s.id)
        return buf !== undefined ? { ...s, content: buf } : s
      })
      updateLibrary({ skills: updated })
    }

    // Save extra file updates
    if (fileUpdates.size > 0) {
      setSkillFiles((prev) => {
        const next = { ...prev }
        for (const [skillId, files] of fileUpdates) {
          next[skillId] = { ...next[skillId], ...files }
        }
        return next
      })
    }

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

  const createSkill = useCallback((customId?: string) => {
    const id = customId || `new-skill-${Date.now()}`
    const name = id.replace(/-/g, ' ').replace(/\b\w/g, (c) => c.toUpperCase())
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
    const tabId = makeTabId(id, SKILL_MD)
    setOpenTabIds((prev) => (prev.includes(tabId) ? prev : [...prev, tabId]))
    setActiveTabId(tabId)
    setContentBuffers((prev) => ({ ...prev, [tabId]: content }))
  }, [skills, updateLibrary])

  const addFile = useCallback((skillId: string, filePath: string, content = '') => {
    setSkillFiles((prev) => ({
      ...prev,
      [skillId]: { ...prev[skillId], [filePath]: content },
    }))
    openFile(skillId, filePath)
    const tabId = makeTabId(skillId, filePath)
    setContentBuffers((prev) => ({ ...prev, [tabId]: content }))
    setUnsavedIds((prev) => new Set(prev).add(tabId))
  }, [openFile])

  const deleteFile = useCallback((skillId: string, filePath: string) => {
    if (filePath === SKILL_MD) return // Can't delete primary file
    setSkillFiles((prev) => {
      const next = { ...prev }
      if (next[skillId]) {
        const files = { ...next[skillId] }
        delete files[filePath]
        if (Object.keys(files).length === 0) delete next[skillId]
        else next[skillId] = files
      }
      return next
    })
    closeTab(makeTabId(skillId, filePath))
  }, [closeTab])

  const deleteSkill = useCallback((id: string) => {
    updateLibrary({ skills: skills.filter((s) => s.id !== id) })
    // Close all tabs for this skill
    setOpenTabIds((prev) => {
      const next = prev.filter((tid) => parseTabId(tid).skillId !== id)
      if (activeTabId && parseTabId(activeTabId).skillId === id) {
        setActiveTabId(next[0] ?? null)
      }
      return next
    })
    setSkillFiles((prev) => { const n = { ...prev }; delete n[id]; return n })
  }, [skills, updateLibrary, activeTabId])

  const getFilesForSkill = useCallback((skillId: string): string[] => {
    const extra = skillFiles[skillId] ? Object.keys(skillFiles[skillId]) : []
    return [SKILL_MD, ...extra.sort()]
  }, [skillFiles])

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
    skillFiles,
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
    addFile,
    deleteFile,
    deleteSkill,
    getFilesForSkill,
    setPreviewTab,
    setPreviewOpen,
  }
}
