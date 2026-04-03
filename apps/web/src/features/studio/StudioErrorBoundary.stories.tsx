import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { StudioErrorBoundary } from './StudioErrorBoundary'
import { createMemoryHistory, createRootRoute, createRouter, RouterProvider } from '@tanstack/react-router'

/**
 * Wraps StudioErrorBoundary in a minimal TanStack Router context
 * so the <Link> components inside it can resolve without crashing.
 */
function WithRouter({ children }: { children: React.ReactNode }) {
  const rootRoute = createRootRoute({ component: () => <>{children}</> })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })
  return <RouterProvider router={router} />
}

const meta: Meta<typeof StudioErrorBoundary> = {
  title: 'Studio/StudioErrorBoundary',
  component: StudioErrorBoundary,
  parameters: { layout: 'centered' },
  decorators: [
    (Story) => (
      <WithRouter>
        <Story />
      </WithRouter>
    ),
  ],
}
export default meta
type Story = StoryObj<typeof StudioErrorBoundary>

/** Not-found state -- when an agent ID does not exist. */
export const NotFound: Story = {
  args: {
    error: new Error('Agent not found'),
    reset: fn(),
  },
}

/** Not-found via 404 Response object. */
export const NotFound404Response: Story = {
  args: {
    error: new Response(null, { status: 404 }) as unknown as Error,
    reset: fn(),
  },
}

/** Generic error with a descriptive message. */
export const GenericError: Story = {
  args: {
    error: new Error('Failed to load agent configuration: network timeout after 30s'),
    reset: fn(),
  },
}

/** Generic error with a short message. */
export const ShortError: Story = {
  args: {
    error: new Error('Permission denied'),
    reset: fn(),
  },
}

/** String error (non-Error instance). */
export const StringError: Story = {
  args: {
    error: 'Something unexpected happened during compilation.' as unknown as Error,
    reset: fn(),
  },
}
