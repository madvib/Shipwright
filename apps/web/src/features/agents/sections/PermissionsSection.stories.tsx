import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { PermissionsSection } from './PermissionsSection'
import type { Permissions } from '@ship/ui'

const meta: Meta<typeof PermissionsSection> = {
  title: 'Agents/PermissionsSection',
  component: PermissionsSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof PermissionsSection>

const fullPermissions: Permissions = {
  tools: { allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'], deny: ['Bash(rm -rf *)'] },
  filesystem: { allow: ['apps/web/**', 'packages/**'], deny: ['.env', 'credentials.*'] },
  commands: { allow: ['git status', 'pnpm *', 'vitest'], deny: ['git push --force'] },
  network: { policy: 'allow-list', allow_hosts: ['localhost', 'api.github.com', 'registry.npmjs.org'] },
  agent: { require_confirmation: [] },
}

const emptyPermissions: Permissions = {
  tools: { allow: [], deny: [] },
  filesystem: { allow: [], deny: [] },
  commands: { allow: [], deny: [] },
  network: { policy: 'allow-list', allow_hosts: [] },
  agent: { require_confirmation: [] },
}

/** Ship-guarded preset with realistic allow/deny rules. */
export const ShipGuarded: Story = {
  args: {
    permissions: fullPermissions,
    activePreset: 'ship-guarded',
    maxTurns: 25,
    onPresetChange: fn(),
    onMaxTurnsChange: fn(),
    onEdit: fn(),
  },
}

/** Locked-down preset with empty rules. */
export const LockedDown: Story = {
  args: {
    permissions: emptyPermissions,
    activePreset: 'locked-down',
    onPresetChange: fn(),
    onMaxTurnsChange: fn(),
    onEdit: fn(),
  },
}

/** Open preset with broad permissions. */
export const Open: Story = {
  args: {
    permissions: {
      tools: { allow: ['*'], deny: [] },
      filesystem: { allow: ['**'], deny: [] },
      commands: { allow: ['*'], deny: [] },
      network: { policy: 'allow-list', allow_hosts: ['*'] },
      agent: { require_confirmation: [] },
    },
    activePreset: 'open',
    maxTurns: undefined,
    onPresetChange: fn(),
    onMaxTurnsChange: fn(),
  },
}

/** Custom preset with specific max turns value. */
export const CustomWithMaxTurns: Story = {
  args: {
    permissions: fullPermissions,
    activePreset: 'custom',
    maxTurns: 10,
    onPresetChange: fn(),
    onMaxTurnsChange: fn(),
    onEdit: fn(),
  },
}
