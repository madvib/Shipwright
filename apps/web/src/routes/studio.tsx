import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/studio')({ component: StudioLayout })

function StudioLayout() {
  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col">
      <Outlet />
    </main>
  )
}
