import type { Meta, StoryObj } from '@storybook/react'
import { AgentHeader } from './AgentHeader'
import type { ResolvedAgentProfile } from '../types'

const meta: Meta<typeof AgentHeader> = {
  title: 'Agents/AgentHeader',
  component: AgentHeader,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof AgentHeader>

const baseProfile: ResolvedAgentProfile = {
  profile: {
    id: 'web-lane',
    name: 'web-lane',
    description: 'Web lane specialist for apps/web/. Active context for web feature work.',
    providers: ['claude', 'gemini'],
    version: 'v0.1.0',
  },
  skills: [
    { id: 'ship-coordination', name: 'ship-coordination', content: '', source: 'custom', vars: {} },
    { id: 'code-review', name: 'code-review', content: '', source: 'community', vars: {} },
  ],
  mcpServers: [
    { name: 'ship', command: 'ship', args: ['mcp', 'serve'], server_type: 'stdio', url: null, timeout_secs: null, codex_enabled_tools: [], codex_disabled_tools: [], gemini_include_tools: [], gemini_exclude_tools: [] },
  ],
  permissions: { preset: 'ship-guarded' },
  hooks: [],
  rules: [],
}

/** Full-featured header with edit button and multiple providers. */
export const Default: Story = {
  args: {
    profile: baseProfile,
    onEdit: () => {},
  },
}

/** Header without an edit handler -- the pencil icon should not render. */
export const ReadOnly: Story = {
  args: {
    profile: baseProfile,
  },
}

/** Single provider: Claude only. */
export const ClaudeOnly: Story = {
  args: {
    profile: {
      ...baseProfile,
      profile: {
        ...baseProfile.profile,
        name: 'backend-rust',
        description: 'Rust runtime specialist. Handles crates/, runtime, and CLI transport layers.',
        providers: ['claude'],
        version: 'v0.3.1',
      },
      skills: [baseProfile.skills[0]],
      mcpServers: [baseProfile.mcpServers[0]],
    },
    onEdit: () => {},
  },
}

/** All four providers attached. */
export const AllProviders: Story = {
  args: {
    profile: {
      ...baseProfile,
      profile: {
        ...baseProfile.profile,
        name: 'full-stack',
        description: 'Cross-provider agent targeting every supported coding assistant.',
        providers: ['claude', 'gemini', 'codex', 'cursor'],
        version: 'v1.0.0',
      },
    },
    onEdit: () => {},
  },
}

/** Agent with no description. */
export const NoDescription: Story = {
  args: {
    profile: {
      ...baseProfile,
      profile: {
        ...baseProfile.profile,
        name: 'minimal-agent',
        description: '',
        version: 'v0.0.1',
        providers: ['gemini'],
      },
      skills: [],
      mcpServers: [],
    },
  },
}

/** Very long name and description to test truncation behavior. */
export const LongContent: Story = {
  args: {
    profile: {
      ...baseProfile,
      profile: {
        ...baseProfile.profile,
        name: 'extremely-long-agent-name-that-should-handle-overflow-gracefully',
        description:
          'This agent has a very detailed description that explains every aspect of its purpose, including edge cases, boundary conditions, integration points, and fallback strategies for when things go wrong in production environments.',
        providers: ['claude', 'gemini', 'codex'],
      },
    },
    onEdit: () => {},
  },
}
