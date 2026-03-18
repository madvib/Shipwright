import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { PlatformNodeType } from '../types'

const FONT_BODY = '"DM Sans Variable", "DM Sans", sans-serif'
const FONT_MONO = 'ui-monospace, "Fira Code", monospace'

const variants = {
  mcp: {
    bg: '#091420',
    border: '#0ea5e928',
    accent: '#0ea5e9',
    accentMuted: '#7dd3fc',
    detailColor: '#0ea5e944',
    glow: '0 0 12px #0ea5e910',
    icon: '{}',
  },
  hook: {
    bg: '#140d1f',
    border: '#a855f728',
    accent: '#a855f7',
    accentMuted: '#d8b4fe',
    detailColor: '#a855f744',
    glow: '0 0 12px #a855f710',
    icon: 'fn',
  },
}

export function PlatformNode({ data }: NodeProps<PlatformNodeType>) {
  const v = variants[data.nodeKind]
  return (
    <div style={{
      background: v.bg, border: `1.5px solid ${v.border}`, borderRadius: 6,
      width: 200, minHeight: 64, padding: '8px 12px', boxSizing: 'border-box',
      boxShadow: v.glow, transition: 'box-shadow 0.2s',
    }}>
      <Handle type="target" position={Position.Left} />
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 3 }}>
        <span style={{
          fontFamily: FONT_MONO, fontSize: 8, color: v.accent,
          background: `${v.accent}12`, border: `1px solid ${v.accent}20`,
          borderRadius: 3, padding: '1px 4px', fontWeight: 700,
          letterSpacing: '0.04em',
        }}>
          {v.icon}
        </span>
        <span style={{
          fontFamily: FONT_BODY, fontSize: 9, color: v.accent,
          fontWeight: 600, textTransform: 'uppercase', letterSpacing: '0.08em',
        }}>
          {data.nodeKind === 'mcp' ? 'MCP Server' : 'Hook'}
        </span>
      </div>
      <div style={{
        fontFamily: FONT_BODY, fontSize: 11, color: v.accentMuted,
        fontWeight: 600, marginTop: 2,
      }}>
        {data.nodeKind === 'hook' ? `${data.title}` : data.title}
      </div>
      <div style={{
        fontFamily: FONT_MONO, fontSize: 9, color: v.detailColor, marginTop: 3,
        letterSpacing: '0.01em',
      }}>
        {data.detail}
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  )
}
