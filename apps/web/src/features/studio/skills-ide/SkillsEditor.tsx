import { useRef, useCallback, useMemo } from 'react'
import { X, Eye, Zap } from 'lucide-react'
import type { Skill } from '@ship/ui'
import { parseTabId } from './useSkillsIDE'

interface Props {
  skills: Skill[]
  openTabIds: string[]
  activeTabId: string | null
  unsavedIds: Set<string>
  content: string
  previewOpen?: boolean
  onTabSelect: (id: string) => void
  onTabClose: (id: string) => void
  onContentChange: (id: string, content: string) => void
  onSave: (id: string) => void
  onTogglePreview?: () => void
}

/** Split content into lines and apply simple syntax classes. */
function highlightLines(content: string) {
  const lines = content.split('\n')
  let inFrontmatter = false
  let fenceCount = 0

  return lines.map((line) => {
    if (line.trim() === '---') {
      fenceCount++
      inFrontmatter = fenceCount === 1
      return { text: line, className: 'text-muted-foreground/50' }
    }

    if (inFrontmatter && fenceCount === 1) {
      const colonIdx = line.indexOf(':')
      if (colonIdx > 0) {
        const key = line.slice(0, colonIdx)
        const val = line.slice(colonIdx + 1)
        return {
          text: line,
          className: '',
          fragments: [
            { text: key, className: 'text-sky-600 dark:text-sky-300' },
            { text: ':', className: 'text-muted-foreground/50' },
            { text: val, className: 'text-emerald-600 dark:text-emerald-300' },
          ],
        }
      }
      return { text: line, className: 'text-emerald-600 dark:text-emerald-300' }
    }

    if (line.startsWith('# ')) return { text: line, className: 'text-foreground font-bold text-sm' }
    if (line.startsWith('## ')) return { text: line, className: 'text-foreground/90 font-semibold' }
    if (line.startsWith('### ')) return { text: line, className: 'text-muted-foreground font-semibold' }
    if (line.startsWith('- ') || line.startsWith('* ')) return { text: line, className: 'text-muted-foreground/60' }
    if (/^\d+\.\s/.test(line)) return { text: line, className: 'text-muted-foreground/60' }
    if (line.trim() === '') return { text: ' ', className: '' }
    return { text: line, className: 'text-muted-foreground/70' }
  })
}

