import type { WorkflowPreset } from '../types'

export const shipflow: WorkflowPreset = {
  nodes: [
    // D0 — Target
    {
      id: 'target-1',
      type: 'artifact',
      position: { x: 540, y: 30 },
      data: { label: 'v0.1.0 — Studio Launch', depth: 0, subtitle: '4 capabilities · 2 in-flight · 1 actual', status: 'in-flight' },
    },
    // Human
    {
      id: 'human',
      type: 'agent',
      position: { x: 80, y: 210 },
      data: { name: 'Human', profile: 'Product Owner', agentType: 'human', icon: '👤', badge: 'HUMAN' },
    },
    // Commander
    {
      id: 'commander',
      type: 'agent',
      position: { x: 310, y: 200 },
      data: { name: 'Commander', profile: 'commander', agentType: 'commander', icon: '⚡', badge: 'COMMANDER', detail: 'ship · orchestration skills' },
    },
    // D1 — Capabilities
    {
      id: 'cap-profile-editor',
      type: 'artifact',
      position: { x: 570, y: 195 },
      data: { label: 'Profile Editor', depth: 1, subtitle: 'Studio · MVP', status: 'actual', accentColor: '#3b82f6' },
    },
    {
      id: 'cap-workflow-canvas',
      type: 'artifact',
      position: { x: 570, y: 278 },
      data: { label: 'Workflow Canvas', depth: 1, subtitle: 'Studio · Core', status: 'in-flight', accentColor: '#3b82f6' },
    },
    {
      id: 'cap-github-import',
      type: 'artifact',
      position: { x: 810, y: 195 },
      data: { label: 'GitHub Import', depth: 1, subtitle: 'Studio · Import', status: 'actual', accentColor: '#10b981' },
    },
    {
      id: 'cap-export',
      type: 'artifact',
      position: { x: 810, y: 278 },
      data: { label: 'Export + CLI Sync', depth: 1, subtitle: 'Studio · Delivery', status: 'planned' },
    },
    // Specialists
    {
      id: 'planner',
      type: 'agent',
      position: { x: 80, y: 460 },
      data: { name: 'Planner', profile: 'default', agentType: 'specialist', icon: '🗺', detail: 'specs · ADRs · notes' },
    },
    {
      id: 'dispatcher',
      type: 'agent',
      position: { x: 280, y: 460 },
      data: { name: 'Dispatcher', profile: 'default', agentType: 'specialist', icon: '📡', detail: 'worktrees · routing' },
    },
    {
      id: 'web-agent',
      type: 'agent',
      position: { x: 490, y: 460 },
      data: { name: 'Web Agent', profile: 'web-lane', agentType: 'specialist', icon: '🌐', detail: 'React · TanStack · CF' },
    },
    {
      id: 'rust-agent',
      type: 'agent',
      position: { x: 700, y: 460 },
      data: { name: 'Rust Agent', profile: 'rust-runtime', agentType: 'specialist', icon: '🦀', detail: 'runtime · DB · MCP' },
    },
    {
      id: 'gate',
      type: 'agent',
      position: { x: 910, y: 460 },
      data: { name: 'Gate Reviewer', profile: 'default', agentType: 'gate', icon: '🔬', detail: 'acceptance · diff review' },
    },
    // D2 — Jobs
    {
      id: 'job-canvas-ui',
      type: 'artifact',
      position: { x: 490, y: 590 },
      data: { label: 'MBXHTbSE · Canvas UI', depth: 2, status: 'in-flight' },
    },
    {
      id: 'job-studio-profiles',
      type: 'artifact',
      position: { x: 490, y: 626 },
      data: { label: 'eMnTJr2f · Studio Profiles', depth: 2, status: 'actual' },
    },
    {
      id: 'job-node-edge',
      type: 'artifact',
      position: { x: 700, y: 590 },
      data: { label: 'in8LGBba · Node/Edge Runtime', depth: 2, status: 'in-flight' },
    },
    {
      id: 'job-export-sync',
      type: 'artifact',
      position: { x: 700, y: 626 },
      data: { label: 'pending · Export sync', depth: 2, status: 'planned' },
    },
    // Platform
    {
      id: 'ship-mcp',
      type: 'platform',
      position: { x: 1120, y: 200 },
      data: { nodeKind: 'mcp', title: 'ship-mcp', detail: 'tools: 24 · jobs, caps, sessions' },
    },
    {
      id: 'github-mcp',
      type: 'platform',
      position: { x: 1120, y: 288 },
      data: { nodeKind: 'mcp', title: 'github-mcp', detail: 'tools: 8 · PRs, issues, files' },
    },
    {
      id: 'hook-post-tool',
      type: 'platform',
      position: { x: 1120, y: 376 },
      data: { nodeKind: 'hook', title: 'Hook · PostToolUse', detail: 'append_job_log' },
    },
    {
      id: 'hook-pre-compact',
      type: 'platform',
      position: { x: 1120, y: 452 },
      data: { nodeKind: 'hook', title: 'Hook · PreCompact', detail: 'log_progress' },
    },
  ],
  edges: [
    // Human → Commander: planning
    { id: 'e-human-cmd', source: 'human', target: 'commander', type: 'channel', data: { channelType: 'planning', label: 'planning' } },
    // Commander → Capabilities: dispatch
    { id: 'e-cmd-cap1', source: 'commander', target: 'cap-profile-editor', type: 'channel', data: { channelType: 'dispatch' } },
    { id: 'e-cmd-cap2', source: 'commander', target: 'cap-workflow-canvas', type: 'channel', data: { channelType: 'dispatch' } },
    // Commander → Planner: dispatch
    { id: 'e-cmd-planner', source: 'commander', target: 'planner', type: 'channel', data: { channelType: 'dispatch', label: 'dispatch' } },
    // Commander → Dispatcher: dispatch
    { id: 'e-cmd-dispatcher', source: 'commander', target: 'dispatcher', type: 'channel', data: { channelType: 'dispatch' } },
    // Dispatcher → Workers: dispatch
    { id: 'e-disp-web', source: 'dispatcher', target: 'web-agent', type: 'channel', data: { channelType: 'dispatch' } },
    { id: 'e-disp-rust', source: 'dispatcher', target: 'rust-agent', type: 'channel', data: { channelType: 'dispatch' } },
    // Workers → Jobs: dispatch
    { id: 'e-web-job1', source: 'web-agent', target: 'job-canvas-ui', type: 'channel', data: { channelType: 'dispatch' } },
    { id: 'e-rust-job1', source: 'rust-agent', target: 'job-node-edge', type: 'channel', data: { channelType: 'dispatch' } },
    // Workers → Gate: output
    { id: 'e-web-gate', source: 'web-agent', target: 'gate', type: 'channel', data: { channelType: 'output', label: 'output' } },
    // Gate → Commander: gate-pass
    { id: 'e-gate-cmd', source: 'gate', target: 'commander', type: 'channel', data: { channelType: 'gate', label: 'gate-pass' } },
  ],
}
