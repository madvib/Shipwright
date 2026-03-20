import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { Zap } from 'lucide-react'
import { useLibrary } from '#/features/compiler/useLibrary'
import { ModeHeader } from '#/features/compiler/components/ModeHeader'
import { LibraryPanel } from '#/features/compiler/components/LibraryPanel'
import { ComposerPanel } from '#/features/compiler/components/ComposerPanel'
import { InspectorPanel, getInspectorTabs } from '#/features/compiler/components/InspectorPanel'
import { ImportDialog } from '#/components/ImportDialog'
import type { ComposerSection } from '#/features/compiler/components/ComposerPanel'

export const Route = createFileRoute('/studio/editor')({ component: StudioEditor })

function StudioEditor() {
  const {
    library,
    modeName,
    selectedProviders,
    compileState,
    updateLibrary,
    setModeName,
    handleImport,
    addMcpServer,
    addSkill,
    toggleProvider,
  } = useLibrary()

  const [activeSection, setActiveSection] = useState<ComposerSection>('providers')
  const [showLibrary, setShowLibrary] = useState(true)
  const [importOpen, setImportOpen] = useState(false)

  const getTabsForProvider = (provider: string) => {
    if (compileState.status !== 'ok') return []
    const result = compileState.output[provider]
    return result ? getInspectorTabs(provider, result) : []
  }

  return (
    <>
      {/* Mobile fallback */}
      <div className="flex md:hidden flex-col items-center justify-center gap-4 px-8 py-20 text-center min-h-[60vh]">
        <div className="flex size-12 items-center justify-center rounded-xl border border-border/60 bg-muted/40">
          <Zap className="size-5 text-muted-foreground" />
        </div>
        <div>
          <p className="font-display text-base font-semibold">Best on desktop</p>
          <p className="mt-1 text-sm text-muted-foreground max-w-xs">
            Ship Studio is a three-panel editor — open it on a wider screen for the full experience.
          </p>
        </div>
      </div>

      <div className="hidden md:flex flex-1 min-h-0 flex-col overflow-hidden">
        <ModeHeader
          modeName={modeName}
          onModeNameChange={setModeName}
          library={library}
          state={compileState}
          selectedProviders={selectedProviders}
          showLibrary={showLibrary}
          onToggleLibrary={() => setShowLibrary((v) => !v)}
          onOpenImport={() => setImportOpen(true)}
          getInspectorTabs={getTabsForProvider}
        />
        <ImportDialog
          open={importOpen}
          onClose={() => setImportOpen(false)}
          onImport={handleImport}
        />
        <div className="flex flex-1 min-h-0 overflow-hidden">
          {showLibrary && (
            <LibraryPanel
              library={library}
              onAddMcp={addMcpServer}
              onAddSkill={addSkill}
            />
          )}
          <ComposerPanel
            library={library}
            activeSection={activeSection}
            selectedProviders={selectedProviders}
            onSectionChange={setActiveSection}
            onLibraryChange={updateLibrary}
            onToggleProvider={toggleProvider}
          />
          <InspectorPanel
            state={compileState}
            selectedProviders={selectedProviders}
          />
        </div>
      </div>
    </>
  )
}
