import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { Pagination } from './Pagination'

const meta: Meta<typeof Pagination> = {
  title: 'Registry/Pagination',
  component: Pagination,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof Pagination>

/** First page -- Prev button should be disabled. */
export const FirstPage: Story = {
  args: {
    page: 1,
    totalPages: 12,
    onPageChange: fn(),
  },
}

/** Middle page -- both buttons active. */
export const MiddlePage: Story = {
  args: {
    page: 6,
    totalPages: 12,
    onPageChange: fn(),
  },
}

/** Last page -- Next button should be disabled. */
export const LastPage: Story = {
  args: {
    page: 12,
    totalPages: 12,
    onPageChange: fn(),
  },
}

/** Single page -- both buttons disabled. */
export const SinglePage: Story = {
  args: {
    page: 1,
    totalPages: 1,
    onPageChange: fn(),
  },
}

/** Two pages on the first. */
export const TwoPagesFirst: Story = {
  args: {
    page: 1,
    totalPages: 2,
    onPageChange: fn(),
  },
}
