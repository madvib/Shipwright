import type { WorkflowPreset } from '../types'

export const superpowersSolo: WorkflowPreset = {
  nodes: [
    {
      id: 'human',
      type: 'agent',
      position: { x: 80, y: 80 },
      data: { name: 'Human', profile: 'developer', agentType: 'human', icon: '👤', badge: 'HUMAN' },
    },
    {
      id: 'agent',
      type: 'agent',
      position: { x: 340, y: 80 },
      data: { name: 'Claude Code', profile: 'default', agentType: 'specialist', icon: '🤖', detail: 'brainstorming · planning · TDD · debugging' },
    },
    {
      id: 'output',
      type: 'artifact',
      position: { x: 340, y: 260 },
      data: { label: 'Working Software', depth: 1, subtitle: 'commits + tests', status: 'in-flight', accentColor: '#7c3aed' },
    },
  ],
  edges: [
    { id: 'e-human-agent', source: 'human', target: 'agent', type: 'channel', data: { channelType: 'planning', label: 'task' } },
    { id: 'e-agent-out', source: 'agent', target: 'output', type: 'channel', data: { channelType: 'output' } },
  ],
}
