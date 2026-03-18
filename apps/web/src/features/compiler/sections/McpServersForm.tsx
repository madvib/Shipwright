import { useState } from 'react'
import { Layers, Server } from 'lucide-react'
import { McpServerListEditor, McpRegistryBrowser } from '@ship/ui'
import type { McpServerConfig } from '@ship/ui'

interface Props {
  servers: McpServerConfig[]
  onChange: (servers: McpServerConfig[]) => void
}

type McpTab = 'manual' | 'registry'

export function McpServersForm({ servers, onChange }: Props) {
  const [tab, setTab] = useState<McpTab>('manual')

  const addedIds = new Set(servers.map((s) => s.name))

  const addFromRegistry = (server: McpServerConfig) => {
    if (servers.some((s) => s.name === server.name)) return
    onChange([...servers, server])
  }

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-0.5 rounded-lg bg-muted/50 p-0.5">
        <button
          onClick={() => setTab('manual')}
          className={`flex-1 rounded-md py-1.5 text-xs font-medium transition ${
            tab === 'manual' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          My servers
        </button>
        <button
          onClick={() => setTab('registry')}
          className={`flex flex-1 items-center justify-center gap-1.5 rounded-md py-1.5 text-xs font-medium transition ${
            tab === 'registry' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'
          }`}
        >
          <Layers className="size-3" />
          Discover
        </button>
      </div>

      {tab === 'manual' ? (
        servers.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed border-border/60 py-8 px-4 text-center">
            <Server className="size-5 text-muted-foreground/40" />
            <p className="text-xs text-muted-foreground">No MCP servers yet. Add one or browse the registry.</p>
          </div>
        ) : (
          <McpServerListEditor servers={servers} onChange={onChange} />
        )
      ) : (
        <McpRegistryBrowser onAdd={addFromRegistry} addedIds={addedIds} />
      )}
    </div>
  )
}
