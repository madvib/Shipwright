import { useState, useRef, useEffect } from 'react'
import { ChevronLeft, Plus, Download } from 'lucide-react'

const FONT_DISPLAY = '"Syne Variable", "Syne", sans-serif'
const FONT_BODY = '"DM Sans Variable", "DM Sans", sans-serif'

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

  // Close menu on outside click
  useEffect(() => {
    if (!showMenu) return
    const handler = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) setShowMenu(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [showMenu])

  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: 12,
      height: 44,
      padding: '0 14px',
      background: '#0d0d14e8',
      backdropFilter: 'blur(12px)',
      borderBottom: '1px solid #1e2030',
      flexShrink: 0,
    }}>
      {/* Back */}
      <button
        onClick={onBack}
        style={{
          display: 'flex', alignItems: 'center', gap: 4,
          background: 'none', border: 'none',
          fontFamily: FONT_BODY, color: '#64748b',
          fontSize: 12, cursor: 'pointer', padding: '4px 8px', borderRadius: 4,
          transition: 'color 0.15s',
          fontWeight: 500,
        }}
        onMouseEnter={(e) => { e.currentTarget.style.color = '#e2e8f0' }}
        onMouseLeave={(e) => { e.currentTarget.style.color = '#64748b' }}
      >
        <ChevronLeft size={14} />
        Workflows
      </button>

      {/* Separator */}
      <div style={{ width: 1, height: 18, background: '#1e2030' }} />

      {/* Name */}
      <span style={{
        fontFamily: FONT_DISPLAY, fontSize: 13, fontWeight: 700,
        color: '#e2e8f0', letterSpacing: '-0.01em',
      }}>
        {name}
      </span>

      <div style={{ flex: 1 }} />

      {/* Add node dropdown */}
      <div style={{ position: 'relative' }} ref={menuRef}>
        <button
          onClick={() => setShowMenu(!showMenu)}
          style={{
            display: 'flex', alignItems: 'center', gap: 5,
            background: showMenu ? '#1a1a2e' : 'transparent',
            border: '1px solid #1e2030', borderRadius: 6,
            fontFamily: FONT_BODY, color: '#94a3b8',
            fontSize: 11, fontWeight: 600, cursor: 'pointer',
            padding: '5px 10px',
            transition: 'all 0.15s',
          }}
          onMouseEnter={(e) => { if (!showMenu) e.currentTarget.style.background = '#12121e' }}
          onMouseLeave={(e) => { if (!showMenu) e.currentTarget.style.background = 'transparent' }}
        >
          <Plus size={12} />
          Add node
        </button>
        {showMenu && (
          <div style={{
            position: 'absolute', top: '100%', right: 0, marginTop: 6,
            background: '#0d0d14f0', backdropFilter: 'blur(16px)',
            border: '1px solid #1e2030', borderRadius: 8,
            padding: 4, minWidth: 200, zIndex: 50,
            boxShadow: '0 12px 40px rgba(0,0,0,0.6), 0 0 0 1px rgba(255,255,255,0.03)',
          }}>
            {ADD_OPTIONS.map((opt) => (
              <button
                key={opt.label}
                onClick={() => { onAddNode(opt.type, opt.data); setShowMenu(false) }}
                style={{
                  display: 'block', width: '100%', textAlign: 'left',
                  background: 'none', border: 'none',
                  fontFamily: FONT_BODY, color: '#94a3b8',
                  fontSize: 11, padding: '7px 10px', borderRadius: 4,
                  cursor: 'pointer', fontWeight: 500,
                  transition: 'all 0.1s',
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
          background: 'transparent', border: '1px solid #1e2030', borderRadius: 6,
          fontFamily: FONT_BODY, color: '#334155',
          fontSize: 11, fontWeight: 600, cursor: 'not-allowed',
          padding: '5px 10px', opacity: 0.4,
        }}
        disabled
      >
        <Download size={12} />
        Export
      </button>
    </div>
  )
}
