// Stable callbacks for the Session page: file ops, annotation I/O, drag-drop.
// Extracted to keep session.tsx under the 300-line cap.

import { useState, useCallback } from 'react'
import type { UseLocalMcpReturn } from '#/features/studio/useLocalMcp'
import type { useAnnotations } from './useAnnotations'
import type { useSessionDrafts } from './useSessionDrafts'
import type { DiffComment } from './DiffViewer'

type Ann = ReturnType<typeof useAnnotations>
type Drafts = ReturnType<typeof useSessionDrafts>

interface HandlerDeps {
  mcp: UseLocalMcpReturn | null
  isConnected: boolean
  ann: Ann
  drafts: Drafts
  openFile: (path: string) => void
  closeTab: (path: string) => void
  setViewMode: (mode: 'file' | 'diff') => void
  setSelectedCommitHash: (hash: string | null) => void
  uploadMutate: (file: File) => void
  deleteMutate: (path: string) => void
}

export function useSessionHandlers({
  mcp, isConnected, ann, drafts,
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
    if (mcp && isConnected) {
      try {
        await mcp.callTool('write_session_file', { path: 'annotations.json', content })
        return
      } catch { /* fall through */ }
    }
    const blob = new Blob([content], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url; a.download = 'annotations.json'; a.click()
    URL.revokeObjectURL(url)
  }, [ann, mcp, isConnected])

  const handleComment = useCallback((selectedText: string, comment: string) => {
    ann.addActionAnnotation(`comment: ${comment}`, selectedText)
  }, [ann])

  const handleDiffComment = useCallback((comment: DiffComment) => {
    ann.addActionAnnotation(
      `diff-comment: ${comment.comment}`,
      `${comment.file}:${comment.lineNum} ${comment.content.slice(0, 100)}`,
    )
    if (mcp && isConnected) {
      mcp.callTool('write_session_file', {
        path: 'diff-comments.json',
        content: JSON.stringify(comment, null, 2),
      }).catch(() => {})
    }
  }, [ann, mcp, isConnected])

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
    if (!mcp) return
    const annotations = ann.allStaged.map(({ filePath, ann: a }) => {
      if (a.type === 'click') return { filePath, type: 'click', selector: a.selector, text: a.text, note: a.note, x: a.x, y: a.y, timestamp: a.timestamp }
      if (a.type === 'box') return { filePath, type: 'box', rect: a.rect, elements: a.elements, note: a.note, timestamp: a.timestamp }
      return { filePath, type: 'action', action: a.action, text: a.text, timestamp: a.timestamp }
    })
    await mcp.postStudioEvent('studio.message', {
      summary: summary || undefined,
      annotations,
    }, false)
    ann.clearAllAnnotations()
  }, [mcp, ann])

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
    if (!isConnected || !e.dataTransfer.files.length) return
    handleUploadFiles(e.dataTransfer.files)
  }, [isConnected, handleUploadFiles])

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
