/** Editable text area with syntax overlay for the Skills IDE. */

import { useRef, useCallback, useMemo } from 'react'
import { FileQuestion } from 'lucide-react'
import { highlightLines, renderInlineCode } from './editor-highlight'

interface Props {
  tabId: string
  content: string
  onContentChange: (id: string, content: string) => void
  onSave: (id: string) => void
}

export function TextEditor({ tabId, content, onContentChange, onSave }: Props) {
  const textareaRef = useRef<HTMLTextAreaElement>(null)
  const lines = useMemo(() => highlightLines(content), [content])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault()
        onSave(tabId)
      }
    },
    [tabId, onSave],
  )

  const fileIsEmpty = content.trim() === ''

  if (fileIsEmpty) {
    return (
      <div className="relative flex flex-1 flex-col items-center justify-center gap-2 text-center min-h-0">
        <FileQuestion className="size-8 text-muted-foreground opacity-50" />
        <p className="text-xs text-muted-foreground">This file is empty</p>
        <p className="text-[11px] text-muted-foreground">Start typing to add content.</p>
        <textarea
          ref={textareaRef}
          value={content}
          onChange={(e) => onContentChange(tabId, e.target.value)}
          className="absolute inset-0 w-full h-full opacity-0 cursor-text"
          autoFocus
        />
      </div>
    )
  }

  return (
    <div className="flex-1 min-h-0 overflow-auto" onKeyDown={handleKeyDown}>
      <div className="flex min-h-full">
        <div className="shrink-0 w-10 pt-4 pb-4 text-right pr-2 font-mono text-[11px] leading-[1.7] text-muted-foreground select-none border-r border-border sticky left-0 bg-background/80">
          {lines.map((_, i) => <div key={i}>{i + 1}</div>)}
        </div>
        <div className="flex-1 relative min-w-0">
          <div className="px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] pointer-events-none whitespace-pre-wrap break-words" aria-hidden="true">
            {lines.map((line, i) => (
              <div key={i} className={line.className}>
                {'fragments' in line && line.fragments
                  ? line.fragments.map((f, fi) => <span key={fi} className={f.className}>{renderInlineCode(f.text)}</span>)
                  : renderInlineCode(line.text)}
              </div>
            ))}
          </div>
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => onContentChange(tabId, e.target.value)}
            className="absolute inset-0 w-full h-full px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] text-transparent caret-foreground bg-transparent resize-none focus:outline-none whitespace-pre-wrap break-words selection:bg-primary/25 selection:text-transparent"
            spellCheck={false}
            autoComplete="off"
          />
        </div>
      </div>
    </div>
  )
}
