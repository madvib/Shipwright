import type { Node, Edge } from '@xyflow/react'

// ── Artifact Node (D0 / D1 / D2) ──
export interface ArtifactNodeData {
  label: string
  depth: 0 | 1 | 2
  accentColor?: string
  status?: 'actual' | 'in-flight' | 'planned' | 'blocked'
  subtitle?: string
  [key: string]: unknown
}
export type ArtifactNodeType = Node<ArtifactNodeData, 'artifact'>

// ── Agent Node ──
export interface AgentNodeData {
  name: string
  profile: string
  agentType: 'human' | 'commander' | 'specialist' | 'gate'
  badge?: string
  icon?: string
  detail?: string
  [key: string]: unknown
}
export type AgentNodeType = Node<AgentNodeData, 'agent'>

// ── Platform Node (MCP / Hook) ──
export interface PlatformNodeData {
  nodeKind: 'mcp' | 'hook'
  title: string
  detail: string
  [key: string]: unknown
}
export type PlatformNodeType = Node<PlatformNodeData, 'platform'>

// ── Channel Edge ──
export type ChannelType = 'planning' | 'dispatch' | 'output' | 'gate' | 'blocked'
export interface ChannelEdgeData {
  channelType: ChannelType
  label?: string
  [key: string]: unknown
}
export type ChannelEdgeType = Edge<ChannelEdgeData>

// ── Status colors ──
export const STATUS_COLORS: Record<string, string> = {
  actual: '#10b981',
  'in-flight': '#f59e0b',
  planned: '#475569',
  blocked: '#ef4444',
}

// ── Channel style config ──
export const CHANNEL_STYLES: Record<ChannelType, { color: string; dashed: boolean }> = {
  planning: { color: '#7c3aed', dashed: true },
  dispatch: { color: '#f59e0b', dashed: false },
  output: { color: '#10b981', dashed: true },
  gate: { color: '#3b82f6', dashed: false },
  blocked: { color: '#ef4444', dashed: true },
}

// ── Agent type style config ──
export const AGENT_STYLES: Record<
  AgentNodeData['agentType'],
  { stroke: string; fill: string; dashed: boolean; glow: boolean }
> = {
  human: { stroke: '#a855f7', fill: '#0d1117', dashed: true, glow: false },
  commander: { stroke: '#f59e0b', fill: '#0d1117', dashed: false, glow: true },
  specialist: { stroke: '#3b82f6', fill: '#0d1117', dashed: false, glow: false },
  gate: { stroke: '#10b981', fill: '#0d1117', dashed: false, glow: false },
}

// ── Preset data shape ──
export interface WorkflowPreset {
  nodes: Node[]
  edges: Edge[]
}
