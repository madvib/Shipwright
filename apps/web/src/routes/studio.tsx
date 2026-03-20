import { createFileRoute, Outlet } from '@tanstack/react-router'
import { useState } from 'react'
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
  const { state: compileState } = useCompiler()
  const [previewOpen, setPreviewOpen] = useState(false)

  return (
    <main className="flex-1 overflow-hidden min-w-0 flex flex-col relative pb-20">
      <div className="flex-1 flex min-h-0 overflow-hidden">
        <div className="flex-1 overflow-auto min-w-0">
          <Outlet />
        </div>
        {previewOpen && (
          <OutputPreview library={library} compileState={compileState} />
        )}
      </div>
      <StudioDock
        previewOpen={previewOpen}
        onTogglePreview={() => setPreviewOpen((p) => !p)}
      />
      <div className="fixed bottom-16 right-4 z-40 pointer-events-none">
        <SyncStatus status={syncStatus} />
      </div>
    </main>
  )
}

function OutputPreview({ library, compileState }: { library: any; compileState: any }) {
  return (
    <aside className="w-80 border-l border-border/60 bg-card/50 flex flex-col overflow-hidden shrink-0">
      <div className="px-4 py-3 border-b border-border/40 flex items-center justify-between">
        <h3 className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">Output Preview</h3>
      </div>
      {compileState.status === 'ok' ? (
        <div className="flex-1 overflow-auto p-4">
          {Object.entries(compileState.output as Record<string, any>).map(([provider, result]) => (
            <div key={provider} className="mb-4">
              <div className="text-[10px] font-semibold text-primary uppercase tracking-wider mb-2">{provider}</div>
              {result.context_content && (
                <div className="mb-2">
                  <div className="text-[10px] text-muted-foreground/60 mb-1">
                    {provider === 'claude' ? 'CLAUDE.md' : provider === 'gemini' ? 'GEMINI.md' : 'AGENTS.md'}
                  </div>
                  <pre className="text-[10px] font-mono text-muted-foreground bg-muted/30 rounded-lg p-3 overflow-x-auto whitespace-pre-wrap max-h-48">
                    {result.context_content.slice(0, 500)}
                    {result.context_content.length > 500 && '...'}
                  </pre>
                </div>
              )}
              {result.mcp_config_path && (
                <div className="text-[10px] text-muted-foreground/60">
                  <span className="font-mono">{result.mcp_config_path}</span>
                </div>
              )}
            </div>
          ))}
        </div>
      ) : compileState.status === 'compiling' ? (
        <div className="flex-1 flex items-center justify-center text-xs text-muted-foreground">
          Compiling...
        </div>
      ) : compileState.status === 'error' ? (
        <div className="flex-1 p-4 text-xs text-destructive">{compileState.message}</div>
      ) : (
        <div className="flex-1 flex items-center justify-center p-6 text-center">
          <p className="text-xs text-muted-foreground">
            Edit your agent config to see live output here.
          </p>
        </div>
      )}
    </aside>
  )
}
