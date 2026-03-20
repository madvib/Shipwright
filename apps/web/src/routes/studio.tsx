import { createFileRoute, Outlet } from '@tanstack/react-router'
import { StudioDock } from '#/features/studio/StudioDock'
import { SyncStatus } from '#/features/studio/SyncStatus'
import { ProtectedRoute } from '#/lib/components/protected-route'
import { useLibrarySync } from '#/features/compiler/useLibrarySync'
import { useCompiler } from '#/features/compiler/useCompiler'
import { useLibrary } from '#/features/compiler/useLibrary'

export const Route = createFileRoute('/studio')({ component: StudioLayout, ssr: false })

function StudioLayout() {
  return (
    <ProtectedRoute>
      <StudioSyncShell />
    </ProtectedRoute>
  )
}

function StudioSyncShell() {
  const { syncStatus } = useLibrarySync()
  const { library } = useLibrary()
  const { compile, compileState } = useCompiler()

  const handleCompile = () => {
    if (library) compile(library)
  }

  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative pb-20">
      <Outlet />
      <StudioDock
        onCompile={handleCompile}
        isCompiling={compileState.status === 'compiling'}
      />
      <div className="fixed bottom-16 right-4 z-40 pointer-events-none">
        <SyncStatus status={syncStatus} />
      </div>
    </main>
  )
}
