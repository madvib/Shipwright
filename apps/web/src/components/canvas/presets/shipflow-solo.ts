import type { WorkflowPreset } from '../types'

export const shipflowSolo: WorkflowPreset = {
  nodes: [
    {
      id: 'human',
      type: 'agent',
      position: { x: 80, y: 80 },
      data: { name: 'Human', profile: 'Product Owner', agentType: 'human', icon: '👤', badge: 'HUMAN' },
    },
    {
      id: 'commander',
      type: 'agent',
      position: { x: 340, y: 80 },
      data: { name: 'Commander', profile: 'commander', agentType: 'commander', icon: '⚡', badge: 'COMMANDER', detail: 'all ship skills loaded' },
    },
    {
      id: 'session-log',
      type: 'artifact',
      position: { x: 340, y: 260 },
      data: { label: 'Session Log', depth: 2, status: 'in-flight' },
    },
    {
      id: 'ship-mcp',
      type: 'platform',
      position: { x: 600, y: 80 },
      data: { nodeKind: 'mcp', title: 'ship mcp', detail: 'jobs, sessions, progress' },
    },
  ],
  edges: [
    { id: 'e-human-cmd', source: 'human', target: 'commander', type: 'channel', data: { channelType: 'planning', label: 'planning' } },
    { id: 'e-cmd-log', source: 'commander', target: 'session-log', type: 'channel', data: { channelType: 'output' } },
  ],
}
