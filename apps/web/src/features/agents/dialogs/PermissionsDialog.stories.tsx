import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { PermissionsDialog } from './PermissionsDialog'
import type { Permissions } from '@ship/ui'

const meta: Meta<typeof PermissionsDialog> = {
  title: 'Agents/Dialogs/PermissionsDialog',
  component: PermissionsDialog,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof PermissionsDialog>

const fullPermissions: Permissions = {
  tools: {
    allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'],
    deny: ['Bash(rm -rf *)'],
  },
  filesystem: {
    allow: ['apps/web/**', 'packages/primitives/**'],
    deny: ['.env', 'credentials.*', 'secrets/'],
  },
  commands: {
    allow: ['git status', 'pnpm *', 'npm test'],
    deny: ['git push --force', 'rm -rf /'],
  },
  network: {
    policy: 'allow-list',
    allow_hosts: ['localhost', 'api.github.com', 'registry.npmjs.org'],
  },
  agent: {
    require_confirmation: ['deploy', 'publish'],
  },
}

/** Fully populated permissions across all dimensions. */
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

/** Permissions with only tools and filesystem configured. */
export const PartialPermissions: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    permissions: {
      tools: { allow: ['Read', 'Grep'], deny: [] },
      filesystem: { allow: ['src/**'], deny: ['.env'] },
    },
    onSave: fn(),
  },
}

/** Network set to unrestricted with no host allowlist. */
export const UnrestrictedNetwork: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    permissions: {
      network: { policy: 'unrestricted', allow_hosts: [] },
    },
    onSave: fn(),
  },
}
