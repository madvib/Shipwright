import { createFileRoute, Outlet, useMatches, useRouterState } from '@tanstack/react-router'
import { useState, useEffect, useRef, useMemo } from 'react'
import { StudioDock } from '#/features/studio/StudioDock'
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

  const { getAgent } = useAgents()
  const activeAgent = activeAgentId ? getAgent(activeAgentId) : undefined

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

  // Session page is full-screen — no dock, no bottom padding
  const pathname = useRouterState({ select: (s) => s.location.pathname })
  const isSession = pathname.startsWith('/studio/session')

  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative">
      {/* Top sub-nav (replaces bottom dock) */}
      {!isSession && (
        <StudioDock
          previewOpen={showCompilerPanel && panelOpen}
          showPreviewToggle={showCompilerPanel}
          onTogglePreview={() => setPanelOpen((p) => !p)}
          onAddSkill={addSkill}
        />
      )}
      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className={`flex-1 min-w-0 ${isSession ? 'flex flex-col overflow-hidden' : 'overflow-auto'}`}>
          <Outlet />
        </div>
        {showCompilerPanel && panelOpen && !isSession && (
          <div className="hidden md:block">
            <PublishPanel
              library={effectiveLibrary}
              compileState={compileState}
              onClose={() => setPanelOpen(false)}
            />
          </div>
        )}
      </div>
    </main>
  )
}
