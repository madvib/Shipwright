import type { WorkflowPreset } from '../types'

export const superpowers: WorkflowPreset = {
  nodes: [
    // Human
    {
      id: 'human',
      type: 'agent',
      position: { x: 340, y: 20 },
      data: { name: 'Human', profile: 'idea source', agentType: 'human', icon: '👤', badge: 'HUMAN' },
    },
    // Phase agents
    {
      id: 'brainstormer',
      type: 'agent',
      position: { x: 60, y: 180 },
      data: { name: 'Brainstormer', profile: 'default', agentType: 'specialist', icon: '💡', detail: 'brainstorming skill' },
    },
    {
      id: 'planner',
      type: 'agent',
      position: { x: 380, y: 180 },
      data: { name: 'Planner', profile: 'default', agentType: 'specialist', icon: '📋', detail: 'writing-plans skill' },
    },
    {
      id: 'executor',
      type: 'agent',
      position: { x: 60, y: 400 },
      data: { name: 'Executor', profile: 'default', agentType: 'commander', icon: '⚡', badge: 'COMMANDER', detail: 'executing-plans · dispatches workers' },
    },
    {
      id: 'verifier',
      type: 'agent',
      position: { x: 380, y: 400 },
      data: { name: 'Verifier', profile: 'default', agentType: 'gate', icon: '🔬', detail: 'requesting-code-review' },
    },
    // Executor's workers
    {
      id: 'web-worker',
      type: 'agent',
      position: { x: 60, y: 580 },
      data: { name: 'Web Agent', profile: 'web-lane', agentType: 'specialist', icon: '🌐', detail: 'TDD · React' },
    },
    {
      id: 'rust-worker',
      type: 'agent',
      position: { x: 280, y: 580 },
      data: { name: 'Rust Agent', profile: 'rust-runtime', agentType: 'specialist', icon: '🦀', detail: 'TDD · runtime' },
    },
    // Artifacts
    {
      id: 'spec-output',
      type: 'artifact',
      position: { x: 60, y: 330 },
      data: { label: 'Spec + ADR', depth: 1, subtitle: 'brainstorm output', status: 'actual', accentColor: '#7c3aed' },
    },
    {
      id: 'plan-output',
      type: 'artifact',
      position: { x: 380, y: 330 },
      data: { label: 'Implementation Plan', depth: 1, subtitle: 'plan output', status: 'in-flight', accentColor: '#38bdf8' },
    },
    {
      id: 'verify-output',
      type: 'artifact',
      position: { x: 580, y: 500 },
      data: { label: 'Review Report', depth: 1, subtitle: 'gate output', status: 'planned', accentColor: '#10b981' },
    },
    // Platform
    {
      id: 'ship-mcp',
      type: 'platform',
      position: { x: 620, y: 180 },
      data: { nodeKind: 'mcp', title: 'ship mcp', detail: 'jobs, sessions, progress' },
    },
  ],
  edges: [
    // Human → Brainstormer
    { id: 'e-human-brain', source: 'human', target: 'brainstormer', type: 'channel', data: { channelType: 'planning', label: 'idea' } },
    // Brainstormer → spec output
    { id: 'e-brain-spec', source: 'brainstormer', target: 'spec-output', type: 'channel', data: { channelType: 'output' } },
    // Brainstormer → Planner
    { id: 'e-brain-plan', source: 'brainstormer', target: 'planner', type: 'channel', data: { channelType: 'dispatch', label: 'spec handoff' } },
    // Planner → plan output
    { id: 'e-plan-out', source: 'planner', target: 'plan-output', type: 'channel', data: { channelType: 'output' } },
    // Planner → Executor
    { id: 'e-plan-exec', source: 'planner', target: 'executor', type: 'channel', data: { channelType: 'dispatch', label: 'plan handoff' } },
    // Executor → workers
    { id: 'e-exec-web', source: 'executor', target: 'web-worker', type: 'channel', data: { channelType: 'dispatch' } },
    { id: 'e-exec-rust', source: 'executor', target: 'rust-worker', type: 'channel', data: { channelType: 'dispatch' } },
    // Workers → Verifier
    { id: 'e-web-verify', source: 'web-worker', target: 'verifier', type: 'channel', data: { channelType: 'output', label: 'output' } },
    { id: 'e-rust-verify', source: 'rust-worker', target: 'verifier', type: 'channel', data: { channelType: 'output' } },
    // Verifier → verify output
    { id: 'e-verify-out', source: 'verifier', target: 'verify-output', type: 'channel', data: { channelType: 'gate' } },
    // Verifier → Executor (reject loop)
    { id: 'e-verify-exec', source: 'verifier', target: 'executor', type: 'channel', data: { channelType: 'blocked', label: 'reject' } },
  ],
}
