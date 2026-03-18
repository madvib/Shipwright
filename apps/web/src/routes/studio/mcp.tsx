import { createFileRoute } from '@tanstack/react-router'
import { useLibrary } from '#/features/compiler/useLibrary'
import { McpServersForm } from '#/features/compiler/sections/McpServersForm'

export const Route = createFileRoute('/studio/mcp')({ component: McpPage })

function McpPage() {
  const { library, updateLibrary } = useLibrary()

  return (
    <div className="h-full flex flex-col">
      {/* View header */}
      <div className="flex items-center px-4 h-11 border-b border-border/60 bg-card/30 shrink-0">
        <span className="text-sm font-semibold text-foreground mr-2">MCP Servers</span>
        <span className="text-[11px] text-muted-foreground/50">{library.mcp_servers.length} configured</span>
      </div>

      <div className="flex-1 overflow-auto p-6">
        <div className="mx-auto max-w-3xl">
          <McpServersForm
            servers={library.mcp_servers}
            onChange={(mcp_servers) => updateLibrary({ mcp_servers })}
          />
        </div>
      </div>
    </div>
  )
}
