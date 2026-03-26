import { useState } from 'react'
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription, DialogFooter,
} from '@ship/primitives'
import { Input } from '@ship/primitives'
import { Button } from '@ship/primitives'
import { Badge } from '@ship/primitives'
import { Server, Loader2, Globe } from 'lucide-react'
import type { McpServerConfig, McpServerType } from '@ship/ui'
import { EnvVarEditor } from './EnvVarEditor'
import { useRegistryMcpSearch } from '../useRegistryAutocomplete'
import type { RegistryMcpResult } from '../useRegistryAutocomplete'

const POPULAR_SERVERS = [
  { name: 'github', command: 'npx', args: ['-y', '@modelcontextprotocol/server-github'], desc: 'GitHub API — PRs, issues, code search' },
  { name: 'filesystem', command: 'npx', args: ['-y', '@modelcontextprotocol/server-filesystem', '.'], desc: 'File system read/write access' },
  { name: 'playwright', command: 'npx', args: ['-y', '@playwright/mcp@latest'], desc: 'Browser automation and testing' },
  { name: 'postgres', command: 'npx', args: ['-y', '@modelcontextprotocol/server-postgres'], desc: 'PostgreSQL read access' },
]

const SERVER_TYPES: McpServerType[] = ['stdio', 'sse', 'http']

interface EnvVar { key: string; value: string }

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
  const [serverType, setServerType] = useState<McpServerType>('stdio')
  const [url, setUrl] = useState('')
  const [envVars, setEnvVars] = useState<EnvVar[]>([])
  const [timeout, setTimeout] = useState('')
  const [disabled, setDisabled] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')

  const { results: registryResults, loading: registryLoading } = useRegistryMcpSearch(
    searchQuery,
    open && mode === 'browse',
  )

  const available = POPULAR_SERVERS.filter((s) => !existingNames.includes(s.name))
  const isRemote = serverType === 'sse' || serverType === 'http'

  const registryFiltered = registryResults.filter(
    (r) => !existingNames.includes(r.id) && !POPULAR_SERVERS.some((p) => p.name === r.id),
  )

  const addPopular = (server: typeof POPULAR_SERVERS[0]) => {
    onAdd({
      name: server.name, command: server.command, args: server.args,
      url: null, timeout_secs: null, server_type: 'stdio',
      codex_enabled_tools: [], codex_disabled_tools: [],
      gemini_include_tools: [], gemini_exclude_tools: [],
    })
    onOpenChange(false)
  }

  const addRegistryServer = (server: RegistryMcpResult) => {
    onAdd({
      name: server.id, command: server.command ?? 'npx', args: server.args,
      url: null, timeout_secs: null, server_type: 'stdio',
      codex_enabled_tools: [], codex_disabled_tools: [],
      gemini_include_tools: [], gemini_exclude_tools: [],
    })
    onOpenChange(false)
  }

  const buildEnvRecord = (): Record<string, string> | undefined => {
    const filtered = envVars.filter((e) => e.key.trim())
    if (filtered.length === 0) return undefined
    return Object.fromEntries(filtered.map((e) => [e.key.trim(), e.value]))
  }

  const addCustom = () => {
    if (!name.trim()) return
    if (isRemote && !url.trim()) return
    if (!isRemote && !command.trim()) return
    const timeoutNum = timeout ? parseInt(timeout, 10) : null
    const validTimeout = timeoutNum && !isNaN(timeoutNum) && timeoutNum > 0 ? timeoutNum : null
    onAdd({
      name: name.trim(),
      command: isRemote ? '' : command.trim(),
      args: isRemote ? [] : args.split(' ').filter(Boolean),
      url: isRemote ? url.trim() : null,
      timeout_secs: validTimeout, server_type: serverType,
      env: buildEnvRecord(), disabled: disabled || undefined,
      codex_enabled_tools: [], codex_disabled_tools: [],
      gemini_include_tools: [], gemini_exclude_tools: [],
    })
    onOpenChange(false)
    resetForm()
  }

  const resetForm = () => {
    setName(''); setCommand(''); setArgs('')
    setServerType('stdio'); setUrl(''); setEnvVars([])
    setTimeout(''); setDisabled(false)
  }

  const canSubmit = name.trim() && (isRemote ? url.trim() : command.trim())

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
          <BrowsePanel
            available={available} onAdd={addPopular}
            searchQuery={searchQuery} onSearchChange={setSearchQuery}
            registryResults={registryFiltered} registryLoading={registryLoading}
            onAddRegistry={addRegistryServer}
          />
        ) : (
          <CustomForm
            name={name} setName={setName}
            serverType={serverType} setServerType={setServerType}
            isRemote={isRemote}
            command={command} setCommand={setCommand}
            args={args} setArgs={setArgs}
            url={url} setUrl={setUrl}
            envVars={envVars} setEnvVars={setEnvVars}
            timeout={timeout} setTimeout={setTimeout}
            disabled={disabled} setDisabled={setDisabled}
          />
        )}

        {mode === 'custom' && (
          <DialogFooter>
            <Button variant="ghost" onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button onClick={addCustom} disabled={!canSubmit}>Add server</Button>
          </DialogFooter>
        )}
      </DialogContent>
    </Dialog>
  )
}

// ── Browse panel ──────────────────────────────────────────────────────────────

