import { useState, useRef, useEffect } from 'react'
import { ChevronLeft, Plus, Download } from 'lucide-react'

interface Props {
  name: string
  onBack: () => void
  onAddNode: (type: string, data: Record<string, unknown>) => void
}

const ADD_OPTIONS = [
  { label: 'Target (D0)', type: 'artifact', data: { label: 'New Target', depth: 0 as const, status: 'planned' as const } },
  { label: 'Capability (D1)', type: 'artifact', data: { label: 'New Capability', depth: 1 as const, status: 'planned' as const, accentColor: '#3b82f6' } },
  { label: 'Job (D2)', type: 'artifact', data: { label: 'New Job', depth: 2 as const, status: 'planned' as const } },
  { label: 'Agent', type: 'agent', data: { name: 'New Agent', profile: 'default', agentType: 'specialist' as const, icon: '🤖' } },
  { label: 'MCP Server', type: 'platform', data: { nodeKind: 'mcp' as const, title: 'mcp-server', detail: 'tools: 0' } },
  { label: 'Hook', type: 'platform', data: { nodeKind: 'hook' as const, title: 'PostToolUse', detail: 'command' } },
]

export function WorkflowToolbar({ name, onBack, onAddNode }: Props) {
  const [showMenu, setShowMenu] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!showMenu) return
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) setShowMenu(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [showMenu])

  return (
    <div className="flex items-center gap-3 h-11 px-3.5 border-b border-border/60 bg-background/90 backdrop-blur-md shrink-0">
      <button
        onClick={onBack}
        className="flex items-center gap-1 text-xs text-muted-foreground hover:text-foreground transition-colors font-medium px-2 py-1 rounded"
      >
        <ChevronLeft size={14} />
        Workflows
      </button>

      <div className="w-px h-4.5 bg-border/60" />

      <span className="text-sm font-bold text-foreground tracking-tight">{name}</span>

      <div className="flex-1" />

      {/* Add node dropdown */}
      <div className="relative" ref={menuRef}>
        <button
          onClick={() => setShowMenu(!showMenu)}
          className={`flex items-center gap-1.5 text-[11px] font-semibold px-2.5 py-1.5 rounded-md border transition-colors ${
            showMenu
              ? 'bg-muted border-border text-foreground'
              : 'bg-transparent border-border/60 text-muted-foreground hover:bg-muted/50 hover:text-foreground'
          }`}
        >
          <Plus size={12} />
          Add node
        </button>
        {showMenu && (
          <div className="absolute top-full right-0 mt-1.5 bg-popover/95 backdrop-blur-xl border border-border rounded-lg p-1 min-w-[200px] z-50 shadow-lg">
            {ADD_OPTIONS.map((opt) => (
              <button
                key={opt.label}
                onClick={() => { onAddNode(opt.type, opt.data); setShowMenu(false) }}
                className="block w-full text-left text-[11px] font-medium text-muted-foreground hover:text-foreground hover:bg-muted rounded px-2.5 py-1.5 transition-colors"
              >
                {opt.label}
              </button>
            ))}
          </div>
        )}
      </div>

      <button
        className="flex items-center gap-1 text-[11px] font-semibold px-2.5 py-1.5 rounded-md border border-border/60 text-muted-foreground/40 cursor-not-allowed opacity-40"
        disabled
      >
        <Download size={12} />
        Export
      </button>
    </div>
  )
}
