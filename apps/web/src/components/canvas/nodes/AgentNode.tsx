import type { NodeProps } from '@xyflow/react'
import { Handle, Position } from '@xyflow/react'
import type { AgentNodeType } from '../types'
import { AGENT_STYLES } from '../types'

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
  const style = AGENT_STYLES[agentType]

  const borderStyle: React.CSSProperties = {
    border: `1.5px ${style.dashed ? 'dashed' : 'solid'} ${style.stroke}`,
    boxShadow: style.glow ? `0 0 12px ${style.stroke}33` : undefined,
  }

  return (
    <div
      style={{
        position: 'relative',
        width: 200,
        minHeight: 100,
        background: style.fill,
        borderRadius: 8,
        padding: '10px 12px',
        boxSizing: 'border-box',
        display: 'flex',
        flexDirection: 'column',
        gap: 6,
        ...borderStyle,
      }}
    >
      <Handle type="target" position={Position.Top} style={{ background: style.stroke }} />

      <div style={{ display: 'flex', alignItems: 'flex-start', gap: 8 }}>
        <div
          style={{
            width: 32,
            height: 32,
            borderRadius: 6,
            background: ICON_BG[agentType],
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: 18,
            flexShrink: 0,
          }}
        >
          {icon ?? DEFAULT_ICONS[agentType]}
        </div>
        <div style={{ display: 'flex', flexDirection: 'column', gap: 2, overflow: 'hidden' }}>
          <div style={{ fontSize: 12, color: '#e2e8f0', fontWeight: 600, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{name}</div>
          <div style={{ fontSize: 10, color: '#475569', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{profile}</div>
          {detail && <div style={{ fontSize: 10, color: '#334155', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{detail}</div>}
        </div>
      </div>

      {badge && (
        <div
          style={{
            alignSelf: 'flex-start',
            background: `${style.stroke}22`,
            color: style.stroke,
            fontSize: 9,
            fontWeight: 700,
            textTransform: 'uppercase',
            letterSpacing: '0.08em',
            padding: '2px 6px',
            borderRadius: 4,
          }}
        >
          {badge}
        </div>
      )}

      <Handle type="source" position={Position.Bottom} style={{ background: style.stroke }} />
    </div>
  )
}
