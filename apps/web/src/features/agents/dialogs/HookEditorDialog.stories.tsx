import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { HookEditorDialog } from './HookEditorDialog'
import type { HookConfig } from '../types'

const meta: Meta<typeof HookEditorDialog> = {
  title: 'Agents/Dialogs/HookEditorDialog',
  component: HookEditorDialog,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof HookEditorDialog>

const demoHook: HookConfig = {
  trigger: 'PreToolUse',
  command: './scripts/check-no-compat-surface.sh',
  providers: ['claude', 'gemini'],
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
      trigger: 'Stop',
      command: 'ship mcp sync-permissions',
      providers: ['claude'],
    },
    onSave: fn(),
    onDelete: fn(),
  },
}

/** Editing a hook with all providers selected. */
export const AllProviders: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    hook: {
      trigger: 'Notification',
      command: "notify-send 'Agent' '$MESSAGE'",
      providers: ['claude', 'gemini', 'codex', 'cursor'],
      matcher: '',
    },
    onSave: fn(),
    onDelete: fn(),
  },
}
