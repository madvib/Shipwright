import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { BentoCard, BentoGrid } from './BentoCard'

const meta: Meta<typeof BentoCard> = {
  title: 'Studio/BentoCard',
  component: BentoCard,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof BentoCard>

/** Default card with violet glow. */
export const Default: Story = {
  args: {
    glowColor: '139, 92, 246',
    onClick: fn(),
    children: (
      <div className="flex flex-col gap-2 p-2">
        <h3 className="text-sm font-semibold text-foreground">Agents</h3>
        <p className="text-xs text-muted-foreground">Manage your AI agent profiles and configurations.</p>
      </div>
    ),
  },
}

/** Card with emerald glow color. */
export const EmeraldGlow: Story = {
  args: {
    glowColor: '34, 197, 94',
    onClick: fn(),
    children: (
      <div className="flex flex-col gap-2 p-2">
        <h3 className="text-sm font-semibold text-foreground">Skills</h3>
        <p className="text-xs text-muted-foreground">Instruction files injected into agent context.</p>
      </div>
    ),
  },
}

/** Card with amber glow color for warnings or attention. */
export const AmberGlow: Story = {
  args: {
    glowColor: '245, 158, 11',
    onClick: fn(),
    children: (
      <div className="flex flex-col gap-2 p-2">
        <h3 className="text-sm font-semibold text-foreground">Workflow</h3>
        <p className="text-xs text-muted-foreground">Wire agents together on a canvas.</p>
      </div>
    ),
  },
}

/** Grid layout with multiple cards. */
export const GridLayout: Story = {
  render: () => (
    <div style={{ maxWidth: 900 }}>
      <BentoGrid>
        <BentoCard glowColor="139, 92, 246" span="col-span-2">
          <h3 className="text-sm font-semibold text-foreground mb-1">Profiles</h3>
          <p className="text-xs text-muted-foreground">Choose which AI providers to target.</p>
        </BentoCard>
        <BentoCard glowColor="34, 197, 94">
          <h3 className="text-sm font-semibold text-foreground mb-1">Skills</h3>
          <p className="text-xs text-muted-foreground">Instruction files for agents.</p>
        </BentoCard>
        <BentoCard glowColor="59, 130, 246">
          <h3 className="text-sm font-semibold text-foreground mb-1">MCP Servers</h3>
          <p className="text-xs text-muted-foreground">Tools and APIs your agents can call.</p>
        </BentoCard>
        <BentoCard glowColor="245, 158, 11" span="col-span-2">
          <h3 className="text-sm font-semibold text-foreground mb-1">Workflow</h3>
          <p className="text-xs text-muted-foreground">Wire agents together for orchestration.</p>
        </BentoCard>
      </BentoGrid>
    </div>
  ),
}
