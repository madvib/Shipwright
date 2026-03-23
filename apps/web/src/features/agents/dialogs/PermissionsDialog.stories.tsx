import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { PermissionsDialog } from './PermissionsDialog'
import type { ProfilePermissions } from '@ship/ui'

const meta: Meta<typeof PermissionsDialog> = {
  title: 'Agents/Dialogs/PermissionsDialog',
  component: PermissionsDialog,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof PermissionsDialog>

const fullPermissions: ProfilePermissions = {
  preset: 'ship-guarded',
  tools_allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'],
  tools_deny: ['Bash(rm -rf *)'],
}

/** Fully populated permissions. */
export const FullPermissions: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    permissions: fullPermissions,
    onSave: fn(),
  },
}

/** Empty permissions -- all fields start blank. */
export const EmptyPermissions: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    permissions: {},
    onSave: fn(),
  },
}

/** Permissions with only tools_allow configured. */
export const PartialPermissions: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    permissions: {
      tools_allow: ['Read', 'Grep'],
      tools_deny: [],
    },
    onSave: fn(),
  },
}
