import type { Meta, StoryObj } from '@storybook/react'
import { ConnectGitHub } from './ConnectGitHub'

const meta: Meta<typeof ConnectGitHub> = {
  title: 'GitHub/ConnectGitHub',
  component: ConnectGitHub,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof ConnectGitHub>

/** Card variant -- the default, prominent connect prompt. */
export const Card: Story = {
  args: {
    variant: 'card',
  },
}

/** Inline variant -- compact row used inside other panels. */
export const Inline: Story = {
  args: {
    variant: 'inline',
  },
}
