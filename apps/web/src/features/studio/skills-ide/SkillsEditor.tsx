import { useRef, useCallback, useMemo } from 'react'
import { X, PanelRight, Zap, Save, WifiOff, Terminal, Plus, FileQuestion } from 'lucide-react'
import type { Skill } from '@ship/ui'
import { parseTabId } from './useSkillsIDE'
import { highlightLines, renderInlineCode } from './editor-highlight'

interface Props {
  skills: Skill[]
  openTabIds: string[]
  activeTabId: string | null
  unsavedIds: Set<string>
  content: string
  isConnected: boolean
  isLoading: boolean
  previewOpen?: boolean
  onTabSelect: (id: string) => void
  onTabClose: (id: string) => void
  onContentChange: (id: string, content: string) => void
  onSave: (id: string) => void
  onTogglePreview?: () => void
  onCreateSkill?: () => void
}

function EditorSkeleton() {
  return (
    <div className="flex flex-1 flex-col min-w-0">
      <div className="flex items-center gap-1 border-b border-border px-2 py-1.5 shrink-0">
        {Array.from({ length: 2 }).map((_, i) => (
          <div key={i} className="h-6 w-20 animate-pulse rounded bg-muted" />
        ))}
      </div>
      <div className="flex-1 p-4 space-y-2">
        {Array.from({ length: 10 }).map((_, i) => (
          <div key={i} className="h-4 animate-pulse rounded bg-muted" style={{ width: `${40 + (i * 7) % 50}%` }} />
        ))}
      </div>
    </div>
  )
}

