import type { Meta, StoryObj } from '@storybook/react'
import { TechIcon, TECH_STACK_LIST } from './TechIcon'

const meta: Meta<typeof TechIcon> = {
  title: 'Studio/TechIcon',
  component: TechIcon,
  parameters: { layout: 'centered' },
  argTypes: {
    stack: {
      control: 'select',
      options: TECH_STACK_LIST.map((t) => t.id),
    },
    size: {
      control: 'radio',
      options: [24, 32, 36, 48],
    },
  },
}
export default meta
type Story = StoryObj<typeof TechIcon>

/** React icon at large size. */
export const React: Story = {
  args: { stack: 'react', size: 48 },
}

/** Rust icon at large size. */
export const Rust: Story = {
  args: { stack: 'rust', size: 48 },
}

/** TypeScript icon at default size. */
export const TypeScript: Story = {
  args: { stack: 'typescript', size: 36 },
}

/** Unknown stack falls back to the custom icon with initials. */
export const CustomFallback: Story = {
  args: { stack: 'unknown-tech', size: 48 },
}

/** All supported tech stacks at small size in a grid. */
export const AllSmall: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      {TECH_STACK_LIST.map((tech) => (
        <div key={tech.id} className="flex flex-col items-center gap-1">
          <TechIcon stack={tech.id} size={32} />
          <span className="text-[9px] text-muted-foreground">{tech.id}</span>
        </div>
      ))}
    </div>
  ),
}

/** All tech stacks at large size. */
export const AllLarge: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-4">
      {TECH_STACK_LIST.map((tech) => (
        <TechIcon key={tech.id} stack={tech.id} size={48} />
      ))}
    </div>
  ),
}
