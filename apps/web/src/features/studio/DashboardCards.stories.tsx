import type { Meta, StoryObj } from '@storybook/react'
import { DashboardCards } from './DashboardCards'
import { createMemoryHistory, createRootRoute, createRouter, RouterProvider } from '@tanstack/react-router'

function WithRouter({ children }: { children: React.ReactNode }) {
  const rootRoute = createRootRoute({ component: () => <>{children}</> })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  return <RouterProvider router={router} />
}

const meta: Meta<typeof DashboardCards> = {
  title: 'Studio/DashboardCards',
  component: DashboardCards,
  parameters: { layout: 'padded' },
  decorators: [
    (Story) => (
      <WithRouter>
        <div style={{ maxWidth: 900 }}>
          <Story />
        </div>
      </WithRouter>
    ),
  ],
}
export default meta
type Story = StoryObj<typeof DashboardCards>

/** Fresh start -- no profiles or jobs created yet. */
export const Empty: Story = {
  args: {
    profileCount: 0,
    workflowJobCount: 0,
  },
}

/** Active workspace -- profiles configured, workflow running. */
export const ActiveWorkspace: Story = {
  args: {
    profileCount: 3,
    workflowJobCount: 5,
  },
}

/** Single profile -- workflow and export become available. */
export const SingleProfile: Story = {
  args: {
    profileCount: 1,
    workflowJobCount: 0,
  },
}

/** Many profiles with many active jobs. */
export const HeavyUsage: Story = {
  args: {
    profileCount: 12,
    workflowJobCount: 28,
  },
}
