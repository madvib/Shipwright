import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { IconPicker } from './IconPicker'

const meta: Meta<typeof IconPicker> = {
  title: 'Studio/IconPicker',
  component: IconPicker,
  parameters: { layout: 'centered' },
}
export default meta
type Story = StoryObj<typeof IconPicker>

/** React icon selected with the default cyan accent. */
export const ReactIcon: Story = {
  args: {
    icon: 'react',
    accentColor: '#61dafb',
    name: 'web-lane',
    onChange: fn(),
  },
}

/** Rust icon selected with red accent. */
export const RustIcon: Story = {
  args: {
    icon: 'rust',
    accentColor: '#ce422b',
    name: 'backend-runtime',
    onChange: fn(),
  },
}

/** TypeScript icon with blue accent. */
export const TypeScriptIcon: Story = {
  args: {
    icon: 'typescript',
    accentColor: '#3178c6',
    name: 'ts-compiler',
    onChange: fn(),
  },
}

/** Custom icon fallback with amber accent. */
export const CustomIcon: Story = {
  args: {
    icon: 'custom',
    accentColor: '#f59e0b',
    name: 'my-agent',
    onChange: fn(),
  },
}
