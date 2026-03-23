import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { EditAgentDialog } from './EditAgentDialog'
import type { ResolvedAgentProfile } from '../types'

const meta: Meta<typeof EditAgentDialog> = {
  title: 'Agents/Dialogs/EditAgentDialog',
  component: EditAgentDialog,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof EditAgentDialog>

const demoProfile: ResolvedAgentProfile = {
  profile: {
    id: 'web-lane',
    name: 'web-lane',
    description: 'Web lane specialist for apps/web/.',
    providers: ['claude', 'gemini'],
    version: 'v0.1.0',
  },
  skills: [],
  mcpServers: [],
  permissions: { preset: 'ship-guarded' },
  hooks: [],
  rules: [],
}

/** Editing the demo agent profile with typical values. */
export const Default: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    profile: demoProfile,
    onSave: fn(),
  },
}

/** Editing a minimal agent with single provider and no description. */
export const MinimalAgent: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    profile: {
      ...demoProfile,
      profile: { ...demoProfile.profile, name: 'qa-runner', description: '', providers: ['codex'], version: 'v0.0.1' },
    },
    onSave: fn(),
  },
}

/** Editing an agent with all providers selected. */
export const AllProviders: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    profile: {
      ...demoProfile,
      profile: { ...demoProfile.profile, name: 'full-stack', description: 'Cross-provider agent.', providers: ['claude', 'gemini', 'codex', 'cursor'] },
    },
    onSave: fn(),
  },
}

/** Agent with a very long name and description to test overflow. */
export const LongContent: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    profile: {
      ...demoProfile,
      profile: {
        ...demoProfile.profile,
        name: 'extremely-long-agent-name-that-should-wrap-or-truncate',
        description: 'This agent handles complex cross-cutting concerns including code review, refactoring, and more.',
        providers: ['claude', 'gemini'],
      },
    },
    onSave: fn(),
  },
}
