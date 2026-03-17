export interface WorkflowRole {
  id: string
  profile: string
  default?: boolean
  description?: string
}

export interface RoutingRule {
  from: string
  jobKind: string
  to: string
  gate: boolean
}

export interface DocKind {
  kind: string
  requiredFields: string[]
}

export const INITIAL_ROLES: WorkflowRole[] = [
  { id: 'commander', profile: 'commander', default: true },
  { id: 'web-lane', profile: 'web-lane' },
  { id: 'rust-compiler', profile: 'rust-compiler' },
]

export const INITIAL_ROUTING: RoutingRule[] = [
  { from: 'commander', jobKind: 'feature', to: 'web-lane', gate: true },
  { from: 'commander', jobKind: 'bug', to: 'web-lane', gate: true },
  { from: 'commander', jobKind: 'human-action', to: 'human', gate: false },
]

export const INITIAL_DOC_KINDS: DocKind[] = [
  { kind: 'spec', requiredFields: ['title', 'scope', 'acceptance_criteria'] },
  { kind: 'adr', requiredFields: ['context', 'decision', 'consequences'] },
  { kind: 'job', requiredFields: ['title', 'description', 'assigned_role'] },
]
