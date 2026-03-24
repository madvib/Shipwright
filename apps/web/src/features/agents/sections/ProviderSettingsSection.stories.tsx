import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { ProviderSettingsSection } from './ProviderSettingsSection'

const meta: Meta<typeof ProviderSettingsSection> = {
  title: 'Agents/ProviderSettingsSection',
  component: ProviderSettingsSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof ProviderSettingsSection>

/** All five providers with defaults. */
export const AllProviders: Story = {
  args: {
    providers: ['claude', 'gemini', 'codex', 'cursor', 'opencode'],
    providerSettings: {},
    onChange: fn(),
  },
}

/** No providers -- the component returns null. */
export const NoProviders: Story = {
  args: {
    providers: [],
    providerSettings: {},
    onChange: fn(),
  },
}

/** Two providers with custom settings. */
export const WithOverrides: Story = {
  args: {
    providers: ['claude', 'codex'],
    providerSettings: {
      claude: { theme: 'dark', auto_updates: true },
      codex: { approval_policy: 'auto-approve', sandbox: 'docker' },
    },
    onChange: fn(),
  },
}
