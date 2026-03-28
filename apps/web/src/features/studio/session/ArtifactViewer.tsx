// Main-area viewer for non-HTML artifacts: markdown, images, and JSON/text.
// Renders in the canvas slot when a non-HTML file is selected.

import { useMemo } from 'react'
import { FileText, Image as ImageIcon, File } from 'lucide-react'
import { MarkdownPreview } from '#/features/studio/skills-ide/MarkdownPreview'
import type { SessionFile } from './types'

interface ArtifactViewerProps {
  file: SessionFile
  content: string
}

function JsonViewer({ content }: { content: string }) {
  const formatted = useMemo(() => {
    try {
      return JSON.stringify(JSON.parse(content), null, 2)
    } catch {
      return content
    }
  }, [content])

  return (
    <pre className="px-6 py-4 text-xs font-mono text-foreground leading-relaxed overflow-auto whitespace-pre-wrap break-words">
      {formatted}
    </pre>
  )
}

function ImageViewer({ file }: { file: SessionFile }) {
  // Images from .ship-session/ are served via the MCP read_session_file tool,
  // but we cannot get a direct URL. For now show a placeholder indicating the
  // file exists. A future iteration could use base64 data URIs.
  return (
    <div className="flex flex-col items-center justify-center h-full gap-3 p-8">
      <ImageIcon className="size-12 text-muted-foreground/40" />
      <div className="text-center">
        <p className="text-sm font-medium text-foreground">{file.name}</p>
        <p className="text-xs text-muted-foreground mt-1">
          Image preview requires direct file access.
        </p>
      </div>
    </div>
  )
}

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

export function ArtifactViewer({ file, content }: ArtifactViewerProps) {
  if (file.type === 'image') {
    return <ImageViewer file={file} />
  }

  if (file.type === 'markdown') {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
          <FileText className="size-3.5 text-emerald-500" />
          <span className="text-xs font-medium text-foreground">{file.name}</span>
        </div>
        <div className="flex-1 overflow-auto">
          <MarkdownPreview content={content} />
        </div>
      </div>
    )
  }

  if (file.name.endsWith('.json')) {
    return (
      <div className="flex flex-col h-full">
        <div className="flex items-center gap-2 px-4 py-2 border-b border-border/60 shrink-0 bg-card/30">
          <File className="size-3.5 text-muted-foreground" />
          <span className="text-xs font-medium text-foreground">{file.name}</span>
        </div>
        <div className="flex-1 overflow-auto">
          <JsonViewer content={content} />
        </div>
      </div>
    )
  }

  return <TextViewer content={content} file={file} />
}
