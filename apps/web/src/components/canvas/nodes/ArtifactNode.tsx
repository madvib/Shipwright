import type { NodeProps } from '@xyflow/react'
import { Handle, Position } from '@xyflow/react'
import type { ArtifactNodeType } from '../types'
import { STATUS_COLORS } from '../types'

export function ArtifactNode(props: NodeProps<ArtifactNodeType>) {
  const { data } = props
  const { label, depth, accentColor = '#3b82f6', status, subtitle } = data
  const statusColor = status ? STATUS_COLORS[status] : undefined
  const isActive = status === 'in-flight'

  if (depth === 0) {
    return (
      <div className="relative w-[280px] min-h-[90px] bg-card border-[1.5px] border-border rounded-sm p-3 shadow-md transition-shadow">
        <Handle type="target" position={Position.Top} className="!bg-border" />
        <svg className="absolute top-0 right-0" width={18} height={18} viewBox="0 0 18 18">
          <polygon points="0,0 18,0 18,18" className="fill-muted" />
        </svg>
        <div className="text-[9px] font-semibold uppercase tracking-widest text-muted-foreground mb-1.5">
          Target
        </div>
        <div className="flex items-center gap-2">
          {statusColor && (
            <span
              className="size-[7px] rounded-full shrink-0"
              style={{
                background: statusColor,
                boxShadow: isActive ? `0 0 6px ${statusColor}88` : undefined,
                animation: isActive ? 'node-pulse 2s ease-in-out infinite' : undefined,
              }}
            />
          )}
          <div className="text-sm font-bold text-foreground leading-tight tracking-tight">
            {label}
          </div>
        </div>
        {subtitle && (
          <div className="text-[10px] text-muted-foreground mt-1.5">{subtitle}</div>
        )}
        <Handle type="source" position={Position.Bottom} className="!bg-border" />
      </div>
    )
  }

  if (depth === 1) {
    return (
      <div
        className="relative w-[220px] min-h-[68px] bg-card border-[1.5px] border-border/60 rounded-md overflow-hidden shadow-sm transition-shadow"
        style={{ boxShadow: `inset 3px 0 0 ${accentColor}` }}
      >
        <Handle type="target" position={Position.Top} className="!bg-border" />
        <div className="p-2.5 pl-3.5">
          <div
            className="text-[9px] font-semibold uppercase tracking-widest mb-1 opacity-85"
            style={{ color: accentColor }}
          >
            Capability
          </div>
          <div className="text-xs font-semibold text-foreground leading-tight tracking-tight">
            {label}
          </div>
          {(subtitle || status) && (
            <div className="flex items-center gap-1.5 mt-1">
              {statusColor && (
                <span
                  className="size-1.5 rounded-full shrink-0"
                  style={{
                    background: statusColor,
                    boxShadow: isActive ? `0 0 5px ${statusColor}66` : undefined,
                    animation: isActive ? 'node-pulse 2s ease-in-out infinite' : undefined,
                  }}
                />
              )}
              <span className="text-[9px] text-muted-foreground font-medium">
                {status ?? subtitle}
              </span>
            </div>
          )}
        </div>
        <Handle type="source" position={Position.Bottom} className="!bg-border" />
      </div>
    )
  }

  // depth === 2
  const dotColor = statusColor ?? 'hsl(var(--muted-foreground))'
  return (
    <div className="relative w-[180px] h-7 bg-card border border-border rounded flex items-center gap-1.5 px-2 transition-colors">
      <Handle type="target" position={Position.Top} className="!bg-border" />
      <span className="size-[5px] rounded-full shrink-0" style={{ background: dotColor }} />
      <span className="text-[10px] text-muted-foreground font-medium overflow-hidden text-ellipsis whitespace-nowrap">
        {label}
      </span>
      <Handle type="source" position={Position.Bottom} className="!bg-border" />
    </div>
  )
}
