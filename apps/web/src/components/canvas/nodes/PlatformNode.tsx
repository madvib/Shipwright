import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { PlatformNodeType } from '../types'

const variants = {
  mcp: { accent: '#0ea5e9', icon: '{}' },
  hook: { accent: '#a855f7', icon: 'fn' },
}

export function PlatformNode({ data }: NodeProps<PlatformNodeType>) {
  const v = variants[data.nodeKind]
  return (
    <div
      className="bg-card border-[1.5px] rounded-md w-[200px] min-h-[64px] p-2 pl-3 transition-shadow"
      style={{
        borderColor: `${v.accent}28`,
        boxShadow: `0 0 12px ${v.accent}10`,
      }}
    >
      <Handle type="target" position={Position.Left} />
      <div className="flex items-center gap-1.5 mb-0.5">
        <span
          className="font-mono text-[8px] font-bold tracking-wide rounded px-1 py-px"
          style={{
            color: v.accent,
            background: `${v.accent}12`,
            border: `1px solid ${v.accent}20`,
          }}
        >
          {v.icon}
        </span>
        <span
          className="text-[9px] font-semibold uppercase tracking-widest"
          style={{ color: v.accent }}
        >
          {data.nodeKind === 'mcp' ? 'MCP Server' : 'Hook'}
        </span>
      </div>
      <div className="text-[11px] font-semibold text-foreground mt-0.5">
        {data.title}
      </div>
      <div
        className="font-mono text-[9px] mt-0.5"
        style={{ color: `${v.accent}66` }}
      >
        {data.detail}
      </div>
      <Handle type="source" position={Position.Right} />
    </div>
  )
}
