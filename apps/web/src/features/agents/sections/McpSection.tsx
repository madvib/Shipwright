import { useState } from 'react'
import { Grid3X3 } from 'lucide-react'
import type { McpServerConfig } from '@ship/ui'
import type { ToolToggleState, ToolPermission } from '../types'
import { MCP_TOOL_REGISTRY } from '../types'
import { SectionShell, Chip, ChipIcon, AddChip } from './SectionShell'
import { McpToolPanel } from './McpToolPanel'

function getInitials(name: string): string {
  const words = name.split(/[-_\s]/)
  if (words.length >= 2) return (words[0][0] + words[1][0]).toUpperCase()
  return name.slice(0, 2).toUpperCase()
}

function getToolBadge(
  server: McpServerConfig,
  toolStates: ToolToggleState | undefined,
): { label: string; variant: 'on' | 'partial' | 'off' } {
  const tools = MCP_TOOL_REGISTRY[server.name]
  if (!tools || !toolStates) return { label: 'all', variant: 'on' }

  const total = tools.length
  const allowed = tools.filter(
    (t) => toolStates[t.name] === 'allow' || toolStates[t.name] === 'ask',
  ).length

  if (allowed === total) return { label: 'all', variant: 'on' }
  if (allowed === 0) return { label: '0/' + total, variant: 'off' }
  return { label: allowed + '/' + total, variant: 'partial' }
}

const BADGE_COLORS = {
  on: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
  partial: 'bg-primary/10 text-primary',
  off: 'bg-muted text-muted-foreground',
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
  toolStates,
  onRemove,
  onSetToolPermission,
  onSetGroupPermission,
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
        {servers.map((server) => {
          const badge = getToolBadge(server, toolStates[server.name])
          return (
            <Chip
              key={server.name}
              icon={<ChipIcon letters={getInitials(server.name)} variant="mcp" />}
              name={server.name}
              meta={`${server.command} ${(server.args ?? []).slice(0, 2).join(' ')} / ${server.server_type ?? 'stdio'}`}
              badge={
                <span className={`rounded px-1.5 py-0.5 text-[9px] font-medium ${BADGE_COLORS[badge.variant]}`}>
                  {badge.label}
                </span>
              }
              onClick={() =>
                setExpandedServer((prev) =>
                  prev === server.name ? null : server.name,
                )
              }
              active={expandedServer === server.name}
              ariaExpanded={expandedServer === server.name}
              onRemove={() => onRemove(server.name)}
            />
          )
        })}
        <AddChip label="Add server" onClick={onAdd} />
      </div>

      {/* Expanded tool toggle panel */}
      {expandedServer && (
        <McpToolPanel
          serverName={expandedServer}
          server={servers.find((s) => s.name === expandedServer)!}
          tools={MCP_TOOL_REGISTRY[expandedServer] ?? []}
          toolStates={toolStates[expandedServer] ?? {}}
          onSetPermission={(tool, perm) =>
            onSetToolPermission(expandedServer, tool, perm)
          }
          onSetGroupPermission={(tools, perm) =>
            onSetGroupPermission(expandedServer, tools, perm)
          }
          onClose={() => setExpandedServer(null)}
        />
      )}
    </SectionShell>
  )
}
