import type { Meta, StoryObj } from '@storybook/react'
import { DangerZoneSection } from './DangerZoneSection'
import { createMemoryHistory, createRootRoute, createRouter, RouterProvider } from '@tanstack/react-router'

/**
 * DangerZoneSection uses useNavigate and useAgentStore internally.
 * We wrap it in a minimal router context. The agent store is
 * localStorage-backed so it works without extra providers.
 */
function WithRouter({ children }: { children: React.ReactNode }) {
  const rootRoute = createRootRoute({ component: () => <>{children}</> })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  return <RouterProvider router={router} />
}

const meta: Meta<typeof DangerZoneSection> = {
  title: 'Settings/DangerZoneSection',
  component: DangerZoneSection,
  parameters: { layout: 'padded' },
  decorators: [
    (Story) => (
      <WithRouter>
        <div style={{ maxWidth: 600 }}>
          <Story />
        </div>
      </WithRouter>
    ),
  ],
}
export default meta
type Story = StoryObj<typeof DangerZoneSection>

/** Default state with destructive action buttons. */
export const Default: Story = {}
