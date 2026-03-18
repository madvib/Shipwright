import type { WorkflowPreset } from '../types'

export const gstack: WorkflowPreset = {
  nodes: [
    // Human
    {
      id: 'human',
      type: 'agent',
      position: { x: 20, y: 200 },
      data: { name: 'Human', profile: 'product owner', agentType: 'human', icon: '👤', badge: 'HUMAN' },
    },
    // D0 — Sprint
    {
      id: 'sprint',
      type: 'artifact',
      position: { x: 220, y: 30 },
      data: { label: 'Sprint 12 — Auth Rewrite', depth: 0, subtitle: '5 stages · 3 in-flight', status: 'in-flight' },
    },
    // Pipeline stages
    {
      id: 'spec-agent',
      type: 'agent',
      position: { x: 220, y: 200 },
      data: { name: 'Spec Writer', profile: 'default', agentType: 'specialist', icon: '📝', detail: 'requirements · acceptance criteria' },
    },
    {
      id: 'design-agent',
      type: 'agent',
      position: { x: 420, y: 200 },
      data: { name: 'Designer', profile: 'default', agentType: 'specialist', icon: '🎨', detail: 'API surface · data model' },
    },
    {
      id: 'impl-agent',
      type: 'agent',
      position: { x: 620, y: 200 },
      data: { name: 'Implementer', profile: 'default', agentType: 'specialist', icon: '🔨', detail: 'TDD · code generation' },
    },
    {
      id: 'review-agent',
      type: 'agent',
      position: { x: 820, y: 200 },
      data: { name: 'Reviewer', profile: 'default', agentType: 'gate', icon: '🔬', detail: 'diff review · acceptance' },
    },
    {
      id: 'deploy-agent',
      type: 'agent',
      position: { x: 1020, y: 200 },
      data: { name: 'Deployer', profile: 'default', agentType: 'specialist', icon: '🚀', detail: 'CI/CD · staging · prod' },
    },
    // D1 artifacts
    {
      id: 'spec-output',
      type: 'artifact',
      position: { x: 220, y: 360 },
      data: { label: 'Spec Document', depth: 1, subtitle: 'requirements', status: 'actual', accentColor: '#7c3aed' },
    },
    {
      id: 'design-output',
      type: 'artifact',
      position: { x: 420, y: 360 },
      data: { label: 'Design Doc', depth: 1, subtitle: 'API + schema', status: 'in-flight', accentColor: '#3b82f6' },
    },
    {
      id: 'impl-output',
      type: 'artifact',
      position: { x: 620, y: 360 },
      data: { label: 'Pull Request', depth: 1, subtitle: 'code', status: 'planned', accentColor: '#10b981' },
    },
    // Platform
    {
      id: 'browser-mcp',
      type: 'platform',
      position: { x: 820, y: 360 },
      data: { nodeKind: 'mcp', title: 'browser-daemon', detail: 'tools: 6 · screenshots, DOM' },
    },
  ],
  edges: [
    // Human → Spec
    { id: 'e-human-spec', source: 'human', target: 'spec-agent', type: 'channel', data: { channelType: 'planning', label: 'idea' } },
    // Pipeline: spec → design → impl → review → deploy
    { id: 'e-spec-design', source: 'spec-agent', target: 'design-agent', type: 'channel', data: { channelType: 'dispatch', label: 'spec handoff' } },
    { id: 'e-design-impl', source: 'design-agent', target: 'impl-agent', type: 'channel', data: { channelType: 'dispatch', label: 'design handoff' } },
    { id: 'e-impl-review', source: 'impl-agent', target: 'review-agent', type: 'channel', data: { channelType: 'output', label: 'PR' } },
    { id: 'e-review-deploy', source: 'review-agent', target: 'deploy-agent', type: 'channel', data: { channelType: 'gate', label: 'approved' } },
    // Review reject loop
    { id: 'e-review-impl', source: 'review-agent', target: 'impl-agent', type: 'channel', data: { channelType: 'blocked', label: 'reject' } },
    // Agents → artifacts
    { id: 'e-spec-out', source: 'spec-agent', target: 'spec-output', type: 'channel', data: { channelType: 'output' } },
    { id: 'e-design-out', source: 'design-agent', target: 'design-output', type: 'channel', data: { channelType: 'output' } },
    { id: 'e-impl-out', source: 'impl-agent', target: 'impl-output', type: 'channel', data: { channelType: 'output' } },
  ],
}
