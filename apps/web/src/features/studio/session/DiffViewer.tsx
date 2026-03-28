// Unified diff viewer — renders parsed git diff output as a native React component.
// Supports file list, collapsible per-file sections, and line-by-line coloring.

import { useState, useMemo, useCallback } from 'react'
import { ChevronRight, ChevronDown, FileCode, Plus, Minus } from 'lucide-react'
import { parseDiff, type DiffFile, type DiffHunk, type DiffLine } from './diff-parser'

interface DiffViewerProps {
  diffText: string
}

function FileHeader({ file, isOpen, onToggle }: { file: DiffFile; isOpen: boolean; onToggle: () => void }) {
  return (
    <button
      onClick={onToggle}
      className="flex items-center gap-2 w-full px-3 py-2 border-b border-border/40 hover:bg-muted/30 transition text-left"
    >
      {isOpen
        ? <ChevronDown className="size-3 text-muted-foreground shrink-0" />
        : <ChevronRight className="size-3 text-muted-foreground shrink-0" />}
      <FileCode className="size-3.5 text-muted-foreground shrink-0" />
      <span className="text-xs font-mono text-foreground truncate flex-1">{file.path}</span>
      <span className="flex items-center gap-2 shrink-0">
        {file.additions > 0 && (
          <span className="flex items-center gap-0.5 text-[10px] text-green-400">
            <Plus className="size-2.5" />
            {file.additions}
          </span>
        )}
        {file.deletions > 0 && (
          <span className="flex items-center gap-0.5 text-[10px] text-red-400">
            <Minus className="size-2.5" />
            {file.deletions}
          </span>
        )}
      </span>
    </button>
  )
}

function HunkHeader({ header }: { header: string }) {
  return (
    <div className="px-3 py-1 bg-blue-900/20 border-y border-blue-800/20 text-[11px] font-mono text-blue-300/80 select-none">
      {header}
    </div>
  )
}

function DiffLineRow({ line }: { line: DiffLine }) {
  const bgClass =
    line.type === 'add' ? 'bg-green-900/30' :
    line.type === 'del' ? 'bg-red-900/30' : ''

  const textClass =
    line.type === 'add' ? 'text-green-300' :
    line.type === 'del' ? 'text-red-300' : 'text-muted-foreground'

  const indicator =
    line.type === 'add' ? '+' :
    line.type === 'del' ? '-' : ' '

  return (
    <div className={`flex font-mono text-[11px] leading-5 ${bgClass}`}>
      <span className="w-10 shrink-0 text-right pr-2 text-muted-foreground/50 select-none border-r border-border/20">
        {line.oldNum ?? ''}
      </span>
      <span className="w-10 shrink-0 text-right pr-2 text-muted-foreground/50 select-none border-r border-border/20">
        {line.newNum ?? ''}
      </span>
      <span className={`w-4 shrink-0 text-center select-none ${textClass}`}>{indicator}</span>
      <span className={`flex-1 pr-3 whitespace-pre-wrap break-all ${textClass}`}>{line.content}</span>
    </div>
  )
}

function FileSection({ file }: { file: DiffFile }) {
  return (
    <div className="border-b border-border/20">
      {file.hunks.map((hunk: DiffHunk, hi: number) => (
        <div key={hi}>
          <HunkHeader header={hunk.header} />
          {hunk.lines.map((line: DiffLine, li: number) => (
            <DiffLineRow key={li} line={line} />
          ))}
        </div>
      ))}
    </div>
  )
}

export function DiffViewer({ diffText }: DiffViewerProps) {
  const parsed = useMemo(() => parseDiff(diffText), [diffText])
  const [collapsed, setCollapsed] = useState<Set<string>>(new Set())

  const toggleFile = useCallback((path: string) => {
    setCollapsed((prev) => {
      const next = new Set(prev)
      if (next.has(path)) next.delete(path)
      else next.add(path)
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
      {/* Summary bar */}
      <div className="flex items-center gap-3 px-4 py-2 border-b border-border/60 shrink-0 bg-card/50">
        <span className="text-xs text-muted-foreground">
          {parsed.files.length} file{parsed.files.length !== 1 ? 's' : ''} changed
        </span>
        {totalAdds > 0 && <span className="text-[11px] text-green-400">+{totalAdds}</span>}
        {totalDels > 0 && <span className="text-[11px] text-red-400">-{totalDels}</span>}
      </div>

      {/* Scrollable diff content */}
      <div className="flex-1 overflow-auto">
        {parsed.files.map((file) => (
          <div key={file.path}>
            <FileHeader
              file={file}
              isOpen={!collapsed.has(file.path)}
              onToggle={() => toggleFile(file.path)}
            />
            {!collapsed.has(file.path) && <FileSection file={file} />}
          </div>
        ))}
      </div>
    </div>
  )
}