/** Render inline code spans within a text line. */
function renderInlineCode(text: string) {
  const parts = text.split(/(`[^`]+`)/)
  if (parts.length === 1) return text
  return parts.map((part, i) => {
    if (part.startsWith('`') && part.endsWith('`')) {
      return (
        <span key={i} className="bg-muted/60 px-1 rounded text-primary text-[11px]">
          {part.slice(1, -1)}
        </span>
      )
    }
    return part
  })
}

export function SkillsEditor({
  skills,
  openTabIds,
  activeTabId,
  unsavedIds,
  content,
  previewOpen,
  onTabSelect,
  onTabClose,
  onContentChange,
  onSave,
  onTogglePreview,
}: Props) {
  const textareaRef = useRef<HTMLTextAreaElement>(null)

  const activeTab = activeTabId ? parseTabId(activeTabId) : null
  const activeSkill = activeTab ? skills.find((s) => s.id === activeTab.skillId) : undefined
  const lines = useMemo(() => highlightLines(content), [content])

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 's') {
        e.preventDefault()
        if (activeTabId) onSave(activeTabId)
      }
    },
    [activeTabId, onSave],
  )

  if (!activeTabId || !activeSkill) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center text-muted-foreground/50 min-w-0">
        <Zap className="size-10 opacity-30" />
        <div>
          <p className="text-sm font-medium text-muted-foreground/60">No file open</p>
          <p className="mt-1 text-xs text-muted-foreground/50">
            Select a skill from the explorer to start editing.
          </p>
        </div>
      </div>
    )
  }

  const activeFilePath = activeTab?.filePath ?? 'SKILL.md'
  const breadcrumb = `skills / ${activeSkill.name || activeSkill.id} / ${activeFilePath}`

  return (
    <div className="flex flex-1 flex-col min-w-0">
      {/* Tab bar */}
      <div className="flex items-center border-b border-border/30 bg-card/20 px-2 h-8 shrink-0 overflow-x-auto">
        {openTabIds.map((tabId) => {
          const { skillId, filePath } = parseTabId(tabId)
          const skill = skills.find((s) => s.id === skillId)
          if (!skill) return null
          const isActive = tabId === activeTabId
          const isUnsaved = unsavedIds.has(tabId)
          return (
            <button
              key={tabId}
              onClick={() => onTabSelect(tabId)}
              className={`group flex items-center gap-1.5 px-3 py-1 text-xs whitespace-nowrap border-b-2 transition-colors ${
                isActive
                  ? 'border-primary text-foreground'
                  : 'border-transparent text-muted-foreground/60 hover:text-muted-foreground'
              }`}
            >
              {isUnsaved && (
                <span className="size-1.5 rounded-full bg-primary shrink-0" />
              )}
              <span className="truncate max-w-[140px]">
                {skill.name || skill.id}/{filePath}
              </span>
              <span
                onClick={(e) => { e.stopPropagation(); onTabClose(tabId) }}
                className="ml-1 text-muted-foreground/50 hover:text-destructive transition-colors"
              >
                <X className="size-3" />
              </span>
            </button>
          )
        })}
      </div>

      {/* Toolbar */}
      <div className="flex items-center justify-between px-4 py-1.5 border-b border-border/20 bg-background/50 shrink-0">
        <div className="text-[11px] text-muted-foreground/50 flex items-center gap-1">
          {breadcrumb.split(' / ').map((part, i, arr) => (
            <span key={i}>
              {i > 0 && <span className="mx-1 text-muted-foreground/40">/</span>}
              <span className={i === arr.length - 1 ? 'text-muted-foreground/60' : ''}>
                {part}
              </span>
            </span>
          ))}
        </div>
        <button
          onClick={onTogglePreview}
          className={`p-1 transition-colors ${previewOpen ? 'text-primary' : 'text-muted-foreground/50 hover:text-muted-foreground'}`}
          title="Toggle skill info"
        >
          <Eye className="size-3.5" />
        </button>
      </div>

      {/* Editor area with line numbers */}
      <div className="flex flex-1 min-h-0 overflow-auto" onKeyDown={handleKeyDown}>
        {/* Line numbers */}
        <div className="shrink-0 w-10 pt-4 pb-4 text-right pr-2 font-mono text-[11px] leading-[1.7] text-muted-foreground/40 select-none border-r border-border/10">
          {lines.map((_, i) => (
            <div key={i}>{i + 1}</div>
          ))}
        </div>

        {/* Content overlay + textarea */}
        <div className="flex-1 relative min-w-0">
          {/* Syntax-highlighted overlay */}
          <div
            className="absolute inset-0 px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] pointer-events-none whitespace-pre-wrap break-words overflow-hidden"
            aria-hidden="true"
          >
            {lines.map((line, i) => (
              <div key={i} className={line.className}>
                {'fragments' in line && line.fragments ? (
                  line.fragments.map((f, fi) => (
                    <span key={fi} className={f.className}>
                      {renderInlineCode(f.text)}
                    </span>
                  ))
                ) : (
                  renderInlineCode(line.text)
                )}
              </div>
            ))}
          </div>

          {/* Transparent textarea for editing */}
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => onContentChange(activeTabId, e.target.value)}
            className="absolute inset-0 w-full h-full px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] text-transparent caret-foreground bg-transparent resize-none focus:outline-none whitespace-pre-wrap break-words"
            spellCheck={false}
            autoComplete="off"
          />
        </div>
      </div>
    </div>
  )
}
