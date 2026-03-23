import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { RulesSection } from './RulesSection'
import type { Rule } from '@ship/ui'

const meta: Meta<typeof RulesSection> = {
  title: 'Agents/RulesSection',
  component: RulesSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof RulesSection>

const demoRules: Rule[] = [
  { file_name: '010-no-compat.md', content: 'No backward compatibility without consumers. Make hard breaks in the same change.' },
  { file_name: '020-test-policy.md', content: 'Add or update tests for every bug fix and behavior change. Cover happy paths and meaningful failure paths.' },
]

/** Empty state -- no rules attached, only the add button. */
export const Empty: Story = {
  args: {
    rules: [],
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}

/** Single rule file. */
export const SingleRule: Story = {
  args: {
    rules: [demoRules[0]],
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}

/** Two rules -- the typical agent configuration. */
export const TwoRules: Story = {
  args: {
    rules: demoRules,
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}

/** Read-only mode -- no remove handler. */
export const ReadOnly: Story = {
  args: {
    rules: demoRules,
    onEdit: fn(),
  },
}

/** Many rules to test list rendering. */
export const ManyRules: Story = {
  args: {
    rules: [
      ...demoRules,
      { file_name: '030-react-patterns.md', content: 'Use functional components with hooks. Prefer composition over inheritance.' },
      { file_name: '040-rust-style.md', content: 'Prefer Result over panic. Use thiserror for error types.' },
      { file_name: '050-commit-style.md', content: 'Stage explicit files only. Keep commit subjects imperative and concise.' },
    ],
    onAdd: fn(),
    onEdit: fn(),
    onRemove: fn(),
  },
}
