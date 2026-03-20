import { createFileRoute, Link } from '@tanstack/react-router'
import { Server } from 'lucide-react'
import { useLibrary } from '#/features/compiler/useLibrary'
import { McpServersForm } from '#/features/compiler/sections/McpServersForm'
import { EmptyState } from '#/components/EmptyState'

export const Route = createFileRoute('/studio/mcp')({ component: McpPage })

function McpPage() {
  const { library, updateLibrary } = useLibrary()

  const servers = library.mcp_servers ?? []

  if (servers.length === 0) {
    return (
      <div className="h-full flex flex-col">
        <EmptyState
          icon={<Server className="size-5" />}
          title="No MCP servers yet"
          description="MCP servers give your agent access to external tools — GitHub, Linear, memory, and more."
          action={
            <Link
              to="/studio/registry"
              className="inline-flex items-center gap-1.5 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground transition hover:opacity-90 no-underline"
            >
              Explore the registry
            </Link>
          }
        />
      </div>
    )
  }

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 overflow-auto p-5">
        <McpServersForm
          servers={servers}
          onChange={(mcp_servers) => updateLibrary({ mcp_servers })}
        />
      </div>
    </div>
  )
}
