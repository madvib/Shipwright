import { useCallback, useEffect, useRef, useState } from 'react'
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
import type { Connection, NodeTypes, EdgeTypes, Node, Edge, NodeChange } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { ArtifactNode } from './nodes/ArtifactNode'
import { AgentNode } from './nodes/AgentNode'
import { PlatformNode } from './nodes/PlatformNode'
import { ChannelEdge } from './edges/ChannelEdge'
import { WorkflowToolbar } from './WorkflowToolbar'
import { NodeInspector } from './NodeInspector'
import type { WorkflowPreset } from './types'

const CANVAS_STYLES = `
  @keyframes node-pulse {
    0%, 100% { opacity: 1; }
    50%      { opacity: 0.4; }
  }
  .react-flow__controls button {
    background: hsl(var(--card)) !important;
    border-color: hsl(var(--border)) !important;
    color: hsl(var(--muted-foreground)) !important;
  }
  .react-flow__controls button:hover {
    background: hsl(var(--muted)) !important;
    color: hsl(var(--foreground)) !important;
  }
  .react-flow__minimap {
    background: hsl(var(--card)) !important;
    border: 1px solid hsl(var(--border)) !important;
    border-radius: 8px !important;
  }
`

const NODE_TYPES: NodeTypes = {
  artifact: ArtifactNode,
  agent: AgentNode,
  platform: PlatformNode,
}

const EDGE_TYPES: EdgeTypes = { channel: ChannelEdge }

// ── Canvas localStorage persistence ──────────────────────────────────────────

const CANVAS_STORAGE_PREFIX = 'ship-canvas-'

function canvasKey(presetName: string): string {
  return CANVAS_STORAGE_PREFIX + presetName.toLowerCase().replace(/\s+/g, '-')
}

function loadCanvasState(key: string): { nodes: Node[]; edges: Edge[] } | null {
  try {
    const raw = window.localStorage.getItem(key)
    if (!raw) return null
    return JSON.parse(raw) as { nodes: Node[]; edges: Edge[] }
  } catch {
    return null
  }
}

function saveCanvasState(key: string, nodes: Node[], edges: Edge[]): void {
  try {
    window.localStorage.setItem(key, JSON.stringify({ nodes, edges }))
  } catch { /* quota exceeded or unavailable — ignore */ }
}

// ── Component ────────────────────────────────────────────────────────────────

interface Props {
  preset: WorkflowPreset
  presetName: string
  onBack: () => void
}

export function WorkflowCanvas({ preset, presetName, onBack }: Props) {
  const storageKey = canvasKey(presetName)
  const stored = useRef(loadCanvasState(storageKey))
  const initialNodes = stored.current?.nodes ?? preset.nodes
  const initialEdges = stored.current?.edges ?? preset.edges

  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges)
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)

  // Persist canvas edits (debounced 500ms)
  const saveTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
  useEffect(() => {
    if (saveTimer.current) clearTimeout(saveTimer.current)
    saveTimer.current = setTimeout(() => saveCanvasState(storageKey, nodes, edges), 500)
    return () => { if (saveTimer.current) clearTimeout(saveTimer.current) }
  }, [nodes, edges, storageKey])

  const selectedNode = selectedNodeId ? nodes.find((n) => n.id === selectedNodeId) : null

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
      const item: Node = { id, type, position: { x: 400, y: 300 }, data }
      setNodes((nds) => [...nds, item])
      setSelectedNodeId(id)
    },
    [setNodes],
  )

  const updateNodeData = useCallback(
    (id: string, data: Record<string, unknown>) => {
      setNodes((nds) => nds.map((n) => (n.id === id ? { ...n, data } : n)))
    },
    [setNodes],
  )

  const deleteNode = useCallback(
    (id: string) => {
      setNodes((nds) => nds.filter((n) => n.id !== id))
      setEdges((eds) => eds.filter((e) => e.source !== id && e.target !== id))
      setSelectedNodeId(null)
    },
    [setNodes, setEdges],
  )

  const handleNodesChange = useCallback(
    (changes: NodeChange[]) => {
      onNodesChange(changes)
      // Track selection changes
      for (const change of changes) {
        if (change.type === 'select' && change.selected) {
          setSelectedNodeId(change.id)
        }
      }
    },
    [onNodesChange],
  )

  return (
    <div className="flex flex-col flex-1 min-h-0">
      <style>{CANVAS_STYLES}</style>
      <WorkflowToolbar name={presetName} onBack={onBack} onAddNode={addNode} />
      <div className="flex flex-1 min-h-0">
        <div className="flex-1 min-h-0">
          <ReactFlow
            nodes={nodes}
            edges={edges}
            onNodesChange={handleNodesChange}
            onEdgesChange={onEdgesChange}
            onConnect={onConnect}
            nodeTypes={NODE_TYPES}
            edgeTypes={EDGE_TYPES}
            onPaneClick={() => setSelectedNodeId(null)}
            fitView
            fitViewOptions={{ padding: 0.2 }}
            className="bg-background"
          >
            <Background variant={BackgroundVariant.Dots} gap={24} size={1} className="!fill-muted-foreground/10" />
            <Controls />
            <MiniMap nodeColor="hsl(var(--muted))" maskColor="hsl(var(--background) / 0.7)" />
          </ReactFlow>
        </div>
        {selectedNode && (
          <NodeInspector
            node={selectedNode}
            onUpdate={updateNodeData}
            onDelete={deleteNode}
            onClose={() => setSelectedNodeId(null)}
          />
        )}
      </div>
    </div>
  )
}
