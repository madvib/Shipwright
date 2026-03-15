import { useState } from 'react'
import { Plus, Trash2, ChevronDown, ChevronRight, Server } from 'lucide-react'
import type { McpServerConfig } from '../types'

interface Props {
  servers: McpServerConfig[]
  onChange: (servers: McpServerConfig[]) => void
}

const EMPTY: McpServerConfig = {
  name: '',
  command: '',
  args: [],
  env: {},
  server_type: 'stdio',
  scope: 'project',
  disabled: false,
  url: null,
  timeout_secs: null,
}

const COMMAND_SUGGESTIONS = ['npx', 'uvx', 'docker', 'node', 'python', 'ship', 'deno']

export function McpServerListEditor({ servers, onChange }: Props) {
  const [expanded, setExpanded] = useState<number | null>(null)

  const add = () => {
    const next = [...servers, { ...EMPTY }]
    onChange(next)
    setExpanded(next.length - 1)
  }

  const remove = (idx: number) => {
    onChange(servers.filter((_, i) => i !== idx))
    if (expanded === idx) setExpanded(null)
  }

  const update = (idx: number, patch: Partial<McpServerConfig>) => {
    onChange(servers.map((s, i) => (i === idx ? { ...s, ...patch } : s)))
  }

  return (
    <div className="space-y-2">
      {servers.length === 0 && (
        <p className="rounded-lg border border-dashed border-border/60 p-4 text-center text-xs text-muted-foreground">
          No MCP servers added yet.
        </p>
      )}

      {servers.map((server, idx) => (
        <ServerRow
          key={idx}
          server={server}
          isExpanded={expanded === idx}
          onToggle={() => setExpanded(expanded === idx ? null : idx)}
          onUpdate={(patch) => update(idx, patch)}
          onRemove={() => remove(idx)}
        />
      ))}

      <button
        onClick={add}
        className="flex w-full items-center justify-center gap-1.5 rounded-xl border border-dashed border-primary/30 bg-primary/5 py-2.5 text-xs font-medium text-primary transition hover:bg-primary/10"
      >
        <Plus className="size-3.5" />
        Add MCP server
      </button>
    </div>
  )
}

function ServerRow({
  server,
  isExpanded,
  onToggle,
  onUpdate,
  onRemove,
}: {
  server: McpServerConfig
  isExpanded: boolean
  onToggle: () => void
  onUpdate: (patch: Partial<McpServerConfig>) => void
  onRemove: () => void
}) {
  const argsStr = (server.args ?? []).join(' ')
  const envEntries = Object.entries(server.env ?? {})

  return (
    <div className={`overflow-hidden rounded-xl border transition ${isExpanded ? 'border-border bg-card' : 'border-border/60 bg-card/50'}`}>
      <div className="flex items-center gap-2 px-3 py-2.5">
        <button onClick={onToggle} className="flex flex-1 items-center gap-2 text-left min-w-0">
          <Server className="size-3.5 shrink-0 text-muted-foreground" />
          <span className="min-w-0 flex-1 truncate text-xs font-medium">
            {server.name || server.command || <span className="text-muted-foreground italic">Unnamed server</span>}
          </span>
          {server.server_type && server.server_type !== 'stdio' && (
            <span className="rounded bg-muted px-1.5 py-0.5 text-[9px] text-muted-foreground uppercase">
              {server.server_type}
            </span>
          )}
          {isExpanded ? (
            <ChevronDown className="size-3.5 shrink-0 text-muted-foreground" />
          ) : (
            <ChevronRight className="size-3.5 shrink-0 text-muted-foreground" />
          )}
        </button>
        <button
          onClick={onRemove}
          className="flex size-6 shrink-0 items-center justify-center rounded text-muted-foreground/60 transition hover:bg-destructive/10 hover:text-destructive"
        >
          <Trash2 className="size-3" />
        </button>
      </div>

      {isExpanded && (
        <div className="border-t border-border/60 bg-muted/20 p-3 space-y-3">
          <div className="grid gap-3 sm:grid-cols-2">
            <Field label="Server name" hint="Key in provider config">
              <TextInput
                value={server.name}
                onChange={(v) => onUpdate({ name: v })}
                placeholder="e.g. github"
              />
            </Field>
            <Field label="Transport">
              <select
                value={server.server_type ?? 'stdio'}
                onChange={(e) => onUpdate({ server_type: e.target.value as McpServerConfig['server_type'] })}
                className="h-7 w-full rounded-md border border-border bg-background px-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary/40"
              >
                <option value="stdio">stdio (local process)</option>
                <option value="sse">SSE (remote)</option>
                <option value="http">HTTP (remote)</option>
              </select>
            </Field>
          </div>

          {(server.server_type ?? 'stdio') === 'stdio' ? (
            <div className="grid gap-3 sm:grid-cols-2">
              <Field label="Command">
                <CommandInput
                  value={server.command}
                  onChange={(v) => onUpdate({ command: v })}
                  suggestions={COMMAND_SUGGESTIONS}
                />
              </Field>
              <Field label="Arguments" hint="Space-separated">
                <TextInput
                  value={argsStr}
                  onChange={(v) => onUpdate({ args: v.trim() ? v.trim().split(/\s+/) : [] })}
                  placeholder="e.g. -y @modelcontextprotocol/server-github"
                  mono
                />
              </Field>
            </div>
          ) : (
            <Field label="URL">
              <TextInput
                value={server.url ?? ''}
                onChange={(v) => onUpdate({ url: v || null })}
                placeholder="https://..."
              />
            </Field>
          )}

          <EnvEditor
            entries={envEntries}
            onChange={(entries) => onUpdate({ env: Object.fromEntries(entries) })}
          />

          <div className="flex items-center gap-2">
            <input
              type="checkbox"
              id={`disabled-${server.name}-${Math.random()}`}
              checked={server.disabled ?? false}
              onChange={(e) => onUpdate({ disabled: e.target.checked })}
              className="accent-primary"
            />
            <label className="text-xs text-muted-foreground">
              Disabled (excluded from compiled output)
            </label>
          </div>
        </div>
      )}
    </div>
  )
}

