import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { PermissionsSection } from './PermissionsSection'
import type { ProfilePermissions } from '@ship/ui'

const meta: Meta<typeof PermissionsSection> = {
  title: 'Agents/PermissionsSection',
  component: PermissionsSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof PermissionsSection>

const fullPermissions: ProfilePermissions = {
  preset: 'ship-guarded',
  tools_allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'],
  tools_deny: ['Bash(rm -rf *)'],
}

const emptyPermissions: ProfilePermissions = {
  tools_allow: [],
  tools_deny: [],
}

/** Ship-guarded preset with realistic allow/deny rules. */
export const ShipGuarded: Story = {
  args: {
    permissions: fullPermissions,
    activePreset: 'ship-guarded',
    onPresetChange: fn(),
    onEdit: fn(),
  },
}

/** Locked-down preset with empty rules. */
export const LockedDown: Story = {
  args: {
    permissions: emptyPermissions,
    activePreset: 'locked-down',
    onPresetChange: fn(),
    onEdit: fn(),
  },
}

/** Open preset with broad permissions. */
export const Open: Story = {
  args: {
    permissions: {
      tools_allow: ['*'],
      tools_deny: [],
    },
    activePreset: 'open',
    onPresetChange: fn(),
  },
}

/** Custom preset. */
export const CustomPreset: Story = {
  args: {
    permissions: fullPermissions,
    activePreset: 'custom',
    onPresetChange: fn(),
    onEdit: fn(),
  },
}
