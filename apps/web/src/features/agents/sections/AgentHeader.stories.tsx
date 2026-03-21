import type { Meta, StoryObj } from '@storybook/react'
import { AgentHeader } from './AgentHeader'
import type { AgentProfile } from '../types'
import { DEMO_AGENT, DEFAULT_SETTINGS } from '../types'

const meta: Meta<typeof AgentHeader> = {
  title: 'Agents/AgentHeader',
  component: AgentHeader,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof AgentHeader>

const baseProfile: AgentProfile = { ...DEMO_AGENT }

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
      name: 'backend-rust',
      description: 'Rust runtime specialist. Handles crates/, runtime, and CLI transport layers.',
      providers: ['claude'],
      version: 'v0.3.1',
      skills: [baseProfile.skills[0]],
      mcpServers: [baseProfile.mcpServers[0]],
      subagents: [],
    },
    onEdit: () => {},
  },
}

/** All four providers attached. */
export const AllProviders: Story = {
  args: {
    profile: {
      ...baseProfile,
      name: 'full-stack',
      description: 'Cross-provider agent targeting every supported coding assistant.',
      providers: ['claude', 'gemini', 'codex', 'cursor'],
      version: 'v1.0.0',
    },
    onEdit: () => {},
  },
}

/** Agent with no description. */
export const NoDescription: Story = {
  args: {
    profile: {
      ...baseProfile,
      name: 'minimal-agent',
      description: '',
      providers: ['gemini'],
      version: 'v0.0.1',
      skills: [],
      mcpServers: [],
      subagents: [],
    },
  },
}

/** Very long name and description to test truncation behavior. */
export const LongContent: Story = {
  args: {
    profile: {
      ...baseProfile,
      name: 'extremely-long-agent-name-that-should-handle-overflow-gracefully',
      description:
        'This agent has a very detailed description that explains every aspect of its purpose, including edge cases, boundary conditions, integration points, and fallback strategies for when things go wrong in production environments.',
      providers: ['claude', 'gemini', 'codex'],
    },
    onEdit: () => {},
  },
}
