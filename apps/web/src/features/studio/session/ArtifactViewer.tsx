// Viewer for non-HTML artifacts: markdown (milkdown editor),
// images (click-to-zoom), JSON, and text files.

import { useMemo, useState } from 'react'
import { FileText, Image as ImageIcon, File, ZoomIn, ZoomOut, Save } from 'lucide-react'
import { MarkdownEditor } from '@ship/primitives'
import { MarkdownPreview } from '#/features/studio/skills-ide/MarkdownPreview'
import type { SessionFile } from './types'

interface ArtifactViewerProps {
  file: SessionFile
  content: string
  /** Draft content (from useSessionDrafts). Falls back to `content` if undefined. */
  draftContent?: string
  /** Whether the draft is dirty */
  isDirty?: boolean
  /** Called when the user edits content */
  onContentChange?: (path: string, content: string) => void
  /** Called when the user saves (Cmd+S or button) */
  onSave?: (path: string) => void
  /** Called when user highlights text and adds a comment */
  onComment?: (selectedText: string, comment: string) => void
}

// ── Image Viewer with zoom ──

function ImageViewer({ file, content }: { file: SessionFile; content: string }) {
  const [zoomed, setZoomed] = useState(false)

  if (!content || !content.startsWith('data:')) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-3 p-8">
        <ImageIcon className="size-12 text-muted-foreground/40" />
        <p className="text-sm text-muted-foreground">{file.name}</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <ImageIcon className="size-3.5 text-amber-500" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
        <div className="flex-1" />
        <button
          onClick={() => setZoomed(!zoomed)}
          className="flex items-center gap-1 rounded px-2 py-0.5 text-[11px] text-muted-foreground hover:text-foreground hover:bg-muted transition"
        >
          {zoomed ? <ZoomOut className="size-3" /> : <ZoomIn className="size-3" />}
          {zoomed ? 'Fit' : 'Zoom'}
        </button>
      </div>
      <div
        className={`flex-1 overflow-auto bg-black/5 dark:bg-black/20 ${zoomed ? 'cursor-zoom-out' : 'cursor-zoom-in'}`}
        onClick={() => setZoomed(!zoomed)}
      >
        <div className={`flex items-center justify-center ${zoomed ? 'p-4' : 'h-full p-4'}`}>
          <img
            src={content}
            alt={file.name}
            className={`rounded ${zoomed ? 'max-w-none' : 'max-w-full max-h-full object-contain'}`}
          />
        </div>
      </div>
    </div>
  )
}

// ── Markdown Editor (milkdown with draft support) ──

function MarkdownFileEditor({
  file, content, draftContent, isDirty, onContentChange, onSave, onComment,
}: {
  file: SessionFile
  content: string
  draftContent?: string
  isDirty?: boolean
  onContentChange?: (path: string, content: string) => void
  onSave?: (path: string) => void
  onComment?: (selectedText: string, comment: string) => void
}) {
  const editable = onContentChange != null
  const displayContent = draftContent ?? content

  if (!editable) {
    // Read-only fallback
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
          <FileText className="size-3.5 text-emerald-500" />
          <span className="text-xs font-medium text-foreground">{file.name}</span>
        </div>
        <div className="flex-1 overflow-auto">
          <MarkdownPreview content={displayContent} />
        </div>
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <FileText className="size-3.5 text-emerald-500" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
        {isDirty && <span className="size-1.5 rounded-full bg-primary" title="Unsaved changes" />}
        <div className="flex-1" />
        {isDirty && onSave && (
          <button
            onClick={() => onSave(file.path)}
            className="flex items-center gap-1 rounded px-2 py-0.5 text-[11px] font-medium text-primary hover:bg-primary/10 transition"
          >
            <Save className="size-3" />
            Save
          </button>
        )}
      </div>
      <div className="flex-1 overflow-hidden">
        <MarkdownEditor
          value={displayContent}
          onChange={(v) => onContentChange(file.path, v)}
          fillHeight
          showStats={false}
          showAiActions={false}
          onComment={onComment}
        />
      </div>
    </div>
  )
}

// ── JSON Viewer ──

function JsonViewer({ content, file }: { content: string; file: SessionFile }) {
  const formatted = useMemo(() => {
    try { return JSON.stringify(JSON.parse(content), null, 2) }
    catch { return content }
  }, [content])

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <File className="size-3.5 text-muted-foreground" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
      </div>
      <pre className="flex-1 px-6 py-4 text-xs font-mono text-foreground leading-relaxed overflow-auto whitespace-pre-wrap break-words">
        {formatted}
      </pre>
    </div>
  )
}

// ── Text Viewer ──

function TextViewer({ content, file }: { content: string; file: SessionFile }) {
  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <File className="size-3.5 text-muted-foreground" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
      </div>
      <pre className="flex-1 px-6 py-4 text-xs font-mono text-foreground leading-relaxed overflow-auto whitespace-pre-wrap break-words">
        {content}
      </pre>
    </div>
  )
}

// ── Router ──

export function ArtifactViewer({ file, content, draftContent, isDirty, onContentChange, onSave, onComment }: ArtifactViewerProps) {
  if (file.type === 'image') return <ImageViewer file={file} content={content} />
  if (file.type === 'markdown') {
    return (
      <MarkdownFileEditor
        file={file}
        content={content}
        draftContent={draftContent}
        isDirty={isDirty}
        onContentChange={onContentChange}
        onSave={onSave}
        onComment={onComment}
      />
    )
  }
  if (file.name.endsWith('.json')) return <JsonViewer content={content} file={file} />
  return <TextViewer content={content} file={file} />
}
