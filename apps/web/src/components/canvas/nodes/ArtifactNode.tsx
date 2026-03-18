import type { NodeProps } from '@xyflow/react'
import { Handle, Position } from '@xyflow/react'
import type { ArtifactNodeType } from '../types'
import { STATUS_COLORS } from '../types'

const FONT_DISPLAY = '"Syne Variable", "Syne", sans-serif'
const FONT_BODY = '"DM Sans Variable", "DM Sans", sans-serif'

export function ArtifactNode(props: NodeProps<ArtifactNodeType>) {
  const { data } = props
  const { label, depth, accentColor = '#3b82f6', status, subtitle } = data
  const statusColor = status ? STATUS_COLORS[status] : undefined
  const isActive = status === 'in-flight'

  if (depth === 0) {
    return (
      <div style={{
        position: 'relative', width: 280, minHeight: 90, background: '#0f1520',
        border: '1.5px solid #334155', borderRadius: 3,
        padding: '12px 14px', boxSizing: 'border-box',
        boxShadow: '0 2px 12px rgba(0,0,0,0.3)',
        transition: 'box-shadow 0.2s',
      }}>
        <Handle type="target" position={Position.Top} style={{ background: '#334155' }} />
        {/* Folded corner */}
        <svg style={{ position: 'absolute', top: 0, right: 0 }} width={18} height={18} viewBox="0 0 18 18">
          <polygon points="0,0 18,0 18,18" fill="#1e2d3d" />
        </svg>
        <div style={{
          fontFamily: FONT_BODY, fontSize: 9, color: '#64748b',
          textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 5,
          fontWeight: 600,
        }}>
          Target
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: 7 }}>
          {statusColor && (
            <span style={{
              width: 7, height: 7, borderRadius: '50%', background: statusColor, flexShrink: 0,
              boxShadow: isActive ? `0 0 6px ${statusColor}88` : undefined,
              animation: isActive ? 'node-pulse 2s ease-in-out infinite' : undefined,
            }} />
          )}
          <div style={{
            fontFamily: FONT_DISPLAY, fontSize: 14, color: '#f1f5f9',
            fontWeight: 700, lineHeight: 1.3, letterSpacing: '-0.01em',
          }}>
            {label}
          </div>
        </div>
        {subtitle && (
          <div style={{
            fontFamily: FONT_BODY, fontSize: 10, color: '#475569', marginTop: 5,
          }}>
            {subtitle}
          </div>
        )}
        <Handle type="source" position={Position.Bottom} style={{ background: '#334155' }} />
      </div>
    )
  }

  if (depth === 1) {
    return (
      <div style={{
        position: 'relative', width: 220, minHeight: 68, background: '#0d1117',
        border: '1.5px solid #1e2535', borderRadius: 6,
        boxSizing: 'border-box', overflow: 'hidden',
        boxShadow: `0 1px 8px rgba(0,0,0,0.2), inset 3px 0 0 ${accentColor}`,
        transition: 'box-shadow 0.2s',
      }}>
        <Handle type="target" position={Position.Top} style={{ background: '#1e2535' }} />
        <div style={{ padding: '10px 12px 10px 14px' }}>
          <div style={{
            fontFamily: FONT_BODY, fontSize: 9, color: accentColor,
            textTransform: 'uppercase', letterSpacing: '0.1em', marginBottom: 4,
            fontWeight: 600, opacity: 0.85,
          }}>
            Capability
          </div>
          <div style={{
            fontFamily: FONT_DISPLAY, fontSize: 12, color: '#e2e8f0',
            fontWeight: 600, lineHeight: 1.3, letterSpacing: '-0.005em',
          }}>
            {label}
          </div>
          {(subtitle || status) && (
            <div style={{ display: 'flex', alignItems: 'center', gap: 5, marginTop: 4 }}>
              {statusColor && (
                <span style={{
                  width: 6, height: 6, borderRadius: '50%', background: statusColor, flexShrink: 0,
                  boxShadow: isActive ? `0 0 5px ${statusColor}66` : undefined,
                  animation: isActive ? 'node-pulse 2s ease-in-out infinite' : undefined,
                }} />
              )}
              {status && (
                <span style={{ fontFamily: FONT_BODY, fontSize: 9, color: '#475569', fontWeight: 500 }}>
                  {status}
                </span>
              )}
              {subtitle && !status && (
                <span style={{ fontFamily: FONT_BODY, fontSize: 9, color: '#475569' }}>{subtitle}</span>
              )}
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
    <div style={{
      position: 'relative', width: 180, height: 28, background: '#111827',
      border: '1px solid #1f2937', borderRadius: 5,
      display: 'flex', alignItems: 'center', gap: 6, padding: '0 8px',
      boxSizing: 'border-box',
      transition: 'border-color 0.2s',
    }}>
      <Handle type="target" position={Position.Top} style={{ background: '#1f2937' }} />
      <span style={{
        width: 5, height: 5, borderRadius: '50%', background: dotColor, flexShrink: 0,
        boxShadow: isActive ? `0 0 4px ${dotColor}66` : undefined,
      }} />
      <span style={{
        fontFamily: FONT_BODY, fontSize: 10, color: '#94a3b8',
        overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap',
        fontWeight: 500,
      }}>
        {label}
      </span>
      <Handle type="source" position={Position.Bottom} style={{ background: '#1f2937' }} />
    </div>
  )
}
