import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { SkillsSection } from './SkillsSection'
import type { Skill } from '@ship/ui'

const meta: Meta<typeof SkillsSection> = {
  title: 'Agents/SkillsSection',
  component: SkillsSection,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof SkillsSection>

const makeSkill = (id: string, name: string, source: Skill['source'], version?: string): Skill => ({
  id,
  name,
  content: `# ${name}\n\nSkill content for ${name}.`,
  source,
  metadata: { version: version ?? 'v0.1.0' },
  vars: {},
  artifacts: [],
})

/** Empty state -- no skills attached, only the "Add skill" chip visible. */
export const Empty: Story = {
  args: {
    skills: [],
    onRemove: fn(),
    onAdd: fn(),
  },
}

/** A single skill chip. */
export const SingleSkill: Story = {
  args: {
    skills: [makeSkill('code-review', 'code-review', 'community', 'v1.2.0')],
    onRemove: fn(),
    onAdd: fn(),
  },
}

/** Five skills -- the typical agent configuration. */
export const FiveSkills: Story = {
  args: {
    skills: [
      makeSkill('ship-coordination', 'ship-coordination', 'custom'),
      makeSkill('code-review', 'code-review', 'community', 'v1.2.0'),
      makeSkill('debug-expert', 'debug-expert', 'community', 'v0.9.0'),
      makeSkill('frontend-design', 'frontend-design', 'custom', 'v2.0.0'),
      makeSkill('vercel-react', 'vercel-react-best-practices', 'community', 'v1.0.0'),
    ],
    onRemove: fn(),
    onAdd: fn(),
  },
}

/** Without an add handler -- the add button should still render but do nothing meaningful. */
export const NoAddHandler: Story = {
  args: {
    skills: [
      makeSkill('readonly-skill', 'readonly-skill', 'custom'),
    ],
    onRemove: fn(),
  },
}
