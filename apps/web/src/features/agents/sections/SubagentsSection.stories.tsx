import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { SubagentsSection } from './SubagentsSection'
import type { SubagentRef } from '../types'

const meta: Meta<typeof SubagentsSection> = {
  title: 'Agents/SubagentsSection',
  component: SubagentsSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof SubagentsSection>

const demoSubagents: SubagentRef[] = [
  { id: 'react-designer', name: 'react-designer', description: 'UI components, shadcn, Tailwind' },
  { id: 'react-architect', name: 'react-architect', description: 'Data flow, TanStack, state' },
]

/** Empty state -- no subagents attached. */
export const Empty: Story = {
  args: {
    subagents: [],
    onRemove: fn(),
    onAdd: fn(),
  },
}

/** Single subagent chip. */
export const SingleSubagent: Story = {
  args: {
    subagents: [demoSubagents[0]],
    onRemove: fn(),
    onAdd: fn(),
  },
}

/** Two subagents -- the typical configuration. */
export const TwoSubagents: Story = {
  args: {
    subagents: demoSubagents,
    onRemove: fn(),
    onAdd: fn(),
  },
}

/** Four subagents to test chip wrapping behavior. */
export const FourSubagents: Story = {
  args: {
    subagents: [
      ...demoSubagents,
      { id: 'qa-engineer', name: 'qa-engineer', description: 'Testing, coverage, CI pipelines' },
      { id: 'devops-lane', name: 'devops-lane', description: 'Infrastructure, deploys, monitoring' },
    ],
    onRemove: fn(),
    onAdd: fn(),
  },
}

/** Without an add handler -- the add chip still renders. */
export const NoAddHandler: Story = {
  args: {
    subagents: demoSubagents,
    onRemove: fn(),
  },
}
