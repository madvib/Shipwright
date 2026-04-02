// Stable callbacks for the Session page: file ops, annotation I/O, drag-drop.
// Extracted to keep session.tsx under the 300-line cap.

import { useState, useCallback } from 'react'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import type { useAnnotations } from './useAnnotations'
import type { DiffComment } from './DiffViewer'

type Ann = ReturnType<typeof useAnnotations>

interface HandlerDeps {
  workspaceId: string
  ann: Ann
  openFile: (path: string) => void
  closeTab: (path: string) => void
  setViewMode: (mode: 'file' | 'diff') => void
  setSelectedCommitHash: (hash: string | null) => void
  uploadMutate: (file: File) => void
  deleteMutate: (path: string) => void
}

async function daemonWriteSessionFile(wsId: string, path: string, content: string): Promise<void> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/workspaces/${encodeURIComponent(wsId)}/session-files/${encodeURIComponent(path)}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content }),
  })
  if (!res.ok) throw new Error(`daemon: write session file ${res.status}`)
}

async function daemonEmitEvent(eventType: string, payload: Record<string, unknown>, workspaceId?: string): Promise<void> {
  const res = await fetch(`${DAEMON_BASE_URL}/api/events/emit`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      event_type: eventType,
      workspace_id: workspaceId,
      payload,
    }),
  })
  if (!res.ok) throw new Error(`daemon: emit event ${res.status}`)
}

export function useSessionHandlers({
  workspaceId, ann,
  openFile, closeTab, setViewMode, setSelectedCommitHash,
  uploadMutate, deleteMutate,
}: HandlerDeps) {
  const [isDragging, setIsDragging] = useState(false)

  const handleDeleteFile = useCallback((path: string) => {
    closeTab(path)
    deleteMutate(path)
  }, [closeTab, deleteMutate])

  const handleExport = useCallback(async () => {
    const content = JSON.stringify(ann.toExportJSON(), null, 2)
    try {
      await daemonWriteSessionFile(workspaceId, 'annotations.json', content)
      return
    } catch { /* fall through to browser download */ }
    const blob = new Blob([content], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = 'annotations.json'; a.click()
    URL.revokeObjectURL(url)
  }, [ann, workspaceId])

  const handleComment = useCallback((selectedText: string, comment: string) => {
    ann.addActionAnnotation(`comment: ${comment}`, selectedText)
  }, [ann])

  const handleDiffComment = useCallback((comment: DiffComment) => {
    ann.addActionAnnotation(
      `diff-comment: ${comment.comment}`,
      `${comment.file}:${comment.lineNum} ${comment.content.slice(0, 100)}`,
    )
    daemonWriteSessionFile(workspaceId, 'diff-comments.json', JSON.stringify(comment, null, 2)).catch(() => {})
  }, [ann, workspaceId])

  const handleUploadFiles = useCallback((fileList: FileList) => {
    for (let i = 0; i < fileList.length; i++) uploadMutate(fileList[i])
  }, [uploadMutate])

  const handleShowDiff = useCallback(() => {
    setSelectedCommitHash(null)
    setViewMode('diff')
  }, [setViewMode, setSelectedCommitHash])

  const handleSelectCommit = useCallback((hash: string) => {
    setSelectedCommitHash(hash)
    setViewMode('diff')
  }, [setViewMode, setSelectedCommitHash])

  const handleNavigateToAnnotation = useCallback((filePath: string, annotationId: string) => {
    openFile(filePath)
    ann.toggleActiveId(annotationId)
  }, [openFile, ann])

  const handleSendToAgent = useCallback(async (summary: string) => {
    const annotations = ann.allStaged.map(({ filePath, ann: a }) => {
      if (a.type === 'click') return { filePath, type: 'click', selector: a.selector, text: a.text, note: a.note, x: a.x, y: a.y, timestamp: a.timestamp }
      if (a.type === 'box') return { filePath, type: 'box', rect: a.rect, elements: a.elements, note: a.note, timestamp: a.timestamp }
      return { filePath, type: 'action', action: a.action, text: a.text, timestamp: a.timestamp }
    })
    await daemonEmitEvent('studio.message', { summary: summary || undefined, annotations }, workspaceId)
    ann.clearAllAnnotations()
  }, [workspaceId, ann])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    if (e.dataTransfer.types.includes('Files')) setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    if (e.currentTarget === e.target || !e.currentTarget.contains(e.relatedTarget as Node)) {
      setIsDragging(false)
    }
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
    if (!e.dataTransfer.files.length) return
    handleUploadFiles(e.dataTransfer.files)
  }, [handleUploadFiles])

  return {
    isDragging,
    handleDeleteFile,
    handleExport,
    handleComment,
    handleDiffComment,
    handleUploadFiles,
    handleShowDiff,
    handleSelectCommit,
    handleNavigateToAnnotation,
    handleSendToAgent,
    handleDragOver,
    handleDragLeave,
    handleDrop,
  }
}
