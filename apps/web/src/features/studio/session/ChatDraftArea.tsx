// Compose area for the Chat tab: text input, staged annotation cards, send button.
// Drag-and-drop files onto the area uploads them via onUploadFiles.

import { useState, useCallback, useRef, useEffect } from 'react'
import { Send, Paperclip } from 'lucide-react'
import { AnnotationCard } from './AnnotationCard'
import type { StagedAnnotation } from './types'

interface Props {
  stagedAnnotations: StagedAnnotation[]
  onSend: (text: string) => Promise<void>
  onRemoveAnnotation: (id: string) => void
  onUploadFiles: (files: FileList) => void
  disabled?: boolean
}

export function ChatDraftArea({
  stagedAnnotations,
  onSend,
  onRemoveAnnotation,
  onUploadFiles,
  disabled = false,
}: Props) {
  const [text, setText] = useState('')
  const [sending, setSending] = useState(false)
  const [isDragOver, setIsDragOver] = useState(false)
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)

  // Auto-focus when staged count changes to > 0
  useEffect(() => {
    if (stagedAnnotations.length > 0) textareaRef.current?.focus()
  }, [stagedAnnotations.length])

  const handleSend = useCallback(async () => {
    if (sending || disabled) return
    setSending(true)
    try {
      await onSend(text.trim())
      setText('')
    } finally {
      setSending(false)
    }
  }, [sending, disabled, text, onSend])

  // Cmd+Enter / Ctrl+Enter to send
  const handleKeyDown = useCallback((e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault()
      void handleSend()
    }
  }, [handleSend])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    if (e.dataTransfer.types.includes('Files')) setIsDragOver(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    if (!e.currentTarget.contains(e.relatedTarget as Node)) setIsDragOver(false)
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setIsDragOver(false)
    if (e.dataTransfer.files.length) onUploadFiles(e.dataTransfer.files)
  }, [onUploadFiles])

  const canSend = (text.trim().length > 0 || stagedAnnotations.length > 0) && !disabled

  return (
    <div
      className={`border-t border-border/50 flex flex-col gap-1.5 p-2 shrink-0 transition-colors ${isDragOver ? 'bg-primary/5 border-primary/30' : ''}`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      {isDragOver && (
        <div className="flex items-center justify-center py-2 text-[11px] text-primary/70 font-medium">
          Drop files to upload
        </div>
      )}

      {stagedAnnotations.length > 0 && (
        <div className="flex flex-col gap-1 max-h-32 overflow-y-auto">
          {stagedAnnotations.map((s) => (
            <AnnotationCard key={s.ann.id} staged={s} onRemove={onRemoveAnnotation} />
          ))}
        </div>
      )}

      <div className="flex items-end gap-1.5">
        <textarea
          ref={textareaRef}
          value={text}
          onChange={(e) => setText(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={disabled ? 'Connect CLI to send messages' : 'Message agent… (⌘↵ to send)'}
          disabled={disabled}
          rows={2}
          className="flex-1 resize-none rounded-md bg-muted/20 border border-border/40 px-2.5 py-1.5 text-xs text-foreground placeholder:text-muted-foreground/50 focus:outline-none focus:border-primary/50 transition disabled:opacity-50 disabled:cursor-not-allowed"
        />
        <div className="flex flex-col gap-1 shrink-0">
          <button
            onClick={() => fileInputRef.current?.click()}
            disabled={disabled}
            className="flex items-center justify-center size-7 rounded-md border border-border/40 text-muted-foreground hover:text-foreground hover:bg-muted/30 transition disabled:opacity-40 disabled:cursor-not-allowed"
            title="Upload files"
            aria-label="Upload files"
          >
            <Paperclip className="size-3.5" />
          </button>
          <button
            onClick={() => void handleSend()}
            disabled={!canSend || sending}
            className="flex items-center justify-center size-7 rounded-md bg-primary text-primary-foreground hover:bg-primary/90 transition disabled:opacity-40 disabled:cursor-not-allowed"
            title="Send (⌘↵)"
            aria-label="Send message"
          >
            <Send className="size-3.5" />
          </button>
        </div>
      </div>

      <input
        ref={fileInputRef}
        type="file"
        multiple
        className="hidden"
        onChange={(e) => {
          if (e.target.files?.length) onUploadFiles(e.target.files)
          e.target.value = ''
        }}
      />
    </div>
  )
}
