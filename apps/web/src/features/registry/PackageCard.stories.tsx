import type { Meta, StoryObj } from '@storybook/react'
import { PackageCard } from './PackageCard'
import type { SearchPackage } from './types'
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

const basePkg: SearchPackage = {
  id: 'pkg-001',
  path: 'ship-ai/code-review',
  name: 'code-review',
  description: 'Automated code review skill with configurable severity levels and language-specific linting rules.',
  scope: 'official',
  repoUrl: 'https://github.com/ship-ai/skills',
  latestVersion: '1.3.0',
  claimedBy: 'ship-ai',
  deprecatedBy: null,
  stars: 142,
  installs: 8_420,
  updatedAt: Date.now() - 15 * 86_400_000,
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
      repoUrl: 'https://github.com/jsmith/react-patterns',
      claimedBy: 'jsmith',
      latestVersion: '2.1.0',
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
      repoUrl: 'https://github.com/random/test-skill',
      claimedBy: null,
      latestVersion: '0.0.1',
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
      latestVersion: null,
      claimedBy: null,
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
      repoUrl: 'https://gitlab.com/gitlab-user/my-skill',
      claimedBy: null,
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
