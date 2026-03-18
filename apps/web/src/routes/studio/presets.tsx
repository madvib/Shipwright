import { createFileRoute } from '@tanstack/react-router'
import { useLibrary } from '#/features/compiler/useLibrary'
import { LibraryPanel } from '#/features/compiler/components/LibraryPanel'

export const Route = createFileRoute('/studio/presets')({ component: PresetsPage })

function PresetsPage() {
  const { library, addMcpServer, addSkill } = useLibrary()

  return (
    <div className="h-full flex flex-col">
      {/* View header */}
      <div className="flex items-center px-4 h-11 border-b border-border/60 bg-card/30 shrink-0">
        <span className="text-sm font-semibold text-foreground mr-2">Presets</span>
        <span className="text-[11px] text-muted-foreground/50">Community configs</span>
      </div>

      <div className="flex-1 overflow-auto p-6">
        <div className="mx-auto max-w-3xl">
          <LibraryPanel
            library={library}
            onAddMcp={addMcpServer}
            onAddSkill={addSkill}
          />
        </div>
      </div>
    </div>
  )
}
