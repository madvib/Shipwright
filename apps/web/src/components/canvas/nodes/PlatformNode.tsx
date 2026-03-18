import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { PlatformNodeType } from '../types'

const styles = {
  mcp: {
    container: { background: '#091420', border: '1.5px solid #0ea5e933', borderRadius: 6, width: 200, minHeight: 64, padding: '8px 12px', boxSizing: 'border-box' as const },
    label: { fontSize: 10, color: '#0ea5e9', fontWeight: 500 },
    title: { fontSize: 11, color: '#7dd3fc', marginTop: 2 },
    detail: { fontSize: 10, color: '#334155', marginTop: 2 },
  },
  hook: {
    container: { background: '#140d1f', border: '1.5px solid #a855f733', borderRadius: 6, width: 200, minHeight: 64, padding: '8px 12px', boxSizing: 'border-box' as const },
    label: { fontSize: 10, color: '#a855f7', fontWeight: 500 },
    title: null,
    detail: { fontSize: 11, color: '#d8b4fe', marginTop: 2 },
  },
}

export function PlatformNode({ data }: NodeProps<PlatformNodeType>) {
  const s = styles[data.nodeKind]
  return (
    <div style={s.container}>
      <Handle type="target" position={Position.Left} />
      {data.nodeKind === 'mcp' ? (
        <>
          <div style={s.label}>MCP Server</div>
          <div style={s.title!}>{data.title}</div>
          <div style={s.detail}>{data.detail}</div>
        </>
      ) : (
        <>
          <div style={s.label}>Hook · {data.title}</div>
          <div style={s.detail}>{data.detail}</div>
        </>
      )}
      <Handle type="source" position={Position.Right} />
    </div>
  )
}
