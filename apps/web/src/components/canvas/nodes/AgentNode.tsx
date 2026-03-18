import type { NodeProps } from '@xyflow/react'
import { Handle, Position } from '@xyflow/react'
import type { AgentNodeType } from '../types'
import { AGENT_STYLES } from '../types'

const FONT_DISPLAY = '"Syne Variable", "Syne", sans-serif'
const FONT_BODY = '"DM Sans Variable", "DM Sans", sans-serif'

const DEFAULT_ICONS: Record<string, string> = {
  human: '👤',
  commander: '⚡',
  specialist: '🌐',
  gate: '🔬',
}

const ICON_BG: Record<string, string> = {
  human: '#1e1030',
  commander: '#1a1200',
  specialist: '#0f1a2e',
  gate: '#0a1628',
}

export function AgentNode(props: NodeProps<AgentNodeType>) {
  const { data } = props
  const { name, profile, agentType, badge, icon, detail } = data
  const s = AGENT_STYLES[agentType]

  return (
    <div
      style={{
        position: 'relative',
        width: 200,
        minHeight: 100,
        background: s.fill,
        borderRadius: 8,
        padding: '10px 12px',
        boxSizing: 'border-box',
        display: 'flex',
        flexDirection: 'column',
        gap: 6,
        border: `1.5px ${s.dashed ? 'dashed' : 'solid'} ${s.stroke}`,
        boxShadow: s.glow
          ? `0 0 16px ${s.stroke}30, 0 0 4px ${s.stroke}18, inset 0 1px 0 ${s.stroke}0a`
          : `0 1px 6px rgba(0,0,0,0.15)`,
        transition: 'box-shadow 0.2s',
      }}
    >
      <Handle type="target" position={Position.Top} style={{ background: s.stroke }} />

      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 8 }}>
        <div
          style={{
            width: 32, height: 32, borderRadius: 6,
            background: ICON_BG[agentType],
            border: `1px solid ${s.stroke}18`,
            display: 'flex', alignItems: 'center', justifyContent: 'center',
            fontSize: 16, flexShrink: 0,
          }}
        >
          {icon ?? DEFAULT_ICONS[agentType]}
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 2, overflow: 'hidden' }}>
          <div style={{
            fontFamily: FONT_DISPLAY, fontSize: 12, color: '#e2e8f0',
            fontWeight: 700, letterSpacing: '-0.005em',
            whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
          }}>
            {name}
          </div>
          <div style={{
            fontFamily: FONT_BODY, fontSize: 10, color: '#475569',
            whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
            fontWeight: 500,
          }}>
            {profile}
          </div>
          {detail && (
            <div style={{
              fontFamily: FONT_BODY, fontSize: 9, color: '#334155',
              whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis',
              letterSpacing: '0.01em',
            }}>
              {detail}
            </div>
          )}
        </div>
      </div>

      {badge && (
        <div
          style={{
            alignSelf: 'flex-start',
            background: `${s.stroke}15`,
            color: s.stroke,
            fontFamily: FONT_BODY,
            fontSize: 8,
            fontWeight: 700,
            textTransform: 'uppercase',
            letterSpacing: '0.1em',
            padding: '2px 7px',
            borderRadius: 4,
            border: `1px solid ${s.stroke}20`,
          }}
        >
          {badge}
        </div>
      )}

      <Handle type="source" position={Position.Bottom} style={{ background: s.stroke }} />
    </div>
  )
}
