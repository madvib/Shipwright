import type { Meta, StoryObj } from '@storybook/react'
import { ProviderLogo } from './ProviderLogo'

const meta: Meta<typeof ProviderLogo> = {
  title: 'Compiler/ProviderLogo',
  component: ProviderLogo,
  parameters: { layout: 'centered' },
  argTypes: {
    provider: {
      control: 'select',
      options: ['claude', 'gemini', 'codex', 'cursor', 'unknown-provider'],
    },
    size: {
      control: 'radio',
      options: ['sm', 'md', 'lg'],
    },
  },
}
export default meta
type Story = StoryObj<typeof ProviderLogo>

/** Claude provider logo. */
export const Claude: Story = { args: { provider: 'claude', size: 'lg' } }

/** Gemini provider logo. */
export const Gemini: Story = { args: { provider: 'gemini', size: 'lg' } }

/** Codex provider logo (OpenAI with dark:invert). */
export const Codex: Story = { args: { provider: 'codex', size: 'lg' } }

/** Cursor provider logo (light/dark swap). */
export const Cursor: Story = { args: { provider: 'cursor', size: 'lg' } }

/** Unknown provider -- falls back to colored initials. */
export const Unknown: Story = { args: { provider: 'windsurf', size: 'lg' } }

/** All providers at small size in a row. */
export const AllSmall: Story = {
  render: () => (
    <div className="flex items-center gap-4">
      <ProviderLogo provider="claude" size="sm" />
      <ProviderLogo provider="gemini" size="sm" />
      <ProviderLogo provider="codex" size="sm" />
      <ProviderLogo provider="cursor" size="sm" />
    </div>
  ),
}

/** All providers at medium size in a row. */
export const AllMedium: Story = {
  render: () => (
    <div className="flex items-center gap-4">
      <ProviderLogo provider="claude" size="md" />
      <ProviderLogo provider="gemini" size="md" />
      <ProviderLogo provider="codex" size="md" />
      <ProviderLogo provider="cursor" size="md" />
    </div>
  ),
}
