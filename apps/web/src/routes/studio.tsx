import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioDock } from '#/features/studio/StudioDock'
import { ProtectedRoute } from '#/lib/components/protected-route'
import { useLibrarySync } from '#/features/compiler/useLibrarySync'

export const Route = createFileRoute('/studio')({ component: StudioLayout })

function StudioLayout() {
  return (
    <ProtectedRoute>
      <StudioSyncShell />
    </ProtectedRoute>
  )
}

/** Inner shell that lives inside ProtectedRoute so useAuth is available. */
function StudioSyncShell() {
  useLibrarySync()

  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative">
      <Outlet />
      <StudioDock />
    </main>
  )
}
