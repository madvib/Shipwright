import type { Meta, StoryObj } from '@storybook/react'
import { SyncStatus } from './SyncStatus'

const meta: Meta<typeof SyncStatus> = {
  title: 'Studio/SyncStatus',
  component: SyncStatus,
  parameters: { layout: 'centered' },
  argTypes: {
    status: {
      control: 'select',
      options: ['idle', 'saving', 'saved', 'error'],
    },
  },
}
export default meta
type Story = StoryObj<typeof SyncStatus>

/** Idle state -- nothing rendered. */
export const Idle: Story = {
  args: {
    status: 'idle',
  },
}

/** Saving state -- spinner with "Saving..." text. */
export const Saving: Story = {
  args: {
    status: 'saving',
  },
}

/** Saved state -- checkmark with "Saved" text (fades out after 2s). */
export const Saved: Story = {
  args: {
    status: 'saved',
  },
}

/** Error state -- warning icon with "Sync failed" text (fades out after 3s). */
export const Error: Story = {
  args: {
    status: 'error',
  },
}
