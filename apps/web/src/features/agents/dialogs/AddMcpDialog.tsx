import { useState } from 'react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { Badge } from '@ship/primitives'
import { Server } from 'lucide-react'
import type { McpServerConfig } from '@ship/ui'

const POPULAR_SERVERS = [
  { name: 'github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], desc: 'GitHub API — PRs, issues, code search' },
  { name: 'filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'], desc: 'File system read/write access' },
  { name: 'playwright', command: 'npx', args: ['-y', '@playwright/mcp@latest'], desc: 'Browser automation and testing' },
  { name: 'postgres', command: 'npx', args: ['-y', '@modelcontextprotocol/server-postgres'], desc: 'PostgreSQL read access' },
]

interface AddMcpDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  existingNames: string[]
  onAdd: (server: McpServerConfig) => void
}

export function AddMcpDialog({ open, onOpenChange, existingNames, onAdd }: AddMcpDialogProps) {
  const [mode, setMode] = useState<'browse' | 'custom'>('browse')
  const [name, setName] = useState('')
  const [command, setCommand] = useState('')
  const [args, setArgs] = useState('')

  const available = POPULAR_SERVERS.filter((s) => !existingNames.includes(s.name))

  const addPopular = (server: typeof POPULAR_SERVERS[0]) => {
    onAdd({
      name: server.name,
      command: server.command,
      args: server.args,
      url: null,
      timeout_secs: null,
      server_type: 'stdio',
      codex_enabled_tools: [],
      codex_disabled_tools: [],
      gemini_include_tools: [],
      gemini_exclude_tools: [],
    })
    onOpenChange(false)
  }

  const addCustom = () => {
    if (!name.trim() || !command.trim()) return
    onAdd({
      name: name.trim(),
      command: command.trim(),
      args: args.split(' ').filter(Boolean),
      url: null,
      timeout_secs: null,
      server_type: 'stdio',
      codex_enabled_tools: [],
      codex_disabled_tools: [],
      gemini_include_tools: [],
      gemini_exclude_tools: [],
    })
    onOpenChange(false)
    setName('')
    setCommand('')
    setArgs('')
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Add MCP server</DialogTitle>
          <DialogDescription>Add a tool server to this agent's configuration.</DialogDescription>
        </DialogHeader>

        <div className="flex gap-2 mb-4">
          <button
            onClick={() => setMode('browse')}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium transition ${mode === 'browse' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground'}`}
          >
            Popular
          </button>
          <button
            onClick={() => setMode('custom')}
            className={`rounded-lg px-3 py-1.5 text-xs font-medium transition ${mode === 'custom' ? 'bg-primary/10 text-primary' : 'text-muted-foreground hover:text-foreground'}`}
          >
            Custom
          </button>
        </div>

        {mode === 'browse' ? (
          <div className="space-y-2 max-h-64 overflow-auto">
            {available.map((s) => (
              <button
                key={s.name}
                onClick={() => addPopular(s)}
                className="w-full flex items-center gap-3 rounded-lg border border-border/60 p-3 text-left hover:border-primary/30 transition"
              >
                <div className="size-8 rounded-lg bg-blue-500/10 flex items-center justify-center shrink-0">
                  <Server className="size-4 text-blue-500" />
                </div>
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-foreground">{s.name}</span>
                    <Badge variant="secondary" className="text-[9px]">stdio</Badge>
                  </div>
                  <p className="text-xs text-muted-foreground truncate">{s.desc}</p>
                </div>
              </button>
            ))}
            {available.length === 0 && (
              <p className="text-xs text-muted-foreground text-center py-4">All popular servers already added</p>
            )}
          </div>
        ) : (
          <div className="space-y-3">
            <div>
              <label className="text-xs font-medium text-foreground mb-1.5 block">Server name</label>
              <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="e.g. my-server" />
            </div>
            <div>
              <label className="text-xs font-medium text-foreground mb-1.5 block">Command</label>
              <Input value={command} onChange={(e) => setCommand(e.target.value)} placeholder="e.g. npx" />
            </div>
            <div>
              <label className="text-xs font-medium text-foreground mb-1.5 block">Arguments</label>
              <Input value={args} onChange={(e) => setArgs(e.target.value)} placeholder="e.g. -y @org/server" />
            </div>
          </div>
        )}

        {mode === 'custom' && (
          <DialogFooter>
            <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button onClick={addCustom} disabled={!name.trim() || !command.trim()}>Add server</Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  )
}
