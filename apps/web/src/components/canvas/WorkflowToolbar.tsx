import { useState, useRef } from 'react'
import { ChevronLeft, Plus, Download } from 'lucide-react'

interface Props {
  name: string
  onBack: () => void
  onAddNode: (type: string, data: Record<string, unknown>) => void
}

const ADD_OPTIONS = [
  { label: 'Artifact D0 (Target)', type: 'artifact', data: { label: 'New Target', depth: 0 as const, status: 'planned' as const } },
  { label: 'Artifact D1 (Capability)', type: 'artifact', data: { label: 'New Capability', depth: 1 as const, status: 'planned' as const, accentColor: '#3b82f6' } },
  { label: 'Artifact D2 (Job)', type: 'artifact', data: { label: 'New Job', depth: 2 as const, status: 'planned' as const } },
  { label: 'Agent', type: 'agent', data: { name: 'New Agent', profile: 'default', agentType: 'specialist' as const, icon: '🤖' } },
  { label: 'MCP Server', type: 'platform', data: { nodeKind: 'mcp' as const, title: 'mcp-server', detail: 'tools: 0' } },
  { label: 'Hook', type: 'platform', data: { nodeKind: 'hook' as const, title: 'PostToolUse', detail: 'command' } },
]

export function WorkflowToolbar({ name, onBack, onAddNode }: Props) {
  const [showMenu, setShowMenu] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)

  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 12,
      height: 44,
      padding: '0 12px',
      background: '#0d0d14',
      borderBottom: '1px solid #1e2030',
      flexShrink: 0,
    }}>
      {/* Back */}
      <button
        onClick={onBack}
        style={{
          display: 'flex', alignItems: 'center', gap: 4,
          background: 'none', border: 'none', color: '#94a3b8',
          fontSize: 12, cursor: 'pointer', padding: '4px 8px', borderRadius: 4,
        }}
        onMouseEnter={(e) => { e.currentTarget.style.color = '#e2e8f0' }}
        onMouseLeave={(e) => { e.currentTarget.style.color = '#94a3b8' }}
      >
        <ChevronLeft size={14} />
        Workflows
      </button>

      {/* Separator */}
      <div style={{ width: 1, height: 20, background: '#1e2030' }} />

      {/* Name */}
      <span style={{ fontSize: 13, fontWeight: 600, color: '#e2e8f0' }}>{name}</span>

      {/* Spacer */}
      <div style={{ flex: 1 }} />

      {/* Add node dropdown */}
      <div style={{ position: 'relative' }} ref={menuRef}>
        <button
          onClick={() => setShowMenu(!showMenu)}
          style={{
            display: 'flex', alignItems: 'center', gap: 4,
            background: '#1a1a2e', border: '1px solid #2a2a3e', borderRadius: 6,
            color: '#94a3b8', fontSize: 11, fontWeight: 600, cursor: 'pointer',
            padding: '5px 10px',
          }}
        >
          <Plus size={12} />
          Add node
        </button>
        {showMenu && (
          <div style={{
            position: 'absolute', top: '100%', right: 0, marginTop: 4,
            background: '#0d0d14', border: '1px solid #1e2030', borderRadius: 8,
            padding: 4, minWidth: 180, zIndex: 50,
            boxShadow: '0 8px 24px rgba(0,0,0,0.5)',
          }}>
            {ADD_OPTIONS.map((opt) => (
              <button
                key={opt.label}
                onClick={() => { onAddNode(opt.type, opt.data); setShowMenu(false) }}
                style={{
                  display: 'block', width: '100%', textAlign: 'left',
                  background: 'none', border: 'none', color: '#94a3b8',
                  fontSize: 11, padding: '6px 10px', borderRadius: 4,
                  cursor: 'pointer',
                }}
                onMouseEnter={(e) => { e.currentTarget.style.background = '#1a1a2e'; e.currentTarget.style.color = '#e2e8f0' }}
                onMouseLeave={(e) => { e.currentTarget.style.background = 'none'; e.currentTarget.style.color = '#94a3b8' }}
              >
                {opt.label}
              </button>
            ))}
          </div>
        )}
      </div>

      {/* Export (placeholder) */}
      <button
        style={{
          display: 'flex', alignItems: 'center', gap: 4,
          background: '#1a1a2e', border: '1px solid #2a2a3e', borderRadius: 6,
          color: '#475569', fontSize: 11, fontWeight: 600, cursor: 'not-allowed',
          padding: '5px 10px', opacity: 0.5,
        }}
        disabled
      >
        <Download size={12} />
        Export
      </button>
    </div>
  )
}
