import { createFileRoute } from '@tanstack/react-router'
import { useLibrary } from '#/features/compiler/useLibrary'
import { LibraryPanel } from '#/features/compiler/components/LibraryPanel'

export const Route = createFileRoute('/studio/templates')({ component: TemplatesPage })

function TemplatesPage() {
  const { library, addMcpServer, addSkill } = useLibrary()

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5">
        <LibraryPanel library={library} onAddMcp={addMcpServer} onAddSkill={addSkill} />
      </div>
    </div>
  )
}
