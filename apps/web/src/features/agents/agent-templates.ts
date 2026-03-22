// Agent templates derived from .ship/agents/templates/*.toml
// These are static TypeScript objects since TOML can't be read at browser runtime yet.

import type { AgentProfile } from './types'
import { DEFAULT_SETTINGS } from './types'
import { DEFAULT_PERMISSIONS } from '@ship/ui'

export interface AgentTemplate {
  id: string
  name: string
  description: string
  icon: string // lucide icon name
  color: string // tailwind color class
  providers: string[]
  permissionPreset: string
  rules: string
}

export const AGENT_TEMPLATES: AgentTemplate[] = [
  {
    id: 'orchestrator',
    name: 'Orchestrator',
    description:
      'Coordinates work across agents — routes jobs, manages state, doesn\'t write code directly',
    icon: 'Users',
    color: 'text-violet-500 bg-violet-500/10',
    providers: ['claude', 'gemini'],
    permissionPreset: 'ship-standard',
    rules:
      'You coordinate work across the team. Route tasks to the right agent.\n' +
      'Read project state and make decisions. Don\'t write code directly — create jobs.\n' +
      'When in doubt, ask rather than assume.',
  },
  {
    id: 'code-reviewer',
    name: 'Code Reviewer',
    description:
      'Reviews code for correctness, security, and architecture — read-only, no modifications',
    icon: 'Eye',
    color: 'text-blue-500 bg-blue-500/10',
    providers: ['claude'],
    permissionPreset: 'ship-readonly',
    rules:
      'You review code. You read, assess, and report — you do not write code.\n' +
      'Focus on: correctness, security (OWASP top 10), architecture consistency, test coverage.\n' +
      'Output PASS/FAIL with specific line references.\n' +
      'Keep suggestions focused on what changed, not pre-existing code.',
  },
  {
    id: 'frontend-dev',
    name: 'Frontend Dev',
    description:
      'UI components, styling, and accessibility — scoped to frontend files',
    icon: 'Palette',
    color: 'text-emerald-500 bg-emerald-500/10',
    providers: ['claude', 'cursor'],
    permissionPreset: 'ship-autonomous',
    rules:
      'Your domain is frontend code — components, styling, and user interactions.\n' +
      'Use the project\'s existing component library and design system.\n' +
      'Always consider mobile viewports and keyboard navigation.\n' +
      'Run tests before marking work done.',
  },
  {
    id: 'backend-dev',
    name: 'Backend Dev',
    description:
      'APIs, database, and server logic — full write access within backend scope',
    icon: 'Server',
    color: 'text-amber-500 bg-amber-500/10',
    providers: ['claude', 'codex'],
    permissionPreset: 'ship-autonomous',
    rules:
      'Your domain is server-side code — APIs, database, business logic.\n' +
      'Keep transport layers thin. Business logic belongs in modules, not handlers.\n' +
      'Require tests for behavior changes and bug fixes.\n' +
      'Migrations must be idempotent. Use explicit error handling, never silent fallbacks.',
  },
  {
    id: 'read-only-analyst',
    name: 'Read-Only Analyst',
    description:
      'Reads and analyzes code without any write access — safe for exploration and auditing',
    icon: 'Search',
    color: 'text-cyan-500 bg-cyan-500/10',
    providers: ['claude'],
    permissionPreset: 'ship-readonly',
    rules:
      'You analyze code and answer questions. You have no write access.\n' +
      'Be thorough in your analysis. Cite specific files and line numbers.\n' +
      'If asked to make changes, explain what you would change and why, but do not attempt writes.',
  },
]

/** Convert an AgentTemplate into the partial shape expected by createAgent(). */
export function templateToAgent(
  template: AgentTemplate,
  name: string,
): Partial<AgentProfile> {
  return {
    name,
    description: template.description,
    providers: [...template.providers],
    permissionPreset: template.permissionPreset,
    permissions: { ...DEFAULT_PERMISSIONS },
    settings: { ...DEFAULT_SETTINGS },
    rules: [
      {
        file_name: '001-role.md',
        content: template.rules,
      },
    ],
  }
}
