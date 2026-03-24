import { useState } from 'react'
import { Grid3X3 } from 'lucide-react'
import type { McpServerConfig } from '@ship/ui'
import type { ToolToggleState, ToolPermission } from '../types'
import { SectionShell, Chip, ChipIcon, AddChip } from './SectionShell'

function getInitials(name: string): string {
  const words = name.split(/[-_\s]/)
  if (words.length >= 2) return (words[0][0] + words[1][0]).toUpperCase()
  return name.slice(0, 2).toUpperCase()
}

interface McpSectionProps {
  servers: McpServerConfig[]
  toolStates: Record<string, ToolToggleState>
  onRemove: (name: string) => void
  onSetToolPermission: (server: string, tool: string, perm: ToolPermission) => void
  onSetGroupPermission: (server: string, tools: string[], perm: ToolPermission) => void
  onAdd?: () => void
}

export function McpSection({
  servers,
  onRemove,
  onAdd,
}: McpSectionProps) {
  const [expandedServer, setExpandedServer] = useState<string | null>(null)

  return (
    <SectionShell
      icon={<Grid3X3 className="size-4" />}
      title="MCP Servers"
      count={`${servers.length} attached`}
      actionLabel="Add"
      onAction={onAdd}
    >
      <div className="flex flex-wrap gap-1.5">
        {servers.map((server) => (
          <Chip
            key={server.name}
            icon={<ChipIcon letters={getInitials(server.name)} variant="mcp" />}
            name={server.name}
            meta={`${server.command} ${(server.args ?? []).slice(0, 2).join(' ')} / ${server.server_type ?? 'stdio'}`}
            onClick={() =>
              setExpandedServer((prev) =>
                prev === server.name ? null : server.name,
              )
            }
            active={expandedServer === server.name}
            ariaExpanded={expandedServer === server.name}
            onRemove={() => onRemove(server.name)}
          />
        ))}
        <AddChip label="Add server" onClick={onAdd} />
      </div>

      {/* Expanded server detail — shows server info, no fake tool list */}
      {expandedServer && (() => {
        const server = servers.find((s) => s.name === expandedServer)
        if (!server) return null
        return (
          <div className="mt-3 rounded-xl border border-border/60 bg-card/30 overflow-hidden">
            <div className="flex items-center justify-between px-4 py-3 border-b border-border/30">
              <div className="flex items-center gap-2.5">
                <div className="flex size-9 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500 dark:text-blue-400 text-sm font-bold">
                  {getInitials(expandedServer)}
                </div>
                <div>
                  <div className="text-sm font-semibold text-foreground">{expandedServer}</div>
                  <div className="text-[10px] text-muted-foreground/50">
                    {server.command} {(server.args ?? []).join(' ')}
                  </div>
                </div>
              </div>
              <button
                onClick={() => setExpandedServer(null)}
                className="rounded border border-border/40 px-2 py-0.5 text-[10px] text-muted-foreground/50 hover:border-primary hover:text-primary transition-colors"
              >
                Done
              </button>
            </div>
            <div className="px-4 py-3">
              <p className="text-[11px] text-muted-foreground/50">
                Tool permissions are configured via <code className="text-[10px] font-mono bg-muted/50 px-1 rounded">permissions.tools_allow</code> / <code className="text-[10px] font-mono bg-muted/50 px-1 rounded">tools_deny</code> using the pattern <code className="text-[10px] font-mono bg-muted/50 px-1 rounded">mcp__{server.name}__*</code>.
              </p>
              <p className="text-[10px] text-muted-foreground/30 mt-2">
                Connect via CLI to discover available tools from this server.
              </p>
            </div>
          </div>
        )
      })()}
    </SectionShell>
  )
}
