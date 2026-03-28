// Preview tab content: canvas controls, annotation toggle, open tabs list.

import { Layers } from 'lucide-react'
import type { ViewMode } from './types'

interface PreviewTabContentProps {
  openTabs: string[]
  activeTab: string | null
  annotationMode: boolean
  viewMode: ViewMode
  onSelectTab: (path: string) => void
  onToggleAnnotationMode: () => void
  onSetViewMode: (mode: ViewMode) => void
}

export function PreviewTabContent({
  openTabs, activeTab, annotationMode, viewMode,
  onSelectTab, onToggleAnnotationMode, onSetViewMode,
}: PreviewTabContentProps) {
  return (
    <div className="space-y-3">
      {viewMode !== 'canvas' && (
        <button
          onClick={() => onSetViewMode('canvas')}
          className="flex items-center gap-1.5 w-full px-2 py-1.5 text-[11px] font-medium text-primary hover:bg-primary/10 rounded-md transition"
        >
          <Layers className="size-3" />
          Switch to canvas view
        </button>
      )}
      <button
        onClick={onToggleAnnotationMode}
        className={`flex items-center gap-1.5 w-full px-2 py-1.5 text-[11px] font-medium rounded-md transition ${
          annotationMode ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:bg-muted/50 hover:text-foreground'
        }`}
      >
        {annotationMode ? 'Annotating...' : 'Toggle annotations'}
      </button>
      <div>
        <p className="text-[9px] font-semibold text-muted-foreground uppercase tracking-wide px-1 mb-1">Open tabs</p>
        {openTabs.length === 0 ? (
          <p className="text-[10px] text-muted-foreground px-1">No canvas tabs open</p>
        ) : (
          <div className="space-y-px">
            {openTabs.map((path) => {
              const name = path.split('/').pop() ?? path
              return (
                <button
                  key={path}
                  onClick={() => onSelectTab(path)}
                  className={`w-full flex items-center gap-1.5 rounded px-1.5 py-1 text-left transition text-[10px] ${
                    activeTab === path ? 'bg-primary/10 text-foreground font-medium' : 'text-muted-foreground hover:bg-muted/50'
                  }`}
                >
                  <Layers className="size-2.5 shrink-0" />
                  <span className="truncate">{name}</span>
                </button>
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}
