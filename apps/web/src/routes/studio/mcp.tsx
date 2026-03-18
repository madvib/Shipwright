import { createFileRoute } from '@tanstack/react-router'
import { useLibrary } from '#/features/compiler/useLibrary'
import { McpServersForm } from '#/features/compiler/sections/McpServersForm'

export const Route = createFileRoute('/studio/mcp')({ component: McpPage })

function McpPage() {
  const { library, updateLibrary } = useLibrary()

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5">
        <McpServersForm
          servers={library.mcp_servers}
          onChange={(mcp_servers) => updateLibrary({ mcp_servers })}
        />
      </div>
    </div>
  )
}
