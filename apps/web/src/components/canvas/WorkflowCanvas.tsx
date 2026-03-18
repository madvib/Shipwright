import { useCallback } from 'react'
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  addEdge,
  useNodesState,
  useEdgesState,
  BackgroundVariant,
} from '@xyflow/react'
import type { Connection, NodeTypes, EdgeTypes } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { ArtifactNode } from './nodes/ArtifactNode'
import { AgentNode } from './nodes/AgentNode'
import { PlatformNode } from './nodes/PlatformNode'
import { ChannelEdge } from './edges/ChannelEdge'
import { WorkflowToolbar } from './WorkflowToolbar'
import type { WorkflowPreset } from './types'

/* Keyframes used by canvas node components */
const CANVAS_STYLES = `
  @keyframes node-pulse {
    0%, 100% { opacity: 1; }
    50%      { opacity: 0.4; }
  }
`

const NODE_TYPES: NodeTypes = {
  artifact: ArtifactNode,
  agent: AgentNode,
  platform: PlatformNode,
}

const EDGE_TYPES: EdgeTypes = { channel: ChannelEdge }

interface Props {
  preset: WorkflowPreset
  presetName: string
  onBack: () => void
}

export function WorkflowCanvas({ preset, presetName, onBack }: Props) {
  const [nodes, , onNodesChange] = useNodesState(preset.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(preset.edges)

  const onConnect = useCallback(
    (connection: Connection) =>
      setEdges((eds) =>
        addEdge({ ...connection, type: 'channel', data: { channelType: 'dispatch' } }, eds),
      ),
    [setEdges],
  )

  const addNode = useCallback(
    (type: string, data: Record<string, unknown>) => {
      const id = `${type}-${Date.now()}`
      onNodesChange([
        { type: 'add', item: { id, type, position: { x: 400, y: 300 }, data } },
      ])
    },
    [onNodesChange],
  )

  return (
    <div className="flex flex-col flex-1 min-h-0">
      <style>{CANVAS_STYLES}</style>
      <WorkflowToolbar name={presetName} onBack={onBack} onAddNode={addNode} />
      <div className="flex-1 min-h-0">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          nodeTypes={NODE_TYPES}
          edgeTypes={EDGE_TYPES}
          fitView
          fitViewOptions={{ padding: 0.2 }}
          style={{ background: '#0a0a0f' }}
        >
          <Background variant={BackgroundVariant.Dots} gap={24} size={1} color="#1e2030" />
          <Controls className="[&_button]:!bg-[#0d0d14] [&_button]:!border-[#1e2030] [&_button]:!text-[#94a3b8]" />
          <MiniMap className="!bg-[#0d0d14] !border !border-[#1e2030] !rounded-lg" nodeColor="#1e2535" />
        </ReactFlow>
      </div>
    </div>
  )
}
