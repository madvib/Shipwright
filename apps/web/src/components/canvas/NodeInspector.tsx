import { X, Trash2 } from 'lucide-react'
import type { Node } from '@xyflow/react'
import type { ArtifactNodeData, AgentNodeData, PlatformNodeData } from './types'
import { STATUS_COLORS, AGENT_STYLES } from './types'

interface Props {
  node: Node
  onUpdate: (id: string, data: Record<string, unknown>) => void
  onDelete: (id: string) => void
  onClose: () => void
}

export function NodeInspector({ node, onUpdate, onDelete, onClose }: Props) {
  const update = (patch: Record<string, unknown>) => onUpdate(node.id, { ...node.data, ...patch })

  return (
    <div className="w-72 border-l border-border/60 bg-card/95 backdrop-blur-md flex flex-col shrink-0 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2.5 border-b border-border/60">
        <span className="text-[10px] font-semibold uppercase tracking-widest text-muted-foreground">
          {node.type === 'artifact' ? 'Artifact' : node.type === 'agent' ? 'Agent' : 'Platform'}
        </span>
        <div className="flex items-center gap-1">
          <button
            onClick={() => onDelete(node.id)}
            className="p-1 rounded text-muted-foreground/40 hover:text-destructive hover:bg-destructive/10 transition-colors"
            title="Delete node"
          >
            <Trash2 className="size-3.5" />
          </button>
          <button
            onClick={onClose}
            className="p-1 rounded text-muted-foreground/40 hover:text-foreground hover:bg-muted transition-colors"
          >
            <X className="size-3.5" />
          </button>
        </div>
      </div>

      {/* Body */}
      <div className="flex-1 overflow-auto p-3 space-y-4">
        {node.type === 'artifact' && <ArtifactFields data={node.data as ArtifactNodeData} onUpdate={update} />}
        {node.type === 'agent' && <AgentFields data={node.data as AgentNodeData} onUpdate={update} />}
        {node.type === 'platform' && <PlatformFields data={node.data as PlatformNodeData} onUpdate={update} />}

        {/* Position (read-only) */}
        <div>
          <Label>Position</Label>
          <div className="flex gap-2">
            <div className="flex-1 rounded border border-border/40 bg-muted/20 px-2 py-1 text-[10px] font-mono text-muted-foreground">
              x: {Math.round(node.position.x)}
            </div>
            <div className="flex-1 rounded border border-border/40 bg-muted/20 px-2 py-1 text-[10px] font-mono text-muted-foreground">
              y: {Math.round(node.position.y)}
            </div>
          </div>
        </div>

        {/* Node ID */}
        <div>
          <Label>ID</Label>
          <div className="rounded border border-border/40 bg-muted/20 px-2 py-1 text-[10px] font-mono text-muted-foreground truncate">
            {node.id}
          </div>
        </div>
      </div>
    </div>
  )
}

function Label({ children }: { children: React.ReactNode }) {
  return <div className="text-[9px] font-semibold uppercase tracking-widest text-muted-foreground/60 mb-1">{children}</div>
}

function Input({ value, onChange, placeholder }: { value: string; onChange: (v: string) => void; placeholder?: string }) {
  return (
    <input
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      spellCheck={false}
      className="w-full rounded border border-border/60 bg-background px-2 py-1.5 text-xs text-foreground placeholder:text-muted-foreground/40 focus:outline-none focus:border-primary/50 transition-colors"
    />
  )
}

function Select({ value, onChange, options }: { value: string; onChange: (v: string) => void; options: { value: string; label: string }[] }) {
  return (
    <select
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className="w-full rounded border border-border/60 bg-background px-2 py-1.5 text-xs text-foreground focus:outline-none focus:border-primary/50 transition-colors"
    >
      {options.map((o) => <option key={o.value} value={o.value}>{o.label}</option>)}
    </select>
  )
}

