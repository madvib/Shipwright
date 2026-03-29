import { useCallback, lazy, Suspense } from 'react'
import { X, PanelRight, Zap, Save, WifiOff, Terminal, Plus } from 'lucide-react'
import type { Skill } from '@ship/ui'
import type { FrontmatterEntry } from '@ship/primitives'
import { parseTabId } from './useSkillsIDE'
import { TextEditor } from './TextEditor'

// Lazy-load tiptap to avoid SSR/worker crashes (uses browser DOM APIs)
const MarkdownEditor = lazy(() =>
  import('@ship/primitives').then((m) => ({ default: m.MarkdownEditor }))
)
import { JsonViewer } from './JsonViewer'
import { ScriptViewer } from './ScriptViewer'
import { EditorSkeleton, EmptyState } from './EditorStates'

type ScriptLang = 'sh' | 'py' | 'js' | 'ts'

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
  onComment?: (selectedText: string, comment: string, skillName: string, tabId: string) => void
  onFrontmatterParsed?: (entries: FrontmatterEntry[], raw: string | null) => void
}

function getFileType(path: string): 'markdown' | 'json' | 'script' | 'text' {
  if (path.endsWith('.md')) return 'markdown'
  if (path.endsWith('.json')) return 'json'
  if (/\.(sh|py|js|ts)$/.test(path)) return 'script'
  return 'text'
}

function getScriptLang(path: string): ScriptLang {
  if (path.endsWith('.py')) return 'py'
  if (path.endsWith('.sh')) return 'sh'
  if (path.endsWith('.ts')) return 'ts'
  return 'js'
}

export function SkillsEditor({
  skills, openTabIds, activeTabId, unsavedIds, content,
  isConnected, isLoading, previewOpen,
  onTabSelect, onTabClose, onContentChange, onSave, onTogglePreview, onCreateSkill,
  onComment: onCommentProp,
  onFrontmatterParsed: onFrontmatterParsedProp,
}: Props) {
  const activeTab = activeTabId ? parseTabId(activeTabId) : null
  const activeSkill = activeTab ? skills.find((s) => s.id === activeTab.skillId) : undefined

  // Highlight-to-comment: write feedback via MCP or log locally
  const handleComment = useCallback((selectedText: string, comment: string) => {
    const skillName = activeSkill?.name ?? activeSkill?.id ?? 'unknown'
    if (onCommentProp && activeTabId) {
      onCommentProp(selectedText, comment, skillName, activeTabId)
    }
  }, [activeTabId, activeSkill, onCommentProp])

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

  if (!isConnected && skills.length === 0 && !activeTabId) {
    return (
      <EmptyState
        icon={<Terminal className="size-10 opacity-50" />}
        title="Connect to Ship CLI to manage your skills"
        subtitle="Start the CLI server, then Studio will sync automatically."
      >
        <code className="mt-3 inline-block rounded-md border border-border bg-muted/50 px-3 py-1.5 text-xs font-mono text-emerald-600 dark:text-emerald-400">
          ship studio --port 51741
        </code>
      </EmptyState>
    )
  }

  if (isConnected && skills.length === 0 && !activeTabId) {
    return (
      <EmptyState
        icon={<Zap className="size-10 opacity-50" />}
        title="No skills found in this project"
        subtitle="Create one to get started."
      >
        {onCreateSkill && (
          <button
            onClick={onCreateSkill}
            className="mt-3 inline-flex items-center gap-1.5 rounded-md border border-border bg-muted/50 px-3 py-1.5 text-xs font-medium text-foreground hover:bg-muted transition-colors"
          >
            <Plus className="size-3.5" />
            Create skill
          </button>
        )}
      </EmptyState>
    )
  }

  if (!activeTabId || !activeSkill) {
    return (
      <EmptyState
        icon={<Zap className="size-10 opacity-50" />}
        title="No file open"
        subtitle="Select a skill from the explorer to start editing."
      />
    )
  }

  const activeFilePath = activeTab?.filePath ?? 'SKILL.md'
  const fileType = getFileType(activeFilePath)
  const breadcrumbParts = ['skills', activeSkill.name || activeSkill.id, ...activeFilePath.split('/')]
  const showDisconnectBanner = !isConnected && unsavedIds.size > 0

  return (
    <div className="flex flex-1 flex-col min-w-0" onKeyDown={handleKeyDown}>
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

      {/* Editor area — dispatched by file type */}
      <EditorBody
        tabId={activeTabId}
        content={content}
        fileType={fileType}
        filePath={activeFilePath}
        onContentChange={onContentChange}
        onSave={onSave}
        onComment={handleComment}
        onFrontmatterParsed={onFrontmatterParsedProp}
      />
    </div>
  )
}

function EditorBody({ tabId, content, fileType, filePath, onContentChange, onSave, onComment, onFrontmatterParsed }: {
  tabId: string; content: string; fileType: 'markdown' | 'json' | 'script' | 'text'
  filePath: string
  onContentChange: (id: string, content: string) => void; onSave: (id: string) => void
  onComment?: (selectedText: string, comment: string) => void
  onFrontmatterParsed?: (entries: FrontmatterEntry[], raw: string | null) => void
}) {
  // Markdown: tiptap handles edit/read mode internally
  if (fileType === 'markdown') {
    return (
      <div className="flex-1 min-h-0 overflow-hidden">
        <Suspense fallback={<div className="p-4 text-xs text-muted-foreground">Loading editor...</div>}>
          <MarkdownEditor
            value={content}
            onChange={(v) => onContentChange(tabId, v)}
            fillHeight
            hideChrome
            onComment={onComment}
            onFrontmatterParsed={onFrontmatterParsed}
          />
        </Suspense>
      </div>
    )
  }

  // JSON files: editable viewer
  if (fileType === 'json') {
    return <JsonViewer content={content} tabId={tabId} filePath={filePath} onContentChange={onContentChange} onSave={onSave} />
  }

  // Script files: read-only viewer
  if (fileType === 'script') {
    return <ScriptViewer content={content} language={getScriptLang(filePath)} />
  }

  // Default: editable text editor (plain text, etc.)
  return (
    <TextEditor
      tabId={tabId}
      content={content}
      onContentChange={onContentChange}
      onSave={onSave}
    />
  )
}
