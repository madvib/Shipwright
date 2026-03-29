// Viewer for non-HTML artifacts: markdown (with source/preview toggle),
// images (with click-to-zoom), JSON, and text files.

import { useMemo, useState } from 'react'
import { FileText, Image as ImageIcon, File, Eye, Code2, ZoomIn, ZoomOut } from 'lucide-react'
import { MarkdownPreview } from '#/features/studio/skills-ide/MarkdownPreview'
import type { SessionFile } from './types'

interface ArtifactViewerProps {
  file: SessionFile
  content: string
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
      <div className={`flex-1 overflow-auto bg-black/5 dark:bg-black/20 ${zoomed ? 'cursor-zoom-out' : 'cursor-zoom-in'}`} onClick={() => setZoomed(!zoomed)}>
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

// ── Markdown Viewer with source/preview toggle ──

function MarkdownViewer({ file, content }: { file: SessionFile; content: string }) {
  const [mode, setMode] = useState<'preview' | 'source'>('preview')

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
        <FileText className="size-3.5 text-emerald-500" />
        <span className="text-xs font-medium text-foreground">{file.name}</span>
        <div className="flex-1" />
        <div className="flex items-center rounded-md border border-border bg-muted/30 p-0.5">
          <button
            onClick={() => setMode('preview')}
            className={`flex items-center gap-1 rounded px-2 py-0.5 text-[10px] font-medium transition ${
              mode === 'preview' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Eye className="size-3" />
            Preview
          </button>
          <button
            onClick={() => setMode('source')}
            className={`flex items-center gap-1 rounded px-2 py-0.5 text-[10px] font-medium transition ${
              mode === 'source' ? 'bg-background text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'
            }`}
          >
            <Code2 className="size-3" />
            Source
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-auto">
        {mode === 'preview' ? (
          <MarkdownPreview content={content} />
        ) : (
          <pre className="px-6 py-4 text-xs font-mono text-foreground leading-relaxed whitespace-pre-wrap break-words">
            {content}
          </pre>
        )}
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

export function ArtifactViewer({ file, content }: ArtifactViewerProps) {
  if (file.type === 'image') return <ImageViewer file={file} content={content} />
  if (file.type === 'markdown') return <MarkdownViewer file={file} content={content} />
  if (file.name.endsWith('.json')) return <JsonViewer content={content} file={file} />
  return <TextViewer content={content} file={file} />
}
