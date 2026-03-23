import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { RuleEditorDialog } from './RuleEditorDialog'

const meta: Meta<typeof RuleEditorDialog> = {
  title: 'Agents/Dialogs/RuleEditorDialog',
  component: RuleEditorDialog,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof RuleEditorDialog>

/** Creating a new rule -- all fields empty, no delete button. */
export const CreateNew: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    rule: null,
    onSave: fn(),
  },
}

/** Editing an existing always-apply rule with content. */
export const EditAlwaysApply: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    rule: {
      file_name: '010-no-compat.md',
      content: 'No backward compatibility without consumers. Make hard breaks in the same change.\n\nDo not keep compatibility aliases, duplicate command surfaces, or transitional wrappers.',
      always_apply: true,
    },
    onSave: fn(),
    onDelete: fn(),
  },
}

/** Editing a conditional rule with file patterns. */
export const EditConditionalGlobs: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    rule: {
      file_name: '030-react-patterns.md',
      content: 'Use functional components with hooks. Prefer composition over inheritance.\nAvoid prop drilling -- use context or state management.',
      always_apply: false,
      globs: ['src/**/*.tsx', 'src/**/*.jsx'],
    },
    onSave: fn(),
    onDelete: fn(),
  },
}

/** Editing a rule with long content to test textarea scrolling. */
export const LongContent: Story = {
  args: {
    open: true,
    onOpenChange: fn(),
    rule: {
      file_name: '020-test-policy.md',
      content: `Add or update tests for every bug fix and behavior change.
Cover happy paths and meaningful failure paths.
Keep error messages actionable and specific.
Avoid silent fallbacks that hide broken state.
Keep command behavior idempotent where practical.
Keep Rust domain logic in runtime/modules and keep CLI/UI transport thin.
Keep React component state and API contracts explicit and stable.
Review changes for regressions, architecture drift, and missing tests before merge.
Stage explicit files only; keep commit subjects imperative and concise.
Use commit types consistently: feat, fix, refactor, test, docs, chore.`,
      always_apply: true,
    },
    onSave: fn(),
    onDelete: fn(),
  },
}
