import { Handle, Position } from '@xyflow/react'
import type { Node, NodeProps } from '@xyflow/react'
import { Bot } from 'lucide-react'
import type { WorkflowRole } from './types'

export type RoleNodeData = WorkflowRole & Record<string, unknown>
export type RoleNodeType = Node<RoleNodeData, 'role'>

export function RoleNode({ data, selected }: NodeProps<RoleNodeType>) {
  return (
    <div
      className={`min-w-[140px] rounded-xl border bg-card shadow-sm transition-all ${
        selected
          ? 'border-primary shadow-md shadow-primary/10'
          : 'border-border/70 hover:border-border'
      }`}
    >
      <Handle
        type="target"
        position={Position.Top}
        className="!w-2.5 !h-2.5 !bg-muted-foreground/40 !border-border/60 !rounded-full"
      />

      <div className="px-3 py-2.5">
        <div className="flex items-start gap-2">
          <div className="mt-0.5 flex size-6 shrink-0 items-center justify-center rounded-lg bg-primary/10">
            <Bot className="size-3.5 text-primary" />
          </div>
          <div className="min-w-0">
            <p className="text-[11px] font-mono font-semibold text-foreground leading-tight truncate">
              {data.id}
            </p>
            <p className="text-[10px] text-muted-foreground truncate mt-0.5">
              {data.profile}
            </p>
          </div>
        </div>
        {data.default && (
          <div className="mt-2 flex">
            <span className="rounded-full bg-emerald-500/10 px-2 py-0.5 text-[9px] font-semibold text-emerald-600 dark:text-emerald-400 uppercase tracking-wide">
              default
            </span>
          </div>
        )}
      </div>

      <Handle
        type="source"
        position={Position.Bottom}
        className="!w-2.5 !h-2.5 !bg-muted-foreground/40 !border-border/60 !rounded-full"
      />
    </div>
  )
}
