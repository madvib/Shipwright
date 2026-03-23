import type { Meta, StoryObj } from '@storybook/react'
import { fn } from '@storybook/test'
import { StudioDock } from './StudioDock'
import { createMemoryHistory, createRootRoute, createRouter, RouterProvider } from '@tanstack/react-router'

function WithRouter({ children, path }: { children: React.ReactNode; path: string }) {
  const rootRoute = createRootRoute({ component: () => <>{children}</> })
  const router = createRouter({
    routeTree: rootRoute,
    history: createMemoryHistory({ initialEntries: [path] }),
  })
  return <RouterProvider router={router} />
}

const meta: Meta<typeof StudioDock> = {
  title: 'Studio/StudioDock',
  component: StudioDock,
  parameters: { layout: 'fullscreen' },
  decorators: [
    (Story) => (
      <WithRouter path="/studio/agents">
        <div className="relative h-40">
          <Story />
        </div>
      </WithRouter>
    ),
  ],
}
export default meta
type Story = StoryObj<typeof StudioDock>

/** Dock with the Agents tab active and preview closed. */
export const PreviewClosed: Story = {
  args: {
    previewOpen: false,
    onTogglePreview: fn(),
  },
}

/** Dock with preview panel open. */
export const PreviewOpen: Story = {
  args: {
    previewOpen: true,
    onTogglePreview: fn(),
  },
}