function EnvEditor({
  entries,
  onChange,
}: {
  entries: [string, string][]
  onChange: (entries: [string, string][]) => void
}) {
  return (
    <div>
      <p className="mb-1.5 text-[11px] font-medium text-muted-foreground">Environment variables</p>
      <div className="space-y-1.5">
        {entries.map(([k, v], idx) => (
          <div key={idx} className="flex gap-1.5">
            <TextInput
              value={k}
              onChange={(nk) => onChange(entries.map((e, i) => (i === idx ? [nk, e[1]] : e)))}
              placeholder="KEY"
              mono
              className="flex-1"
            />
            <TextInput
              value={v}
              onChange={(nv) => onChange(entries.map((e, i) => (i === idx ? [e[0], nv] : e)))}
              placeholder="$VALUE or value"
              mono
              className="flex-1"
            />
            <button
              onClick={() => onChange(entries.filter((_, i) => i !== idx))}
              className="flex size-7 shrink-0 items-center justify-center rounded text-muted-foreground/60 hover:bg-destructive/10 hover:text-destructive"
            >
              <Trash2 className="size-3" />
            </button>
          </div>
        ))}
        <button
          onClick={() => onChange([...entries, ['', '']])}
          className="flex items-center gap-1 text-[11px] text-muted-foreground transition hover:text-foreground"
        >
          <Plus className="size-3" /> Add variable
        </button>
      </div>
    </div>
  )
}

function Field({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <div className="space-y-1">
      <label className="block text-[11px] font-medium text-muted-foreground">
        {label}
        {hint && <span className="ml-1 font-normal opacity-60">— {hint}</span>}
      </label>
      {children}
    </div>
  )
}

function TextInput({
  value,
  onChange,
  placeholder,
  mono,
  className = '',
}: {
  value: string
  onChange: (v: string) => void
  placeholder?: string
  mono?: boolean
  className?: string
}) {
  return (
    <input
      type="text"
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      autoCorrect="off"
      autoCapitalize="none"
      spellCheck={false}
      className={`h-7 w-full rounded-md border border-border bg-background px-2 text-xs placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-primary/40 ${mono ? 'font-mono' : ''} ${className}`}
    />
  )
}

function CommandInput({
  value,
  onChange,
  suggestions,
}: {
  value: string
  onChange: (v: string) => void
  suggestions: string[]
}) {
  return (
    <div className="relative">
      <input
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder="e.g. npx"
        list="mcp-cmd-suggestions"
        autoCorrect="off"
        autoCapitalize="none"
        spellCheck={false}
        className="h-7 w-full rounded-md border border-border bg-background px-2 font-mono text-xs placeholder:text-muted-foreground/60 focus:outline-none focus:ring-1 focus:ring-primary/40"
      />
      <datalist id="mcp-cmd-suggestions">
        {suggestions.map((s) => <option key={s} value={s} />)}
      </datalist>
    </div>
  )
}
