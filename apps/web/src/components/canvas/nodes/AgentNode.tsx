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

export function AgentNode(props: NodeProps<AgentNodeType>) {
  const { data } = props
  const { name, profile, agentType, badge, icon, detail } = data
  const s = AGENT_STYLES[agentType]

  return (
    <div
      className="relative w-[200px] min-h-[100px] bg-card rounded-lg p-2.5 flex flex-col gap-1.5 transition-shadow"
      style={{
        border: `1.5px ${s.dashed ? 'dashed' : 'solid'} ${s.stroke}`,
        boxShadow: s.glow
          ? `0 0 16px ${s.stroke}30, 0 0 4px ${s.stroke}18`
          : '0 1px 6px hsl(var(--foreground) / 0.05)',
      }}
    >
      <Handle type="target" position={Position.Top} style={{ background: s.stroke }} />

      <div className="flex items-start gap-2">
        <div
          className="size-8 rounded-md flex items-center justify-center text-base shrink-0 border"
          style={{
            background: `${s.stroke}10`,
            borderColor: `${s.stroke}18`,
          }}
        >
          {icon ?? DEFAULT_ICONS[agentType]}
        </div>
        <div className="flex flex-col gap-0.5 overflow-hidden">
          <div className="text-xs font-bold text-foreground tracking-tight truncate">
            {name}
          </div>
          <div className="text-[10px] text-muted-foreground font-medium truncate">
            {profile}
          </div>
          {detail && (
            <div className="text-[9px] text-muted-foreground/60 truncate">
              {detail}
            </div>
          )}
        </div>
      </div>

      {badge && (
        <div
          className="self-start text-[8px] font-bold uppercase tracking-widest px-1.5 py-0.5 rounded"
          style={{
            background: `${s.stroke}15`,
            color: s.stroke,
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