function ArtifactFields({ data, onUpdate }: { data: ArtifactNodeData; onUpdate: (p: Record<string, unknown>) => void }) {
  const depthLabels = ['D0 — Target', 'D1 — Capability', 'D2 — Job']
  return (
    <>
      <div>
        <Label>Label</Label>
        <Input value={data.label} onChange={(label) => onUpdate({ label })} placeholder="Artifact name" />
      </div>
      <div>
        <Label>Depth</Label>
        <Select
          value={String(data.depth)}
          onChange={(v) => onUpdate({ depth: Number(v) })}
          options={[0, 1, 2].map((d) => ({ value: String(d), label: depthLabels[d] }))}
        />
      </div>
      <div>
        <Label>Status</Label>
        <div className="flex gap-1.5 flex-wrap">
          {(['planned', 'in-flight', 'actual', 'blocked'] as const).map((s) => (
            <button
              key={s}
              onClick={() => onUpdate({ status: s })}
              className={`flex items-center gap-1.5 rounded px-2 py-1 text-[10px] font-medium border transition-colors ${
                data.status === s
                  ? 'border-foreground/20 bg-foreground/5 text-foreground'
                  : 'border-border/40 text-muted-foreground hover:text-foreground'
              }`}
            >
              <span className="size-1.5 rounded-full" style={{ background: STATUS_COLORS[s] }} />
              {s}
            </button>
          ))}
        </div>
      </div>
      <div>
        <Label>Subtitle</Label>
        <Input value={data.subtitle ?? ''} onChange={(subtitle) => onUpdate({ subtitle: subtitle || undefined })} placeholder="Optional" />
      </div>
      {data.depth === 1 && (
        <div>
          <Label>Accent Color</Label>
          <div className="flex items-center gap-2">
            <input
              type="color"
              value={data.accentColor ?? '#3b82f6'}
              onChange={(e) => onUpdate({ accentColor: e.target.value })}
              className="size-7 rounded border border-border/60 cursor-pointer"
            />
            <span className="text-[10px] font-mono text-muted-foreground">{data.accentColor ?? '#3b82f6'}</span>
          </div>
        </div>
      )}
    </>
  )
}

function AgentFields({ data, onUpdate }: { data: AgentNodeData; onUpdate: (p: Record<string, unknown>) => void }) {
  return (
    <>
      <div>
        <Label>Name</Label>
        <Input value={data.name} onChange={(name) => onUpdate({ name })} placeholder="Agent name" />
      </div>
      <div>
        <Label>Type</Label>
        <div className="flex gap-1.5 flex-wrap">
          {(['human', 'commander', 'specialist', 'gate'] as const).map((t) => {
            const s = AGENT_STYLES[t]
            return (
              <button
                key={t}
                onClick={() => onUpdate({ agentType: t })}
                className={`flex items-center gap-1.5 rounded px-2 py-1 text-[10px] font-medium border transition-colors ${
                  data.agentType === t
                    ? 'border-foreground/20 bg-foreground/5 text-foreground'
                    : 'border-border/40 text-muted-foreground hover:text-foreground'
                }`}
              >
                <span className="size-1.5 rounded-full" style={{ background: s.stroke }} />
                {t}
              </button>
            )
          })}
        </div>
      </div>
      <div>
        <Label>Profile</Label>
        <Input value={data.profile} onChange={(profile) => onUpdate({ profile })} placeholder="Profile name" />
      </div>
      <div>
        <Label>Icon</Label>
        <Input value={data.icon ?? ''} onChange={(icon) => onUpdate({ icon: icon || undefined })} placeholder="Emoji or symbol" />
      </div>
      <div>
        <Label>Detail</Label>
        <Input value={data.detail ?? ''} onChange={(detail) => onUpdate({ detail: detail || undefined })} placeholder="Optional context" />
      </div>
      <div>
        <Label>Badge</Label>
        <Input value={data.badge ?? ''} onChange={(badge) => onUpdate({ badge: badge || undefined })} placeholder="Optional badge" />
      </div>
    </>
  )
}

function PlatformFields({ data, onUpdate }: { data: PlatformNodeData; onUpdate: (p: Record<string, unknown>) => void }) {
  return (
    <>
      <div>
        <Label>Kind</Label>
        <Select
          value={data.nodeKind}
          onChange={(nodeKind) => onUpdate({ nodeKind })}
          options={[
            { value: 'mcp', label: 'MCP Server' },
            { value: 'hook', label: 'Hook' },
          ]}
        />
      </div>
      <div>
        <Label>Title</Label>
        <Input value={data.title} onChange={(title) => onUpdate({ title })} placeholder="Server or hook name" />
      </div>
      <div>
        <Label>Detail</Label>
        <Input value={data.detail} onChange={(detail) => onUpdate({ detail })} placeholder="e.g. tools: 5" />
      </div>
    </>
  )
}
