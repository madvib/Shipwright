import type { Meta, StoryObj } from '@storybook/react'
import { PackageCard } from './PackageCard'
import type { RegistryPackage } from './types'
import { createMemoryHistory, createRootRoute, createRouter, RouterProvider } from '@tanstack/react-router'

function WithRouter({ children }: { children: React.ReactNode }) {
  const rootRoute = createRootRoute({ component: () => <>{children}</> })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  return <RouterProvider router={router} />
}

const meta: Meta<typeof PackageCard> = {
  title: 'Registry/PackageCard',
  component: PackageCard,
  parameters: { layout: 'centered' },
  decorators: [
    (Story) => (
      <WithRouter>
        <div style={{ width: 320 }}>
          <Story />
        </div>
      </WithRouter>
    ),
  ],
}
export default meta
type Story = StoryObj<typeof PackageCard>

const basePkg: RegistryPackage = {
  id: 'pkg-001',
  path: 'ship-ai/code-review',
  name: 'code-review',
  description: 'Automated code review skill with configurable severity levels and language-specific linting rules.',
  scope: 'official',
  repo_url: 'https://github.com/ship-ai/skills',
  default_branch: 'main',
  latest_version: '1.3.0',
  content_hash: 'abc123',
  source_type: 'native',
  claimed_by: 'ship-ai',
  deprecated_by: null,
  stars: 142,
  installs: 8_420,
  indexed_at: '2026-01-15T00:00:00Z',
  updated_at: '2026-03-10T00:00:00Z',
}

/** Official package with verified owner and GitHub avatar. */
export const Official: Story = {
  args: {
    pkg: basePkg,
  },
}

/** Community-scoped package with high install count. */
export const Community: Story = {
  args: {
    pkg: {
      ...basePkg,
      id: 'pkg-002',
      path: 'jsmith/react-patterns',
      name: 'react-patterns',
      description: 'Common React design patterns including compound components, render props, and custom hooks.',
      scope: 'community',
      repo_url: 'https://github.com/jsmith/react-patterns',
      claimed_by: 'jsmith',
      latest_version: '2.1.0',
      installs: 12_350,
    },
  },
}

/** Unofficial scope -- unverified, no claimed owner. */
export const Unofficial: Story = {
  args: {
    pkg: {
      ...basePkg,
      id: 'pkg-003',
      path: 'random/test-skill',
      name: 'test-skill',
      description: 'An experimental skill without verification.',
      scope: 'unofficial',
      repo_url: 'https://github.com/random/test-skill',
      claimed_by: null,
      latest_version: '0.0.1',
      installs: 23,
      stars: 1,
    },
  },
}

/** Package with no release version. */
export const NoVersion: Story = {
  args: {
    pkg: {
      ...basePkg,
      id: 'pkg-004',
      path: 'acme/unreleased',
      name: 'unreleased-skill',
      description: 'A package that has been indexed but has no published version yet.',
      scope: 'community',
      latest_version: null,
      content_hash: null,
      claimed_by: null,
      installs: 0,
    },
  },
}

/** Non-GitHub repo URL -- avatar should fall back to initials. */
export const NoGitHubAvatar: Story = {
  args: {
    pkg: {
      ...basePkg,
      id: 'pkg-005',
      path: 'gitlab-user/my-skill',
      name: 'gitlab-hosted-skill',
      description: 'A skill hosted on a non-GitHub platform.',
      scope: 'community',
      repo_url: 'https://gitlab.com/gitlab-user/my-skill',
      claimed_by: null,
      installs: 580,
    },
  },
}

/** High install count formatting. */
export const HighInstalls: Story = {
  args: {
    pkg: {
      ...basePkg,
      id: 'pkg-006',
      installs: 45_200,
    },
  },
}
