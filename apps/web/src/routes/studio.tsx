import { createFileRoute, Outlet, useMatches } from '@tanstack/react-router'
import { useState, useEffect, useRef, useMemo } from 'react'
import { StudioDock } from '#/features/studio/StudioDock'
import { SyncStatus } from '#/features/studio/SyncStatus'
import type { SyncStatusValue } from '#/features/studio/SyncStatus'
import { PublishPanel } from '#/features/studio/PublishPanel'
import { useCompiler } from '#/features/compiler/useCompiler'
import { useLibrary } from '#/features/compiler/useLibrary'
import { useAgents } from '#/features/agents/useAgents'
import { agentToLibrary } from '#/features/agents/agent-to-library'
import { StudioErrorBoundary } from '#/features/studio/StudioErrorBoundary'
import { LocalMcpProvider } from '#/features/studio/LocalMcpContext'
import { PanicSaveProvider } from '#/features/agents/PanicSaveContext'

export const Route = createFileRoute('/studio')({
  component: StudioLayout,
  errorComponent: StudioErrorBoundary,
  ssr: false,
})

function StudioLayout() {
  return (
    <PanicSaveProvider>
      <LocalMcpProvider>
        <StudioSyncShell />
      </LocalMcpProvider>
    </PanicSaveProvider>
  )
}

function StudioSyncShell() {
  const { library, addSkill } = useLibrary()
  const { state: compileState, compile } = useCompiler()
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined)

  // Detect active agent from route matches (/studio/agents/$id)
  const matches = useMatches()
  const activeAgentId = useMemo(() => {
    for (const match of matches) {
      const params = match.params as Record<string, string> | undefined
      if (params?.id && match.routeId === '/studio/agents/$id') {
        return params.id
      }
    }
    return null
  }, [matches])

  const [panelOpen, setPanelOpen] = useState(() =>
    typeof window !== 'undefined' ? window.innerWidth >= 768 : true
  )

  const { getAgent, isConnected } = useAgents()
  const activeAgent = activeAgentId ? getAgent(activeAgentId) : undefined

  // Derive sync status from MCP connection state
  const syncStatus: SyncStatusValue = isConnected ? 'saved' : 'idle'

  // Build effective library: merge agent config when viewing an agent
  const effectiveLibrary = useMemo(() => {
    if (!library) return library
    if (!activeAgent) return library
    return agentToLibrary(activeAgent, library)
  }, [library, activeAgent])

  // Auto-compile when effective library changes while panel is open
  useEffect(() => {
    if (!panelOpen || !effectiveLibrary) return
    clearTimeout(debounceRef.current)
    debounceRef.current = setTimeout(() => compile(effectiveLibrary), 600)
    return () => clearTimeout(debounceRef.current)
  }, [effectiveLibrary, panelOpen])

  // Immediate compile when panel opens
  useEffect(() => {
    if (panelOpen && effectiveLibrary) {
      compile(effectiveLibrary)
    }
  }, [panelOpen])

  // Only show compiler output panel on agent detail pages
  const showCompilerPanel = Boolean(activeAgentId)

  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative pb-20">
      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className="flex-1 overflow-auto min-w-0">
          <Outlet />
        </div>
        {showCompilerPanel && panelOpen && (
          <div className="hidden md:block">
            <PublishPanel
              library={effectiveLibrary}
              compileState={compileState}
              onClose={() => setPanelOpen(false)}
            />
          </div>
        )}
      </div>
      <StudioDock
        previewOpen={showCompilerPanel && panelOpen}
        showPreviewToggle={showCompilerPanel}
        onTogglePreview={() => setPanelOpen((p) => !p)}
        onAddSkill={addSkill}
      />
      <div className="fixed bottom-16 right-4 z-40 pointer-events-none">
        <SyncStatus status={syncStatus} />
      </div>
    </main>
  )
}
