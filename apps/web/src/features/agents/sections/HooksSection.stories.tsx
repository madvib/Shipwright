import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { HooksSection } from './HooksSection'
import type { HookConfig } from '@ship/ui'

const meta: Meta<typeof HooksSection> = {
  title: 'Agents/HooksSection',
  component: HooksSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof HooksSection>

const demoHooks: HookConfig[] = [
  { id: 'hook-1', trigger: 'PreToolUse', command: './scripts/check-no-compat-surface.sh' },
  { id: 'hook-2', trigger: 'Stop', command: 'ship mcp sync-permissions' },
  { id: 'hook-3', trigger: 'Notification', command: "notify-send 'Agent' '$MESSAGE'" },
]

/** Empty state -- no hooks configured, only the add button. */
export const Empty: Story = {
  args: {
    hooks: [],
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}

/** Single hook with multiple providers. */
export const SingleHook: Story = {
  args: {
    hooks: [demoHooks[0]],
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}

/** Three hooks showing the typical agent configuration. */
export const ThreeHooks: Story = {
  args: {
    hooks: demoHooks,
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}

/** Read-only mode -- no edit or remove handlers. */
export const ReadOnly: Story = {
  args: {
    hooks: demoHooks,
    onAdd: fn(),
  },
}

/** Hook with a very long command string to test truncation. */
export const LongCommand: Story = {
  args: {
    hooks: [
      {
        id: 'hook-4',
        trigger: 'PreToolUse',
        command: './scripts/validate-all-the-things.sh --strict --no-cache --format=json --output=/tmp/validation-results.log 2>&1',
      },
    ],
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}
