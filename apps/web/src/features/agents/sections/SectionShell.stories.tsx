import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { SectionShell, Chip, ChipIcon, AddChip } from './SectionShell'
import { Zap, Grid3X3, Lock } from 'lucide-react'

const meta: Meta<typeof SectionShell> = {
  title: 'Agents/SectionShell',
  component: SectionShell,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof SectionShell>

/** Section shell with action button and chip children. */
export const WithChips: Story = {
  args: {
    icon: <Zap className="size-4" />,
    title: 'Skills',
    count: '3 attached',
    actionLabel: 'Add',
    onAction: fn(),
    children: (
      <div className="flex flex-wrap gap-1.5">
        <Chip
          icon={<ChipIcon letters="SC" variant="skill" />}
          name="ship-coordination"
          meta="v0.1.0 / custom"
          onRemove={fn()}
        />
        <Chip
          icon={<ChipIcon letters="CR" variant="skill" />}
          name="code-review"
          meta="v1.2.0 / community"
          onRemove={fn()}
        />
        <Chip
          icon={<ChipIcon letters="FD" variant="skill" />}
          name="frontend-design"
          meta="v2.0.0 / custom"
          onRemove={fn()}
        />
        <AddChip label="Add skill" onClick={fn()} />
      </div>
    ),
  },
}

/** Section shell without action button (read-only view). */
export const ReadOnly: Story = {
  args: {
    icon: <Lock className="size-4" />,
    title: 'Permissions',
    children: (
      <p className="text-xs text-muted-foreground">Read-only content goes here.</p>
    ),
  },
}

/** MCP variant with chip badges. */
export const McpVariant: Story = {
  args: {
    icon: <Grid3X3 className="size-4" />,
    title: 'MCP Servers',
    count: '2 attached',
    actionLabel: 'Add',
    onAction: fn(),
    children: (
      <div className="flex flex-wrap gap-1.5">
        <Chip
          icon={<ChipIcon letters="SH" variant="mcp" />}
          name="ship"
          meta="ship mcp serve / stdio"
          badge={
            <span className="rounded px-1.5 py-0.5 text-[9px] font-medium bg-emerald-500/10 text-emerald-600">
              all
            </span>
          }
          onRemove={fn()}
        />
        <Chip
          icon={<ChipIcon letters="GH" variant="mcp" />}
          name="github"
          meta="npx -y @mcp/server-github / stdio"
          badge={
            <span className="rounded px-1.5 py-0.5 text-[9px] font-medium bg-primary/10 text-primary">
              8/18
            </span>
          }
          active
          onClick={fn()}
          onRemove={fn()}
        />
        <AddChip label="Add server" onClick={fn()} />
      </div>
    ),
  },
}

/** Empty section. */
export const Empty: Story = {
  args: {
    icon: <Zap className="size-4" />,
    title: 'Skills',
    count: '0 attached',
    actionLabel: 'Add',
    onAction: fn(),
    children: (
      <div className="flex flex-wrap gap-1.5">
        <AddChip label="Add skill" onClick={fn()} />
      </div>
    ),
  },
}
