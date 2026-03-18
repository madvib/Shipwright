import type { NodeProps } from '@xyflow/react'
import { Handle, Position } from '@xyflow/react'
import type { ArtifactNodeType } from '../types'
import { STATUS_COLORS } from '../types'

export function ArtifactNode(props: NodeProps<ArtifactNodeType>) {
  const { data } = props
  const { label, depth, accentColor = '#3b82f6', status, subtitle } = data
  const statusColor = status ? STATUS_COLORS[status] : undefined

  if (depth === 0) {
    return (
      <div style={{ position: 'relative', width: 280, height: 90, background: '#0f1520', border: '1.5px solid #334155', borderRadius: 3, padding: '12px 14px', boxSizing: 'border-box' }}>
        <Handle type="target" position={Position.Top} style={{ background: '#334155' }} />
        <svg style={{ position: 'absolute', top: 0, right: 0 }} width={18} height={18} viewBox="0 0 18 18">
          <polygon points="0,0 18,0 18,18" fill="#1e2d3d" />
        </svg>
        <div style={{ fontSize: 10, color: '#94a3b8', textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 4 }}>Target</div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 6 }}>
          {statusColor && <span style={{ width: 7, height: 7, borderRadius: '50%', background: statusColor, flexShrink: 0 }} />}
          <div style={{ fontSize: 14, color: '#f1f5f9', fontWeight: 600, lineHeight: 1.3 }}>{label}</div>
        </div>
        {subtitle && <div style={{ fontSize: 11, color: '#475569', marginTop: 4 }}>{subtitle}</div>}
        <Handle type="source" position={Position.Bottom} style={{ background: '#334155' }} />
      </div>
    )
  }

  if (depth === 1) {
    return (
      <div style={{ position: 'relative', width: 220, height: 68, background: '#0d1117', border: '1.5px solid #1e2535', borderRadius: 6, boxSizing: 'border-box', overflow: 'hidden' }}>
        <Handle type="target" position={Position.Top} style={{ background: '#1e2535' }} />
        <div style={{ position: 'absolute', left: 0, top: 0, bottom: 0, width: 3, background: accentColor }} />
        <div style={{ padding: '10px 12px 10px 15px' }}>
          <div style={{ fontSize: 10, color: accentColor, textTransform: 'uppercase', letterSpacing: '0.08em', marginBottom: 3 }}>Capability</div>
          <div style={{ fontSize: 12, color: '#e2e8f0', fontWeight: 500, lineHeight: 1.3 }}>{label}</div>
          {(subtitle || status) && (
            <div style={{ display: 'flex', alignItems: 'center', gap: 5, marginTop: 3 }}>
              {statusColor && <span style={{ width: 6, height: 6, borderRadius: '50%', background: statusColor, flexShrink: 0 }} />}
              {status && <span style={{ fontSize: 10, color: '#475569' }}>{status}</span>}
              {subtitle && !status && <span style={{ fontSize: 10, color: '#475569' }}>{subtitle}</span>}
            </div>
          )}
        </div>
        <Handle type="source" position={Position.Bottom} style={{ background: '#1e2535' }} />
      </div>
    )
  }

  // depth === 2
  const dotColor = statusColor ?? '#475569'
  return (
    <div style={{ position: 'relative', width: 180, height: 28, background: '#111827', border: '1px solid #1f2937', borderRadius: 5, display: 'flex', alignItems: 'center', gap: 6, padding: '0 8px', boxSizing: 'border-box' }}>
      <Handle type="target" position={Position.Top} style={{ background: '#1f2937' }} />
      <span style={{ width: 5, height: 5, borderRadius: '50%', background: dotColor, flexShrink: 0 }} />
      <span style={{ fontSize: 11, color: '#94a3b8', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{label}</span>
      <Handle type="source" position={Position.Bottom} style={{ background: '#1f2937' }} />
    </div>
  )
}
