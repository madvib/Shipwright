import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioDock } from '#/features/studio/StudioDock'

export const Route = createFileRoute('/studio')({ component: StudioLayout })

function StudioLayout() {
  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative">
      <Outlet />
      <StudioDock />
    </main>
  )
}
