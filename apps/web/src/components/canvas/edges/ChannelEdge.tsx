import { getBezierPath, EdgeLabelRenderer, BaseEdge, type EdgeProps } from '@xyflow/react'
import { CHANNEL_STYLES, type ChannelEdgeData } from '../types'

const DASH: Record<string, string> = {
  planning: '6 3',
  output: '5 3',
  blocked: '4 2',
}

export function ChannelEdge(props: EdgeProps) {
  const {
    sourceX, sourceY, sourcePosition,
    targetX, targetY, targetPosition,
    style,
  } = props

  const data = props.data as ChannelEdgeData | undefined
  const channelType = data?.channelType ?? 'dispatch'
  const { color, dashed } = CHANNEL_STYLES[channelType]
  const markerId = `arrow-${channelType}`

  const [edgePath, labelX, labelY] = getBezierPath({
    sourceX, sourceY, sourcePosition,
    targetX, targetY, targetPosition,
  })

  const strokeDasharray = dashed ? DASH[channelType] : undefined

  return (
    <>
      <svg style={{ overflow: 'visible', position: 'absolute', width: 0, height: 0 }}>
        <defs>
          <marker
            id={markerId}
            viewBox="0 0 8 8"
            refX={6}
            refY={4}
            markerWidth={6}
            markerHeight={6}
            orient="auto"
          >
            <path d="M0,0 L8,4 L0,8 Z" fill={color} />
          </marker>
        </defs>
      </svg>
      <BaseEdge
        path={edgePath}
        markerEnd={`url(#${markerId})`}
        style={{
          ...style,
          stroke: color,
          strokeWidth: 1.5,
          strokeDasharray,
        }}
      />
      {data?.label && (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              transform: `translate(-50%, -50%) translate(${labelX}px, ${labelY}px)`,
              background: '#0d0d14',
              border: `1px solid ${color}25`,
              borderRadius: 4,
              padding: '2px 8px',
              fontFamily: '"DM Sans Variable", "DM Sans", sans-serif',
              fontSize: 9,
              fontWeight: 600,
              color: `${color}aa`,
              letterSpacing: '0.02em',
              pointerEvents: 'all',
              whiteSpace: 'nowrap',
            }}
            className="nodrag nopan"
          >
            {data.label}
          </div>
        </EdgeLabelRenderer>
      )}
    </>
  )
}
