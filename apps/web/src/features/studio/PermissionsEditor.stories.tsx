import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { PermissionsEditor } from './PermissionsEditor'
import { DEFAULT_PERMISSIONS } from '#/features/compiler/types'

const meta: Meta<typeof PermissionsEditor> = {
  title: 'Studio/PermissionsEditor',
  component: PermissionsEditor,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof PermissionsEditor>

/** Default preset selected with standard permissions. */
export const DefaultPreset: Story = {
  args: {
    permissions: DEFAULT_PERMISSIONS,
    onChange: fn(),
  },
}

/** Strict preset -- everything locked down. */
export const StrictPreset: Story = {
  args: {
    permissions: {
      tools: { allow: [], deny: [] },
      filesystem: { allow: [], deny: ['**/*'] },
      commands: { allow: [], deny: [] },
      network: { policy: 'none', allow_hosts: [] },
      agent: { require_confirmation: ['*'] },
    },
    onChange: fn(),
  },
}

/** Permissive preset -- everything open. */
export const PermissivePreset: Story = {
  args: {
    permissions: {
      tools: { allow: ['*'], deny: [] },
      filesystem: { allow: ['**/*'], deny: [] },
      commands: { allow: ['*'], deny: [] },
      network: { policy: 'unrestricted', allow_hosts: [] },
      agent: { require_confirmation: [] },
    },
    onChange: fn(),
  },
}

/** Custom permissions with both allow and deny rules. */
export const CustomRules: Story = {
  args: {
    permissions: {
      tools: {
        allow: ['Read', 'Grep', 'Glob', 'Bash(git *)'],
        deny: ['Bash(rm -rf *)', 'Bash(sudo *)'],
      },
      filesystem: { allow: ['apps/**'], deny: ['.env'] },
      commands: { allow: ['git status'], deny: [] },
      network: { policy: 'allow-list', allow_hosts: ['localhost'] },
      agent: { require_confirmation: [] },
    },
    onChange: fn(),
  },
}
