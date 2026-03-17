import { useState, useCallback, useMemo } from 'react'
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
import type { Edge, Connection, NodeTypes } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { Plus, Download, FileCode, Layers } from 'lucide-react'
import { RoleNode } from './RoleNode'
import type { RoleNodeType } from './RoleNode'
import { RoutingEdge } from './RoutingEdge'
import { SchemaPanel } from './SchemaPanel'
import { exportWorkflowToml } from './toml'
import {
  INITIAL_ROLES,
  INITIAL_ROUTING,
  INITIAL_DOC_KINDS,
} from './types'
import type { WorkflowRole, RoutingRule } from './types'

const NODE_TYPES: NodeTypes = { role: RoleNode }
const EDGE_TYPES = { routing: RoutingEdge }

function rolesToNodes(roles: WorkflowRole[]): RoleNodeType[] {
  const count = roles.length
  return roles.map((role, i) => {
    const angle = (i / count) * 2 * Math.PI - Math.PI / 2
    const rx = count > 1 ? 260 : 0
    const ry = count > 1 ? 180 : 0
    const cx = 420
    const cy = 240
    return {
      id: role.id,
      type: 'role' as const,
      position: {
        x: cx + rx * Math.cos(angle) - 70,
        y: cy + ry * Math.sin(angle) - 40,
      },
      data: { ...role },
    }
  })
}

function routingToEdges(routing: RoutingRule[], knownIds: Set<string>): Edge[] {
  return routing.map((rule, i) => ({
    id: `edge-${i}`,
    source: rule.from,
    target: knownIds.has(rule.to) ? rule.to : rule.from,
    type: 'routing',
    data: { jobKind: rule.jobKind, gate: rule.gate },
    label: rule.to !== rule.from && !knownIds.has(rule.to) ? `→ ${rule.to}` : undefined,
  }))
}

let roleCounter = 0

interface AddRoleFormProps {
  onAdd: (role: WorkflowRole) => void
  onCancel: () => void
}

function AddRoleForm({ onAdd, onCancel }: AddRoleFormProps) {
  const [id, setId] = useState('')
  const [profile, setProfile] = useState('')

  const submit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!id.trim() || !profile.trim()) return
    onAdd({ id: id.trim(), profile: profile.trim() })
  }

  return (
    <div className="absolute inset-0 z-20 flex items-center justify-center bg-background/60 backdrop-blur-sm">
      <form
        onSubmit={submit}
        className="w-72 rounded-xl border border-border/70 bg-card shadow-xl p-5 space-y-4"
      >
        <h3 className="text-sm font-semibold text-foreground">Add Role</h3>
        <div className="space-y-2">
          <label className="block text-[11px] font-medium text-muted-foreground">
            Role ID
            <input
              value={id}
              onChange={(e) => setId(e.target.value)}
              placeholder="e.g. qa-lane"
              autoFocus
              className="mt-1 w-full rounded-md border border-border/60 bg-background px-3 py-1.5 font-mono text-xs text-foreground placeholder:text-muted-foreground/60 focus:border-primary focus:outline-none"
            />
          </label>
          <label className="block text-[11px] font-medium text-muted-foreground">
            Profile
            <input
              value={profile}
              onChange={(e) => setProfile(e.target.value)}
              placeholder="e.g. qa-lane"
              className="mt-1 w-full rounded-md border border-border/60 bg-background px-3 py-1.5 font-mono text-xs text-foreground placeholder:text-muted-foreground/60 focus:border-primary focus:outline-none"
            />
          </label>
        </div>
        <div className="flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="rounded-lg border border-border/60 px-3 py-1.5 text-xs text-muted-foreground hover:bg-muted transition"
          >
            Cancel
          </button>
          <button
            type="submit"
            disabled={!id.trim() || !profile.trim()}
            className="rounded-lg bg-primary px-3 py-1.5 text-xs font-semibold text-primary-foreground hover:opacity-90 disabled:opacity-40 transition"
          >
            Add
          </button>
        </div>
      </form>
    </div>
  )
}

