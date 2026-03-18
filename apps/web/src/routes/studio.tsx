import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioDock } from '#/features/studio/StudioDock'
import { SyncStatus } from '#/features/studio/SyncStatus'
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
  const { syncStatus } = useLibrarySync()

  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative">
      <Outlet />
      <StudioDock />
      {/* Sync status — bottom-right, above the dock */}
      <div className="fixed bottom-16 right-4 z-40 pointer-events-none">
        <SyncStatus status={syncStatus} />
      </div>
    </main>
  )
}