export function SkillsEditor({
  skills, openTabIds, activeTabId, unsavedIds, content,
  isConnected, isLoading, previewOpen,
  onTabSelect, onTabClose, onContentChange, onSave, onTogglePreview, onCreateSkill,
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

  if (isLoading && skills.length === 0) return <EditorSkeleton />

  // No skills loaded + disconnected
  if (!isConnected && skills.length === 0 && !activeTabId) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center text-muted-foreground min-w-0 px-6">
        <Terminal className="size-10 opacity-50" />
        <div>
          <p className="text-sm font-medium text-foreground">Connect to Ship CLI to manage your skills</p>
          <p className="mt-2 text-xs text-muted-foreground">Start the CLI server, then Studio will sync automatically.</p>
          <code className="mt-3 inline-block rounded-md border border-border bg-muted/50 px-3 py-1.5 text-xs font-mono text-emerald-600 dark:text-emerald-400">
            ship mcp serve --http --port 51741
          </code>
        </div>
      </div>
    )
  }

  // No skills loaded + connected
  if (isConnected && skills.length === 0 && !activeTabId) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center text-muted-foreground min-w-0 px-6">
        <Zap className="size-10 opacity-50" />
        <div>
          <p className="text-sm font-medium text-foreground">No skills found in this project</p>
          <p className="mt-1 text-xs text-muted-foreground">Create one to get started.</p>
          {onCreateSkill && (
            <button
              onClick={onCreateSkill}
              className="mt-3 inline-flex items-center gap-1.5 rounded-md border border-border bg-muted/50 px-3 py-1.5 text-xs font-medium text-foreground hover:bg-muted transition-colors"
            >
              <Plus className="size-3.5" />
              Create skill
            </button>
          )}
        </div>
      </div>
    )
  }

  // No active tab or stale tab
  if (!activeTabId || !activeSkill) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center text-muted-foreground min-w-0">
        <Zap className="size-10 opacity-50" />
        <div>
          <p className="text-sm font-medium text-foreground">No file open</p>
          <p className="mt-1 text-xs text-muted-foreground">Select a skill from the explorer to start editing.</p>
        </div>
      </div>
    )
  }

  const activeFilePath = activeTab?.filePath ?? 'SKILL.md'
  const breadcrumbParts = ['skills', activeSkill.name || activeSkill.id, activeFilePath]
  const fileIsEmpty = content.trim() === ''
  const showDisconnectBanner = !isConnected && unsavedIds.size > 0

  return (
    <div className="flex flex-1 flex-col min-w-0">
      {showDisconnectBanner && (
        <div className="flex items-center gap-2 px-4 py-1 border-b border-amber-500/30 bg-amber-500/10 text-[11px] text-amber-600 dark:text-amber-400 shrink-0">
          <WifiOff className="size-3 shrink-0" />
          CLI disconnected — changes are saved locally
        </div>
      )}

      {/* Breadcrumb toolbar */}
      <div className="flex items-center justify-between px-4 py-1.5 border-b border-border bg-background/50 shrink-0">
        <div className="text-[11px] text-muted-foreground flex items-center gap-0.5">
          {breadcrumbParts.map((part, i, arr) => (
            <span key={i} className="flex items-center">
              {i > 0 && <span className="mx-1 text-muted-foreground">/</span>}
              <span className={i === arr.length - 1 ? 'text-foreground/80 font-medium' : ''}>{part}</span>
            </span>
          ))}
        </div>
        <div className="flex items-center gap-1">
          {unsavedIds.has(activeTabId) && (
            <button onClick={() => onSave(activeTabId)} className="flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium bg-primary/15 text-primary hover:bg-primary/25 transition-colors" title="Save (Cmd+S)">
              <Save className="size-3" />
              <span>Save</span>
            </button>
          )}
          <button
            onClick={onTogglePreview}
            className={`p-1 rounded transition-colors ${previewOpen ? 'text-primary bg-primary/10' : 'text-muted-foreground hover:text-foreground hover:bg-muted/50'}`}
            title={previewOpen ? 'Hide detail panel' : 'Show detail panel'}
          >
            <PanelRight className="size-4" />
          </button>
        </div>
      </div>

      {/* File tabs */}
      <div className="flex items-center border-b border-border bg-card/20 px-2 h-8 shrink-0 overflow-x-auto">
        {openTabIds.map((tabId) => {
          const { skillId, filePath } = parseTabId(tabId)
          const skill = skills.find((s) => s.id === skillId)
          if (!skill) return null
          const isActive = tabId === activeTabId
          const isUnsaved = unsavedIds.has(tabId)
          const fileName = filePath.split('/').pop() ?? filePath
          return (
            <button
              key={tabId}
              onClick={() => onTabSelect(tabId)}
              className={`group flex items-center gap-1.5 px-3 py-1 text-xs whitespace-nowrap border-b-2 transition-colors ${
                isActive ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground hover:text-foreground'
              }`}
            >
              {isUnsaved && <span className="size-1.5 rounded-full bg-primary shrink-0" />}
              <span className="truncate max-w-[140px]">{fileName}</span>
              <span onClick={(e) => { e.stopPropagation(); onTabClose(tabId) }} className="ml-1 text-muted-foreground hover:text-destructive transition-colors">
                <X className="size-3" />
              </span>
            </button>
          )
        })}
      </div>

      {/* Editor area */}
      {fileIsEmpty ? (
        <div className="relative flex flex-1 flex-col items-center justify-center gap-2 text-center min-h-0">
          <FileQuestion className="size-8 text-muted-foreground opacity-50" />
          <p className="text-xs text-muted-foreground">This file is empty</p>
          <p className="text-[11px] text-muted-foreground">Start typing to add content.</p>
          <textarea
            ref={textareaRef}
            value={content}
            onChange={(e) => onContentChange(activeTabId, e.target.value)}
            className="absolute inset-0 w-full h-full opacity-0 cursor-text"
            autoFocus
          />
        </div>
      ) : (
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
                onChange={(e) => onContentChange(activeTabId, e.target.value)}
                className="absolute inset-0 w-full h-full px-4 pt-4 pb-4 font-mono text-xs leading-[1.7] text-transparent caret-foreground bg-transparent resize-none focus:outline-none whitespace-pre-wrap break-words selection:bg-primary/25 selection:text-transparent"
                spellCheck={false}
                autoComplete="off"
              />
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
