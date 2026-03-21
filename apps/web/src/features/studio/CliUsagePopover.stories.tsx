import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { CliUsagePopover } from './CliUsagePopover'

const meta: Meta<typeof CliUsagePopover> = {
  title: 'Studio/CliUsagePopover',
  component: CliUsagePopover,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof CliUsagePopover>

/** With an agent name -- shows both "ship use <name>" and "ship install <name>". */
export const WithAgentName: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    agentName: 'web-lane',
  },
}

/** Without an agent name -- shows only the generic "ship use" command. */
export const WithoutAgentName: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
  },
}
