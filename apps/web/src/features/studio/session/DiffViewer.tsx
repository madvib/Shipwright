// Unified diff viewer with inline comments.
// Renders parsed git diff with collapsible files, colored hunks, and comment support.

import { useState, useMemo, useCallback } from 'react'
import { ChevronRight, ChevronDown, FileCode, Plus, Minus, MessageSquarePlus } from 'lucide-react'
import { parseDiff, type DiffFile, type DiffHunk, type DiffLine } from './diff-parser'

interface DiffComment {
  file: string
  lineNum: number
  lineType: DiffLine['type']
  content: string
  comment: string
  timestamp: string
}

interface DiffViewerProps {
  diffText: string
  onComment?: (comment: DiffComment) => void
}

function FileHeader({ file, isOpen, onToggle }: { file: DiffFile; isOpen: boolean; onToggle: () => void }) {
  return (
    <button onClick={onToggle} className="flex items-center gap-2 w-full px-3 py-2 border-b border-border/40 hover:bg-muted/30 transition text-left">
      {isOpen ? <ChevronDown className="size-3 text-muted-foreground shrink-0" /> : <ChevronRight className="size-3 text-muted-foreground shrink-0" />}
      <FileCode className="size-3.5 text-muted-foreground shrink-0" />
      <span className="text-xs font-mono text-foreground truncate flex-1">{file.path}</span>
      <span className="flex items-center gap-2 shrink-0">
        {file.additions > 0 && <span className="flex items-center gap-0.5 text-[10px] text-emerald-600 dark:text-green-400"><Plus className="size-2.5" />{file.additions}</span>}
        {file.deletions > 0 && <span className="flex items-center gap-0.5 text-[10px] text-red-600 dark:text-red-400"><Minus className="size-2.5" />{file.deletions}</span>}
      </span>
    </button>
  )
}

function HunkHeader({ header }: { header: string }) {
  return (
    <div className="px-3 py-1 bg-sky-500/10 dark:bg-blue-900/20 border-y border-sky-500/20 dark:border-blue-800/20 text-[11px] font-mono text-sky-600 dark:text-blue-300/80 select-none">
      {header}
    </div>
  )
}

function DiffLineRow({ line, filePath, onComment }: {
  line: DiffLine
  filePath: string
  onComment?: (comment: DiffComment) => void
}) {
  const [showInput, setShowInput] = useState(false)
  const [commentText, setCommentText] = useState('')

  const bgClass =
    line.type === 'add' ? 'bg-emerald-500/10 dark:bg-green-900/30' :
    line.type === 'del' ? 'bg-red-500/10 dark:bg-red-900/30' : ''

  const textClass =
    line.type === 'add' ? 'text-emerald-700 dark:text-green-300' :
    line.type === 'del' ? 'text-red-700 dark:text-red-300' : 'text-muted-foreground'

  const indicator = line.type === 'add' ? '+' : line.type === 'del' ? '-' : ' '

  const handleSubmit = () => {
    if (!commentText.trim() || !onComment) return
    onComment({
      file: filePath,
      lineNum: line.newNum ?? line.oldNum ?? 0,
      lineType: line.type,
      content: line.content,
      comment: commentText.trim(),
      timestamp: new Date().toISOString(),
    })
    setCommentText('')
    setShowInput(false)
  }

  return (
    <>
      <div className={`group flex font-mono text-[11px] leading-5 ${bgClass}`}>
        <span className="w-10 shrink-0 text-right pr-2 text-muted-foreground/50 select-none border-r border-border/20">{line.oldNum ?? ''}</span>
        <span className="w-10 shrink-0 text-right pr-2 text-muted-foreground/50 select-none border-r border-border/20">{line.newNum ?? ''}</span>
        <span className={`w-4 shrink-0 text-center select-none ${textClass}`}>{indicator}</span>
        <span className={`flex-1 pr-3 whitespace-pre-wrap break-all ${textClass}`}>{line.content}</span>
        {onComment && (
          <button
            onClick={() => setShowInput(true)}
            className="shrink-0 px-1.5 opacity-0 group-hover:opacity-100 text-primary/60 hover:text-primary transition-opacity"
            title="Add comment"
          >
            <MessageSquarePlus className="size-3.5" />
          </button>
        )}
      </div>
      {showInput && (
        <div className="flex items-start gap-2 px-3 py-2 bg-muted/30 border-y border-border/30">
          <textarea
            autoFocus
            value={commentText}
            onChange={(e) => setCommentText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) { e.preventDefault(); handleSubmit() }
              if (e.key === 'Escape') { setShowInput(false); setCommentText('') }
            }}
            placeholder="Add a comment on this line..."
            className="flex-1 rounded-md border border-border bg-background px-2.5 py-1.5 text-xs outline-none focus:border-primary/50 resize-none"
            rows={2}
          />
          <div className="flex flex-col gap-1">
            <button onClick={handleSubmit} disabled={!commentText.trim()} className="rounded-md bg-primary px-2.5 py-1 text-[10px] font-medium text-primary-foreground disabled:opacity-40">
              Comment
            </button>
            <button onClick={() => { setShowInput(false); setCommentText('') }} className="rounded-md px-2.5 py-1 text-[10px] text-muted-foreground hover:text-foreground">
              Cancel
            </button>
          </div>
        </div>
      )}
    </>
  )
}

function FileSection({ file, onComment }: { file: DiffFile; onComment?: (c: DiffComment) => void }) {
  return (
    <div className="border-b border-border/20">
      {file.hunks.map((hunk: DiffHunk, hi: number) => (
        <div key={hi}>
          <HunkHeader header={hunk.header} />
          {hunk.lines.map((line: DiffLine, li: number) => (
            <DiffLineRow key={li} line={line} filePath={file.path} onComment={onComment} />
          ))}
        </div>
      ))}
    </div>
  )
}

export function DiffViewer({ diffText, onComment }: DiffViewerProps) {
  const parsed = useMemo(() => parseDiff(diffText), [diffText])
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set())

  const toggleFile = useCallback((path: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev)
      next.has(path) ? next.delete(path) : next.add(path)
      return next
    })
  }, [])

  if (parsed.files.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        <p className="text-sm">No changes to display</p>
      </div>
    )
  }

  const totalAdds = parsed.files.reduce((s, f) => s + f.additions, 0)
  const totalDels = parsed.files.reduce((s, f) => s + f.deletions, 0)

  return (
    <div className="flex flex-col h-full min-h-0">
      <div className="flex items-center gap-3 px-4 py-2 border-b border-border/60 shrink-0 bg-card/50">
        <span className="text-xs text-muted-foreground">{parsed.files.length} file{parsed.files.length !== 1 ? 's' : ''} changed</span>
        {totalAdds > 0 && <span className="text-[11px] text-emerald-600 dark:text-green-400">+{totalAdds}</span>}
        {totalDels > 0 && <span className="text-[11px] text-red-600 dark:text-red-400">-{totalDels}</span>}
      </div>
      <div className="flex-1 overflow-auto">
        {parsed.files.map((file) => (
          <div key={file.path}>
            <FileHeader file={file} isOpen={!collapsed.has(file.path)} onToggle={() => toggleFile(file.path)} />
            {!collapsed.has(file.path) && <FileSection file={file} onComment={onComment} />}
          </div>
        ))}
      </div>
    </div>
  )
}

export type { DiffComment }
