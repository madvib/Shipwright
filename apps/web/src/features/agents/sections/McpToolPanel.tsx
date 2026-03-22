import { useState, useMemo } from 'react'
import { Search } from 'lucide-react'
import type { McpServerConfig } from '@ship/ui'
import type { McpToolConfig, ToolPermission, ToolToggleState } from '../types'

// ── Three-state toggle ──────────────────────────────────────────────────────

const PERM_CYCLE: ToolPermission[] = ['deny', 'allow', 'ask']

function nextPermission(current: ToolPermission): ToolPermission {
  const idx = PERM_CYCLE.indexOf(current)
  return PERM_CYCLE[(idx + 1) % PERM_CYCLE.length]
}

function ToolToggle({
  permission,
  onChange,
}: {
  permission: ToolPermission
  onChange: (p: ToolPermission) => void
}) {
  const bg =
    permission === 'allow'
      ? 'bg-emerald-500'
      : permission === 'ask'
        ? 'bg-primary'
        : 'bg-muted'
  const knobPos =
    permission === 'deny' ? 'left-[2px]' : 'left-[14px]'

  const next = nextPermission(permission)

  return (
    <button
      onClick={() => onChange(next)}
      aria-label={`Permission: ${permission}. Click to change to ${next}`}
      className={`relative h-4 w-7 shrink-0 rounded-full transition-colors ${bg}`}
    >
      <span
        className={`absolute top-[2px] size-3 rounded-full bg-white transition-all ${knobPos}`}
      />
    </button>
  )
}

// ── Permission badge ────────────────────────────────────────────────────────

const PERM_BADGE_STYLES: Record<ToolPermission, string> = {
  allow: 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400',
  ask: 'bg-primary/10 text-primary',
  deny: 'bg-destructive/10 text-destructive',
}

// ── Tool group ──────────────────────────────────────────────────────────────

function ToolGroup({
  title,
  tools,
  toolStates,
  onSetPermission,
  onSetGroupPermission,
}: {
  title: string
  tools: McpToolConfig[]
  toolStates: ToolToggleState
  onSetPermission: (tool: string, perm: ToolPermission) => void
  onSetGroupPermission: (tools: string[], perm: ToolPermission) => void
}) {
  const toolNames = tools.map((t) => t.name)

  return (
    <div className="border-b border-border/30 last:border-b-0">
      <div className="flex items-center justify-between px-4 py-2">
        <span className="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground/50">
          {title}
        </span>
        <div className="flex gap-1.5">
          <button
            onClick={() => onSetGroupPermission(toolNames, 'allow')}
            className="rounded px-1.5 py-0.5 text-[10px] text-muted-foreground/40 hover:bg-primary/10 hover:text-primary transition-colors"
          >
            Enable all
          </button>
          <button
            onClick={() => onSetGroupPermission(toolNames, 'deny')}
            className="rounded px-1.5 py-0.5 text-[10px] text-muted-foreground/40 hover:bg-destructive/10 hover:text-destructive transition-colors"
          >
            Disable all
          </button>
        </div>
      </div>
      <div className="flex flex-col gap-px px-2 pb-2">
        {tools.map((tool) => {
          const perm = toolStates[tool.name] ?? 'deny'
          return (
            <div
              key={tool.name}
              className="flex items-center gap-2.5 rounded-md px-2 py-1.5 hover:bg-muted/40 transition-colors"
            >
              <ToolToggle
                permission={perm}
                onChange={(p) => onSetPermission(tool.name, p)}
              />
              <span
                className={`flex-1 font-mono text-[11px] min-w-0 truncate ${
                  perm === 'deny'
                    ? 'text-muted-foreground/40 line-through'
                    : 'text-foreground/80'
                }`}
              >
                {tool.name}
              </span>
              <span className="hidden sm:block text-[10px] text-muted-foreground/40 max-w-[240px] truncate">
                {tool.description}
              </span>
              <span
                className={`shrink-0 rounded px-1.5 py-0.5 text-[9px] font-medium ${PERM_BADGE_STYLES[perm]}`}
              >
                {perm}
              </span>
            </div>
          )
        })}
      </div>
    </div>
  )
}

// ── Main panel ──────────────────────────────────────────────────────────────

interface McpToolPanelProps {
  serverName: string
  server: McpServerConfig
  tools: McpToolConfig[]
  toolStates: ToolToggleState
  onSetPermission: (tool: string, perm: ToolPermission) => void
  onSetGroupPermission: (tools: string[], perm: ToolPermission) => void
  onClose?: () => void
}

