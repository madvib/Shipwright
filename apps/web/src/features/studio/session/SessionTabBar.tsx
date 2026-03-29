import { X } from 'lucide-react'

interface OpenTab {
  path: string
  name: string
  type: string
}

interface SessionTabBarProps {
  openTabs: OpenTab[]
  activeTabPath: string | null
  viewMode: 'file' | 'diff'
  unsavedPaths: Set<string>
  selectedCommitHash: string | null
  onSelectTab: (path: string) => void
  onCloseTab: (path: string) => void
}

export function SessionTabBar({
  openTabs, activeTabPath, viewMode, unsavedPaths,
  selectedCommitHash, onSelectTab, onCloseTab,
}: SessionTabBarProps) {
  const showDiff = viewMode === 'diff'

  return (
    <div className="flex items-center border-b border-border bg-card/20 px-1 h-9 shrink-0 overflow-x-auto">
      {openTabs.map((tab) => {
        const isActive = viewMode === 'file' && tab.path === activeTabPath
        const unsaved = unsavedPaths.has(tab.path)
        return (
          <button
            key={tab.path}
            onClick={() => onSelectTab(tab.path)}
            className={`group flex items-center gap-1.5 px-3 py-1.5 text-xs whitespace-nowrap border-b-2 transition-colors ${
              isActive
                ? 'border-primary text-foreground font-medium'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            {unsaved && <span className="size-1.5 rounded-full bg-primary shrink-0" title="Unsaved" />}
            <span className="truncate max-w-[140px]">{tab.name}</span>
            <span
              onClick={(e) => { e.stopPropagation(); onCloseTab(tab.path) }}
              className="size-4 rounded flex items-center justify-center opacity-0 group-hover:opacity-100 hover:bg-destructive/10 hover:text-destructive transition"
            >
              <X className="size-3" />
            </span>
          </button>
        )
      })}

      {showDiff && (
        <button
          className="flex items-center gap-1.5 px-3 py-1.5 text-xs whitespace-nowrap border-b-2 border-primary text-foreground font-medium"
        >
          {selectedCommitHash ? `Diff ${selectedCommitHash.slice(0, 7)}` : 'Diff'}
        </button>
      )}

      {openTabs.length === 0 && !showDiff && (
        <span className="px-3 py-1.5 text-xs text-muted-foreground/40">No files open</span>
      )}
    </div>
  )
}
