import { createFileRoute, Outlet } from '@tanstack/react-router'
import { useState, useEffect, useRef } from 'react'
import { StudioDock } from '#/features/studio/StudioDock'
import { SyncStatus } from '#/features/studio/SyncStatus'
import { PublishPanel } from '#/features/studio/PublishPanel'
import { ProtectedRoute, useAuth } from '#/lib/components/protected-route'
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
  const { library, selectedProviders } = useLibrary()
  const { state: compileState, compile } = useCompiler()
  const auth = useAuth()
  const [panelOpen, setPanelOpen] = useState(false)
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined)

  // Auto-compile when library changes while panel is open
  useEffect(() => {
    if (!panelOpen || !library) return
    clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => compile(library), 600)
    return () => clearTimeout(debounceRef.current)
  }, [library, panelOpen])

  // Immediate compile when panel opens
  useEffect(() => {
    if (panelOpen && library) {
      compile(library)
    }
  }, [panelOpen])

  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative pb-20">
      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className="flex-1 overflow-auto min-w-0">
          <Outlet />
        </div>
        {panelOpen && (
          <PublishPanel
            auth={auth}
            library={library}
            compileState={compileState}
            selectedProviders={selectedProviders}
            onCompile={() => { if (library) compile(library) }}
            onClose={() => setPanelOpen(false)}
          />
        )}
      </div>
      <StudioDock
        previewOpen={panelOpen}
        onTogglePreview={() => setPanelOpen((p) => !p)}
      />
      <div className="fixed bottom-16 right-4 z-40 pointer-events-none">
        <SyncStatus status={syncStatus} />
      </div>
    </main>
  )
}
