import { useState, useCallback } from 'react'
import { X, PanelRight, Zap, Save, WifiOff, Terminal, Plus, Eye, Code2 } from 'lucide-react'
import type { Skill } from '@ship/ui'
import { parseTabId } from './useSkillsIDE'
import { TextEditor } from './TextEditor'
import { MarkdownPreview } from './MarkdownPreview'
import { JsonViewer } from './JsonViewer'
import { ScriptViewer } from './ScriptViewer'

type ViewMode = 'source' | 'preview'
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

function EmptyState({ icon, title, subtitle, children }: {
  icon: React.ReactNode; title: string; subtitle: string; children?: React.ReactNode
}) {
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-3 text-center text-muted-foreground min-w-0 px-6">
      {icon}
      <div>
        <p className="text-sm font-medium text-foreground">{title}</p>
        <p className="mt-1 text-xs text-muted-foreground">{subtitle}</p>
        {children}
      </div>
    </div>
  )
}

export function SkillsEditor({
  skills, openTabIds, activeTabId, unsavedIds, content,
  isConnected, isLoading, previewOpen,
  onTabSelect, onTabClose, onContentChange, onSave, onTogglePreview, onCreateSkill,
}: Props) {
  const [viewMode, setViewMode] = useState<ViewMode>('source')
  const activeTab = activeTabId ? parseTabId(activeTabId) : null
  const activeSkill = activeTab ? skills.find((s) => s.id === activeTab.skillId) : undefined

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
          ship mcp serve --http --port 51741
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
  const breadcrumbParts = ['skills', activeSkill.name || activeSkill.id, activeFilePath]
  const showDisconnectBanner = !isConnected && unsavedIds.size > 0
  const isMarkdown = fileType === 'markdown'

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
          {isMarkdown && (
            <div className="flex items-center rounded border border-border text-[11px] mr-1">
              <button
                onClick={() => setViewMode('source')}
                className={`flex items-center gap-1 px-2 py-0.5 rounded-l transition-colors ${viewMode === 'source' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground'}`}
              >
                <Code2 className="size-3" />
                Source
              </button>
              <button
                onClick={() => setViewMode('preview')}
                className={`flex items-center gap-1 px-2 py-0.5 rounded-r transition-colors ${viewMode === 'preview' ? 'bg-muted text-foreground' : 'text-muted-foreground hover:text-foreground'}`}
              >
                <Eye className="size-3" />
                Preview
              </button>
            </div>
          )}
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
        viewMode={viewMode}
        onContentChange={onContentChange}
        onSave={onSave}
      />
    </div>
  )
}

function EditorBody({ tabId, content, fileType, filePath, viewMode, onContentChange, onSave }: {
  tabId: string; content: string; fileType: 'markdown' | 'json' | 'script' | 'text'
  filePath: string; viewMode: ViewMode
  onContentChange: (id: string, content: string) => void; onSave: (id: string) => void
}) {
  // Markdown in preview mode
  if (fileType === 'markdown' && viewMode === 'preview') {
    return (
      <div className="flex-1 min-h-0 overflow-auto">
        <MarkdownPreview content={content} />
      </div>
    )
  }

  // JSON files: read-only viewer
  if (fileType === 'json') {
    return <JsonViewer content={content} />
  }

  // Script files: read-only viewer
  if (fileType === 'script') {
    return <ScriptViewer content={content} language={getScriptLang(filePath)} />
  }

  // Default: editable text editor (markdown source, plain text, etc.)
  return (
    <TextEditor
      tabId={tabId}
      content={content}
      onContentChange={onContentChange}
      onSave={onSave}
    />
  )
}
