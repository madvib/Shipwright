import { BaseEdge, EdgeLabelRenderer, getBezierPath } from '@xyflow/react'
import type { EdgeProps } from '@xyflow/react'
import { ShieldCheck } from 'lucide-react'

interface RoutingEdgeData {
  jobKind: string
  gate: boolean
}

export function RoutingEdge({
  id,
  sourceX,
  sourceY,
  targetX,
  targetY,
  sourcePosition,
  targetPosition,
  data,
  selected,
}: EdgeProps) {
  const edgeData = data as RoutingEdgeData | undefined
  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    sourcePosition,
    targetX,
    targetY,
    targetPosition,
  })

  return (
    <>
      <BaseEdge
        id={id}
        path={edgePath}
        style={{
          stroke: selected ? 'hsl(var(--primary))' : 'hsl(var(--border))',
          strokeWidth: selected ? 2 : 1.5,
        }}
      />
      <EdgeLabelRenderer>
        <div
          style={{
            transform: `translate(-50%, -50%) translate(${labelX}px,${labelY}px)`,
            pointerEvents: 'all',
          }}
          className="nodrag nopan absolute"
        >
          <div className="flex items-center gap-1 rounded-full border border-border/60 bg-card px-2 py-0.5 shadow-sm text-[10px] font-mono font-medium text-foreground/70">
            {edgeData?.gate && (
              <ShieldCheck className="size-2.5 text-amber-500 shrink-0" />
            )}
            <span>{edgeData?.jobKind ?? ''}</span>
          </div>
        </div>
      </EdgeLabelRenderer>
    </>
  )
}
