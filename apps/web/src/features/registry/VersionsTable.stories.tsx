import type { Meta, StoryObj } from '@storybook/react'
import { VersionsTable } from './VersionsTable'
import type { PackageVersion } from './types'

const meta: Meta<typeof VersionsTable> = {
  title: 'Registry/VersionsTable',
  component: VersionsTable,
  parameters: { layout: 'padded' },
}
export default meta
type Story = StoryObj<typeof VersionsTable>

const makeVersion = (
  version: string,
  gitTag: string,
  commitSha: string,
  skills: string[],
  daysAgo: number,
): PackageVersion => ({
  id: `ver-${version}`,
  version,
  gitTag,
  commitSha,
  skills,
  agents: [],
  indexedAt: Date.now() - daysAgo * 86_400_000,
})

/** Empty state -- no versions indexed yet. */
export const Empty: Story = {
  args: {
    versions: [],
  },
}

/** Single version tagged as latest. */
export const SingleVersion: Story = {
  args: {
    versions: [
      makeVersion('1.0.0', 'v1.0.0', 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0', ['code-review', 'debug-expert'], 2),
    ],
  },
}

/** Multiple versions showing version history. */
export const MultipleVersions: Story = {
  args: {
    versions: [
      makeVersion('1.3.0', 'v1.3.0', 'f1e2d3c4b5a6f7e8d9c0b1a2f3e4d5c6b7a8f9e0', ['code-review', 'debug-expert', 'frontend-design'], 1),
      makeVersion('1.2.1', 'v1.2.1', 'a9b8c7d6e5f4a3b2c1d0e9f8a7b6c5d4e3f2a1b0', ['code-review', 'debug-expert'], 14),
      makeVersion('1.2.0', 'v1.2.0', 'c1d2e3f4a5b6c7d8e9f0a1b2c3d4e5f6a7b8c9d0', ['code-review', 'debug-expert'], 30),
      makeVersion('1.1.0', 'v1.1.0', 'e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4', ['code-review'], 60),
      makeVersion('1.0.0', 'v1.0.0', 'a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0', ['code-review'], 120),
    ],
  },
}

/** Version with no skills attached. */
export const NoSkills: Story = {
  args: {
    versions: [
      makeVersion('0.1.0', 'v0.1.0', 'deadbeefcafe1234567890abcdef1234567890ab', [], 5),
    ],
  },
}
