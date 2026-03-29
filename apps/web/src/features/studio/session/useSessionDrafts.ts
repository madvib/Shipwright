// Draft state management for session file editing.
// Same IndexedDB caching pattern as the Skills IDE (useSkillsIDE.ts).
// Drafts persist across page reloads. Save writes to MCP.

import { useState, useCallback, useEffect, useRef, useMemo } from 'react'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useLocalMcpContext } from '#/features/studio/LocalMcpContext'
import { idbGet, idbSet } from '#/lib/idb-cache'
import { sessionKeys } from './query-keys'

const DRAFTS_KEY = 'ship-session-drafts-v1'
const SAVE_DELAY = 800

export interface SessionDraft {
  content: string
  originalContent: string
}

export interface UseSessionDraftsReturn {
  /** Get draft content for a path, or undefined if no draft exists */
  getDraft: (path: string) => string | undefined
  /** Initialize a draft from server content (no-op if draft already exists) */
  openFile: (path: string, serverContent: string) => void
  /** Update draft content */
  updateContent: (path: string, content: string) => void
  /** Whether the draft for this path differs from original */
  isDirty: (path: string) => boolean
  /** Set of all paths with unsaved changes */
  unsavedPaths: Set<string>
  /** Save a single file to MCP */
  saveFile: (path: string) => void
  /** Whether a save is in progress */
  isSaving: boolean
}

export function useSessionDrafts(): UseSessionDraftsReturn {
  const mcp = useLocalMcpContext()
  const queryClient = useQueryClient()
  const [drafts, setDrafts] = useState<Record<string, SessionDraft>>({})
  const loadedRef = useRef(false)
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Load from IndexedDB on mount
  useEffect(() => {
    let cancelled = false
    async function load() {
      try {
        const stored = await idbGet<Record<string, SessionDraft>>(DRAFTS_KEY)
        if (stored && !cancelled) setDrafts(stored)
      } catch { /* IDB unavailable */ }
      loadedRef.current = true
    }
    void load()
    return () => { cancelled = true }
  }, [])

  // Debounced save to IndexedDB
  useEffect(() => {
    if (!loadedRef.current) return
    if (saveTimerRef.current) clearTimeout(saveTimerRef.current)
    saveTimerRef.current = setTimeout(() => {
      idbSet(DRAFTS_KEY, drafts).catch(() => {})
    }, SAVE_DELAY)
    return () => { if (saveTimerRef.current) clearTimeout(saveTimerRef.current) }
  }, [drafts])

  const getDraft = useCallback((path: string): string | undefined => {
    return drafts[path]?.content
  }, [drafts])

  const openFile = useCallback((path: string, serverContent: string) => {
    setDrafts((prev) => {
      // Don't overwrite existing draft — it may have unsaved edits
      if (prev[path]) return prev
      return { ...prev, [path]: { content: serverContent, originalContent: serverContent } }
    })
  }, [])

  const updateContent = useCallback((path: string, content: string) => {
    setDrafts((prev) => {
      const existing = prev[path]
      if (!existing) {
        return { ...prev, [path]: { content, originalContent: content } }
      }
      return { ...prev, [path]: { ...existing, content } }
    })
  }, [])

  const isDirty = useCallback((path: string): boolean => {
    const draft = drafts[path]
    if (!draft) return false
    return draft.content !== draft.originalContent
  }, [drafts])

  const unsavedPaths = useMemo(() => {
    const paths = new Set<string>()
    for (const [path, draft] of Object.entries(drafts)) {
      if (draft.content !== draft.originalContent) paths.add(path)
    }
    return paths
  }, [drafts])

  // Save to MCP
  const saveMutation = useMutation({
    mutationFn: async ({ path, content }: { path: string; content: string }) => {
      if (!mcp) throw new Error('Not connected')
      await mcp.callTool('write_session_file', { path, content })
    },
    onSuccess: (_, { path }) => {
      // Mark draft as saved
      setDrafts((prev) => {
        const draft = prev[path]
        if (!draft) return prev
        return { ...prev, [path]: { ...draft, originalContent: draft.content } }
      })
      // Invalidate file content cache
      void queryClient.invalidateQueries({ queryKey: sessionKeys.fileContent(path) })
      void queryClient.invalidateQueries({ queryKey: sessionKeys.files() })
    },
  })

  const saveFile = useCallback((path: string) => {
    const draft = drafts[path]
    if (!draft) return
    saveMutation.mutate({ path, content: draft.content })
  }, [drafts, saveMutation])

  return {
    getDraft,
    openFile,
    updateContent,
    isDirty,
    unsavedPaths,
    saveFile,
    isSaving: saveMutation.isPending,
  }
}