export function McpToolPanel({
  serverName,
  server,
  tools,
  toolStates,
  onSetPermission,
  onSetGroupPermission,
  onClose,
}: McpToolPanelProps) {
  const [query, setQuery] = useState('')

  const filtered = useMemo(() => {
    if (!query) return tools
    const q = query.toLowerCase()
    return tools.filter(
      (t) =>
        t.name.toLowerCase().includes(q) ||
        t.description.toLowerCase().includes(q),
    )
  }, [tools, query])

  const groups = useMemo(() => {
    const read = filtered.filter((t) => t.group === 'read')
    const write = filtered.filter((t) => t.group === 'write')
    const admin = filtered.filter((t) => t.group === 'admin')
    return { read, write, admin }
  }, [filtered])

  const stats = useMemo(() => {
    let allowed = 0
    let ask = 0
    let denied = 0
    for (const t of tools) {
      const p = toolStates[t.name] ?? 'deny'
      if (p === 'allow') allowed++
      else if (p === 'ask') ask++
      else denied++
    }
    return { allowed, ask, denied }
  }, [tools, toolStates])

  const initials =
    serverName.split(/[-_\s]/).length >= 2
      ? (serverName.split(/[-_\s]/)[0][0] + serverName.split(/[-_\s]/)[1][0]).toUpperCase()
      : serverName.slice(0, 2).toUpperCase()

  return (
    <div className="mt-3 rounded-xl border border-border/60 bg-card/30 overflow-hidden">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-border/30">
        <div className="flex items-center gap-2.5">
          <div className="flex size-9 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500 dark:text-blue-400 text-sm font-bold">
            {initials}
          </div>
          <div>
            <div className="text-sm font-semibold text-foreground">{serverName}</div>
            <div className="text-[10px] text-muted-foreground/50">
              {server.command} {(server.args ?? []).join(' ')}
            </div>
          </div>
        </div>
        <div className="flex items-center gap-2 text-[11px] text-muted-foreground/60">
          <span className="text-emerald-500 font-medium">{stats.allowed + stats.ask}</span>
          <span>of {tools.length} tools active</span>
        </div>
      </div>

      {/* Search */}
      <div className="px-4 py-2 border-b border-border/30">
        <div className="flex items-center gap-2 rounded-md border border-border/40 bg-background/40 px-2.5 py-1.5">
          <Search className="size-3 text-muted-foreground/40 shrink-0" />
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Filter tools..."
            className="flex-1 bg-transparent text-[11px] text-foreground placeholder:text-muted-foreground/30 focus:outline-none min-w-0"
          />
        </div>
      </div>

      {/* Tool groups */}
      {groups.read.length > 0 && (
        <ToolGroup
          title="Read Operations"
          tools={groups.read}
          toolStates={toolStates}
          onSetPermission={onSetPermission}
          onSetGroupPermission={onSetGroupPermission}
        />
      )}
      {groups.write.length > 0 && (
        <ToolGroup
          title="Write Operations"
          tools={groups.write}
          toolStates={toolStates}
          onSetPermission={onSetPermission}
          onSetGroupPermission={onSetGroupPermission}
        />
      )}
      {groups.admin.length > 0 && (
        <ToolGroup
          title="Admin Operations"
          tools={groups.admin}
          toolStates={toolStates}
          onSetPermission={onSetPermission}
          onSetGroupPermission={onSetGroupPermission}
        />
      )}

      {/* Status bar */}
      <div className="flex items-center justify-between px-4 py-2.5 bg-muted/20 border-t border-border/30">
        <div className="flex items-center gap-3 text-[11px] text-muted-foreground/50">
          <span className="flex items-center gap-1">
            <span className="size-1.5 rounded-full bg-emerald-500" />
            {stats.allowed} allowed
          </span>
          <span className="flex items-center gap-1">
            <span className="size-1.5 rounded-full bg-primary" />
            {stats.ask} ask
          </span>
          <span className="flex items-center gap-1">
            <span className="size-1.5 rounded-full bg-destructive" />
            {stats.denied} denied
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          <button
            onClick={() => {
              for (const t of tools) {
                onSetPermission(t.name, 'allow')
              }
            }}
            className="rounded border border-border/40 px-2 py-0.5 text-[10px] text-muted-foreground/50 hover:border-primary hover:text-primary transition-colors"
          >
            Reset to defaults
          </button>
          {onClose && (
            <button
              onClick={onClose}
              className="rounded border border-border/40 px-2 py-0.5 text-[10px] text-muted-foreground/50 hover:border-primary hover:text-primary transition-colors"
            >
              Done
            </button>
          )}
        </div>
      </div>
    </div>
  )
}