export function WorkflowCanvas() {
  const [roles, setRoles] = useState<WorkflowRole[]>(INITIAL_ROLES)
  const [routing] = useState<RoutingRule[]>(INITIAL_ROUTING)
  const [showAddForm, setShowAddForm] = useState(false)
  const [showSchema, setShowSchema] = useState(false)

  const initialNodeIds = useMemo(() => new Set(INITIAL_ROLES.map((r) => r.id)), [])
  const initialNodes = useMemo(() => rolesToNodes(INITIAL_ROLES), [])
  const initialEdges = useMemo(() => routingToEdges(INITIAL_ROUTING, initialNodeIds), [initialNodeIds])

  const [nodes, , onNodesChange] = useNodesState<RoleNodeType>(initialNodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges)

  const onConnect = useCallback(
    (connection: Connection) => setEdges((eds) => addEdge(connection, eds)),
    [setEdges],
  )

  const handleAddRole = useCallback(
    (role: WorkflowRole) => {
      roleCounter++
      const newNode: RoleNodeType = {
        id: role.id,
        type: 'role' as const,
        position: { x: 200 + roleCounter * 200, y: 460 },
        data: { ...role },
      }
      // setNodes not needed — useNodesState manages internal state after addNode
      // We use onNodesChange-compatible approach: dispatch an add action
      onNodesChange([{ type: 'add' as const, item: newNode }])
      setRoles((prev) => [...prev, role])
      setShowAddForm(false)
    },
    [onNodesChange],
  )

  const handleExport = () => {
    const toml = exportWorkflowToml('shipflow', roles, routing)
    const blob = new Blob([toml], { type: 'text/plain' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'workflow.toml'
    a.click()
    URL.revokeObjectURL(url)
  }

  return (
    <div className="relative flex flex-1 min-h-0 overflow-hidden">
      {/* Canvas */}
      <div className="relative flex-1 min-w-0">
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          nodeTypes={NODE_TYPES}
          edgeTypes={EDGE_TYPES}
          fitView
          fitViewOptions={{ padding: 0.3 }}
          className="bg-background"
        >
          <Background variant={BackgroundVariant.Dots} gap={20} size={1} className="opacity-30" />
          <Controls className="[&_button]:bg-card [&_button]:border-border/60 [&_button]:text-foreground" />
          <MiniMap
            className="!bg-card !border !border-border/60 !rounded-xl"
            nodeColor="hsl(var(--muted))"
          />
        </ReactFlow>

        {/* Toolbar overlay */}
        <div className="absolute top-3 left-3 z-10 flex items-center gap-2">
          <div className="flex items-center gap-1.5 rounded-xl border border-border/60 bg-card/80 backdrop-blur-sm px-3 py-2 shadow-sm">
            <FileCode className="size-3.5 text-primary" />
            <span className="text-xs font-semibold text-foreground">workflow.toml</span>
            <span className="rounded-full bg-muted px-1.5 py-0.5 text-[9px] text-muted-foreground">
              {roles.length} roles · {routing.length} rules
            </span>
          </div>
          <button
            onClick={() => setShowAddForm(true)}
            className="flex items-center gap-1.5 rounded-xl border border-border/60 bg-card/80 backdrop-blur-sm px-3 py-2 text-xs font-medium text-foreground hover:bg-card transition shadow-sm"
          >
            <Plus className="size-3.5" />
            Add Role
          </button>
          <button
            onClick={() => setShowSchema((v) => !v)}
            className={`flex items-center gap-1.5 rounded-xl border px-3 py-2 text-xs font-medium transition shadow-sm backdrop-blur-sm ${
              showSchema
                ? 'border-primary/40 bg-primary/10 text-primary'
                : 'border-border/60 bg-card/80 text-foreground hover:bg-card'
            }`}
          >
            <Layers className="size-3.5" />
            Schema
          </button>
          <button
            onClick={handleExport}
            className="flex items-center gap-1.5 rounded-xl bg-primary px-3 py-2 text-xs font-semibold text-primary-foreground hover:opacity-90 transition shadow-sm"
          >
            <Download className="size-3.5" />
            Export
          </button>
        </div>

        {/* Add role form */}
        {showAddForm && (
          <AddRoleForm onAdd={handleAddRole} onCancel={() => setShowAddForm(false)} />
        )}
      </div>

      {/* Schema panel */}
      {showSchema && (
        <SchemaPanel docKinds={INITIAL_DOC_KINDS} onClose={() => setShowSchema(false)} />
      )}
    </div>
  )
}