function BrowsePanel({ available, onAdd, searchQuery, onSearchChange, registryResults, registryLoading, onAddRegistry }: {
  available: typeof POPULAR_SERVERS
  onAdd: (s: typeof POPULAR_SERVERS[0]) => void
  searchQuery: string
  onSearchChange: (q: string) => void
  registryResults: RegistryMcpResult[]
  registryLoading: boolean
  onAddRegistry: (s: RegistryMcpResult) => void
}) {
  const filtered = available.filter(
    (s) => !searchQuery || s.name.toLowerCase().includes(searchQuery.toLowerCase()) || s.desc.toLowerCase().includes(searchQuery.toLowerCase()),
  )

  return (
    <div className="space-y-3 max-h-72 overflow-auto">
      <Input value={searchQuery} onChange={(e) => onSearchChange(e.target.value)} placeholder="Search MCP servers..." className="text-sm" />
      {filtered.map((s) => (
        <button key={s.name} onClick={() => onAdd(s)} className="w-full flex items-center gap-3 rounded-lg border border-border/60 p-3 text-left hover:border-primary/30 transition">
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
      {registryLoading && searchQuery.length >= 2 && (
        <div className="flex items-center justify-center gap-2 py-3">
          <Loader2 className="size-3.5 animate-spin text-muted-foreground" />
          <span className="text-xs text-muted-foreground">Searching registry...</span>
        </div>
      )}
      {registryResults.length > 0 && (
        <>
          <p className="text-[10px] font-medium text-muted-foreground/60 uppercase tracking-wider">Registry</p>
          {registryResults.map((s) => (
            <button key={s.id} onClick={() => onAddRegistry(s)} className="w-full flex items-center gap-3 rounded-lg border border-border/60 p-3 text-left hover:border-primary/30 transition">
              <div className="size-8 rounded-lg bg-emerald-500/10 flex items-center justify-center shrink-0">
                <Globe className="size-4 text-emerald-500" />
              </div>
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2">
                  <span className="text-sm font-medium text-foreground">{s.name}</span>
                  <Badge variant="secondary" className="text-[9px]">registry</Badge>
                </div>
                {s.description && <p className="text-xs text-muted-foreground truncate">{s.description}</p>}
              </div>
            </button>
          ))}
        </>
      )}
      {filtered.length === 0 && registryResults.length === 0 && !registryLoading && (
        <p className="text-xs text-muted-foreground text-center py-4">No servers found. Try a different search or add a custom server.</p>
      )}
    </div>
  )
}

// ── Custom form ───────────────────────────────────────────────────────────────

interface CustomFormProps {
  name: string; setName: (v: string) => void
  serverType: McpServerType; setServerType: (v: McpServerType) => void
  isRemote: boolean
  command: string; setCommand: (v: string) => void
  args: string; setArgs: (v: string) => void
  url: string; setUrl: (v: string) => void
  envVars: EnvVar[]; setEnvVars: (v: EnvVar[]) => void
  timeout: string; setTimeout: (v: string) => void
  disabled: boolean; setDisabled: (v: boolean) => void
}

function CustomForm(p: CustomFormProps) {
  return (
    <div className="space-y-3 max-h-80 overflow-auto pr-1">
      <div>
        <label className="text-xs font-medium text-foreground mb-1.5 block">Server name</label>
        <Input value={p.name} onChange={(e) => p.setName(e.target.value)} placeholder="e.g. my-server" />
      </div>
      <div>
        <label className="text-xs font-medium text-foreground mb-1.5 block">Server type</label>
        <div className="flex gap-1">
          {SERVER_TYPES.map((t) => (
            <button key={t} type="button" onClick={() => p.setServerType(t)}
              className={`rounded-md px-3 py-1.5 text-xs font-medium border transition ${p.serverType === t ? 'border-primary bg-primary/10 text-primary' : 'border-border text-muted-foreground hover:text-foreground hover:border-foreground/20'}`}>
              {t.toUpperCase()}
            </button>
          ))}
        </div>
      </div>
      {p.isRemote ? (
        <div>
          <label className="text-xs font-medium text-foreground mb-1.5 block">URL</label>
          <Input value={p.url} onChange={(e) => p.setUrl(e.target.value)} placeholder="https://example.com/mcp" />
        </div>
      ) : (
        <>
          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Command</label>
            <Input value={p.command} onChange={(e) => p.setCommand(e.target.value)} placeholder="e.g. npx" />
          </div>
          <div>
            <label className="text-xs font-medium text-foreground mb-1.5 block">Arguments</label>
            <Input value={p.args} onChange={(e) => p.setArgs(e.target.value)} placeholder="e.g. -y @org/server" />
          </div>
        </>
      )}
      <EnvVarEditor entries={p.envVars} onChange={p.setEnvVars} />
      <div className="flex gap-4 items-end">
        <div className="flex-1">
          <label className="text-xs font-medium text-foreground mb-1.5 block">Timeout (seconds)</label>
          <Input type="number" min={1} value={p.timeout} onChange={(e) => p.setTimeout(e.target.value)} placeholder="30" />
        </div>
        <label className="flex items-center gap-2 pb-2 cursor-pointer select-none">
          <input type="checkbox" checked={p.disabled} onChange={(e) => p.setDisabled(e.target.checked)} className="size-4 rounded border-border accent-primary" />
          <span className="text-xs font-medium text-muted-foreground">Disabled</span>
        </label>
      </div>
    </div>
  )
}
