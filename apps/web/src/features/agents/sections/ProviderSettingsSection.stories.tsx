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

/** Claude only with some settings pre-filled. */
export const ClaudeOnly: Story = {
  args: {
    providers: ['claude'],
    providerSettings: {
      claude: {
        theme: 'dark',
        auto_updates: true,
        include_co_authored_by: false,
      },
    },
    onChange: fn(),
  },
}

/** All four providers configured. */
export const AllProviders: Story = {
  args: {
    providers: ['claude', 'gemini', 'codex', 'cursor'],
    providerSettings: {
      claude: { theme: 'auto', auto_updates: true },
      gemini: { default_approval_mode: 'auto-approve-reads', max_session_turns: 50 },
      codex: { approval_policy: 'unless-allow-listed', sandbox: 'docker', reasoning_effort: 'high' },
      cursor: { environment: { EDITOR: 'cursor', NODE_ENV: 'development' } },
    },
    onChange: fn(),
  },
}

/** No providers -- the component returns null and renders nothing. */
export const NoProviders: Story = {
  args: {
    providers: [],
    providerSettings: {},
    onChange: fn(),
  },
}

/** Two providers with empty settings -- shows defaults. */
export const TwoProvidersDefaults: Story = {
  args: {
    providers: ['gemini', 'codex'],
    providerSettings: {},
    onChange: fn(),
  },
}
