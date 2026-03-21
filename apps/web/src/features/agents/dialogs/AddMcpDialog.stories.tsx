import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { AddMcpDialog } from './AddMcpDialog'

const meta: Meta<typeof AddMcpDialog> = {
  title: 'Agents/Dialogs/AddMcpDialog',
  component: AddMcpDialog,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof AddMcpDialog>

/** Browse mode -- popular servers available, none already added. */
export const BrowseMode: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    existingNames: [],
    onAdd: fn(),
  },
}

/** Browse mode with some servers already added. */
export const BrowsePartiallyUsed: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    existingNames: ['github', 'filesystem'],
    onAdd: fn(),
  },
}

/** Browse mode with all popular servers already added. */
export const BrowseAllUsed: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    existingNames: ['github', 'filesystem', 'playwright', 'postgres'],
    onAdd: fn(),
  },
}
