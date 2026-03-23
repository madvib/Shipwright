import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { HookEditorDialog } from './HookEditorDialog'
import type { HookConfig } from '@ship/ui'

const meta: Meta<typeof HookEditorDialog> = {
  title: 'Agents/Dialogs/HookEditorDialog',
  component: HookEditorDialog,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof HookEditorDialog>

const demoHook: HookConfig = {
  id: 'hook-1',
  trigger: 'PreToolUse',
  command: './scripts/check-no-compat-surface.sh',
  matcher: 'Edit|Write',
}

/** Creating a new hook -- all fields empty, no delete button. */
export const CreateNew: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    hook: null,
    onSave: fn(),
  },
}

/** Editing an existing hook with all fields populated. */
export const EditExisting: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    hook: demoHook,
    onSave: fn(),
    onDelete: fn(),
  },
}

/** Editing a hook with no matcher pattern set. */
export const EditWithoutMatcher: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    hook: {
      id: 'hook-2',
      trigger: 'Stop',
      command: 'ship mcp sync-permissions',
    },
    onSave: fn(),
    onDelete: fn(),
  },
}

/** Editing a hook with a long command. */
export const AllProviders: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    hook: {
      id: 'hook-3',
      trigger: 'Notification',
      command: "notify-send 'Agent' '$MESSAGE'",
      matcher: '',
    },
    onSave: fn(),
    onDelete: fn(),
  },
}
