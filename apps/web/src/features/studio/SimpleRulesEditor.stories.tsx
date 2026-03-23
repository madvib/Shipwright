import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { SimpleRulesEditor } from './SimpleRulesEditor'

const meta: Meta<typeof SimpleRulesEditor> = {
  title: 'Studio/SimpleRulesEditor',
  component: SimpleRulesEditor,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof SimpleRulesEditor>

/** Empty state -- no rules, shows placeholder text. */
export const Empty: Story = {
  args: {
    rules: [],
    onChange: fn(),
  },
}

/** Single rule entry. */
export const SingleRule: Story = {
  args: {
    rules: ['Always use TypeScript strict mode'],
    onChange: fn(),
  },
}

/** Multiple rules showing a typical configuration. */
export const MultipleRules: Story = {
  args: {
    rules: [
      'No backward compatibility without consumers',
      'Add tests for every bug fix',
      'Keep error messages actionable and specific',
      'Prefer explicit failures over silent fallback',
      'File length cap: 300 lines per file',
    ],
    onChange: fn(),
  },
}

/** Many rules to test scrolling and layout. */
export const ManyRules: Story = {
  args: {
    rules: [
      'Use TypeScript strict mode',
      'Prefer composition over inheritance',
      'Keep functions under 50 lines',
      'No inline styles in React',
      'Always use named exports',
      'Test happy paths and failure paths',
      'Use semantic HTML elements',
      'No console.log in production code',
      'Use CSS variables for theming',
      'Prefer immutable data patterns',
    ],
    onChange: fn(),
  },
}
