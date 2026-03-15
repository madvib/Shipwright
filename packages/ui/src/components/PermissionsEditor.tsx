import { Shield, FileSearch, ShieldAlert } from 'lucide-react'
import type { Permissions } from '../types'

interface Props {
  permissions: Permissions
  onChange: (p: Permissions) => void
}

const PRESETS = [
  {
    id: 'readonly',
    name: 'Read-only',
    description: 'Read files and read-only MCP tools only.',
    icon: FileSearch,
    color: 'text-blue-500',
    get: (): Permissions => ({
      tools: {
        // Explicit allowlist: only named read tools work. Bash, Write, Edit etc. are blocked.
        allow: ['Read', 'Glob', 'Grep', 'LS', 'mcp__*__read*', 'mcp__*__list*', 'mcp__*__get*', 'mcp__*__search*'],
        deny: [],
      },
      filesystem: { allow: ['**/*'], deny: [] },
      commands: { allow: [], deny: ['*'] },
      network: { policy: 'none', allow_hosts: [] },
      agent: { require_confirmation: [] },
    }),
  },
  {
    id: 'guarded',
    name: 'Ship Guarded',
    description: 'All tools available. Destructive ops require confirmation.',
    icon: Shield,
    color: 'text-emerald-500',
    get: (): Permissions => ({
      tools: {
        // No allow list — an explicit allowlist silently blocks everything not named.
        // Use deny for specific dangerous MCP ops only.
        allow: ['*'],
        deny: ['mcp__*__delete*', 'mcp__*__drop*', 'mcp__*__destroy*'],
      },
      filesystem: { allow: ['**/*'], deny: ['/etc/**', '/sys/**', '~/.ssh/**', '~/.aws/**'] },
      commands: { allow: ['*'], deny: ['rm -rf *', 'git push --force*', 'sudo *', 'curl * | *sh*'] },
      network: { policy: 'none', allow_hosts: [] },
      agent: { require_confirmation: ['Bash', 'Write', 'Edit', 'MultiEdit'] },
    }),
  },
  {
    id: 'full',
    name: 'Full Access',
    description: 'No restrictions. Use only in trusted environments.',
    icon: ShieldAlert,
    color: 'text-rose-500',
    get: (): Permissions => ({
      tools: { allow: ['*'], deny: [] },
      filesystem: { allow: ['**/*'], deny: [] },
      commands: { allow: ['*'], deny: [] },
      network: { policy: 'unrestricted', allow_hosts: [] },
      agent: { require_confirmation: [] },
    }),
  },
]

const NETWORK_POLICIES = ['none', 'localhost', 'allow-list', 'unrestricted'] as const

const TOOL_SUGGESTIONS = [
  '*', 'Read', 'Write', 'Edit', 'MultiEdit', 'Bash', 'Glob', 'Grep',
  'mcp__*__*', 'mcp__*__read*', 'mcp__*__write*', 'mcp__*__delete*', 'mcp__ship__*',
]

const COMMAND_SUGGESTIONS = [
  'git *', 'cargo *', 'npm *', 'pnpm *', 'python *',
  'ls', 'cat', 'rm -rf *', 'git push --force',
]

const FILESYSTEM_SUGGESTIONS = [
  '**/*', 'src/**', 'docs/**', '~/.ssh/**', '/etc/**', '/sys/**', '/proc/**',
]

export function PermissionsEditor({ permissions, onChange }: Props) {
  const update = (patch: Partial<Permissions>) => onChange({ ...permissions, ...patch })

  return (
    <div className="space-y-5">
      <div>
        <p className="mb-2 text-xs font-semibold text-foreground">Quick presets</p>
        <div className="grid gap-2 sm:grid-cols-3">
          {PRESETS.map(({ id, name, description, icon: Icon, color, get }) => (
            <button
              key={id}
              onClick={() => onChange(get())}
              className="flex flex-col items-start gap-1.5 rounded-xl border border-border/60 bg-card p-3 text-left transition hover:border-border"
            >
              <Icon className={`size-4 ${color}`} />
              <span className="text-xs font-semibold">{name}</span>
              <span className="text-[10px] leading-tight text-muted-foreground">{description}</span>
            </button>
          ))}
        </div>
      </div>

      <PatternSection
        title="Tools"
        subtitle="Allow or deny specific tools by name or glob."
        allow={permissions.tools.allow}
        deny={permissions.tools.deny}
        suggestions={TOOL_SUGGESTIONS}
        sectionKey="tools"
        onAllow={(allow) => update({ tools: { ...permissions.tools, allow } })}
        onDeny={(deny) => update({ tools: { ...permissions.tools, deny } })}
      />

      <PatternSection
        title="Filesystem"
        subtitle="Glob patterns for allowed/denied file paths."
        allow={permissions.filesystem.allow}
        deny={permissions.filesystem.deny}
        suggestions={FILESYSTEM_SUGGESTIONS}
        sectionKey="fs"
        onAllow={(allow) => update({ filesystem: { ...permissions.filesystem, allow } })}
        onDeny={(deny) => update({ filesystem: { ...permissions.filesystem, deny } })}
      />

      <PatternSection
        title="Shell commands"
        subtitle="Shell command patterns the agent may or may not run."
        allow={permissions.commands.allow}
        deny={permissions.commands.deny}
        suggestions={COMMAND_SUGGESTIONS}
        sectionKey="cmd"
        onAllow={(allow) => update({ commands: { ...permissions.commands, allow } })}
        onDeny={(deny) => update({ commands: { ...permissions.commands, deny } })}
      />

      <div className="rounded-xl border border-border/60 bg-card/50 p-3.5 space-y-3">
        <p className="text-xs font-semibold">Network</p>
        <div className="flex items-center gap-2">
          <label className="text-[11px] text-muted-foreground w-16 shrink-0">Policy</label>
          <select
            value={permissions.network.policy}
            onChange={(e) =>
              update({
                network: {
                  ...permissions.network,
                  policy: e.target.value as Permissions['network']['policy'],
                },
              })
            }
            className="h-7 rounded-md border border-border bg-background px-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary/40"
          >
            {NETWORK_POLICIES.map((p) => (
              <option key={p} value={p}>{p}</option>
            ))}
          </select>
        </div>
        {permissions.network.policy === 'allow-list' && (
          <div className="space-y-1">
            <label className="text-[11px] text-muted-foreground">Allowed hosts (one per line)</label>
            <textarea
              value={permissions.network.allow_hosts.join('\n')}
              onChange={(e) =>
                update({
                  network: {
                    ...permissions.network,
                    allow_hosts: e.target.value.split('\n').map((h) => h.trim()).filter(Boolean),
                  },
                })
              }
              rows={3}
              className="w-full resize-y rounded-md border border-border bg-background p-2 font-mono text-xs focus:outline-none focus:ring-1 focus:ring-primary/40"
              placeholder="api.example.com"
            />
          </div>
        )}
      </div>
    </div>
  )
}

function PatternSection({
  title,
  subtitle,
  allow,
  deny,
  suggestions,
  sectionKey,
  onAllow,
  onDeny,
}: {
  title: string
  subtitle: string
  allow: string[]
  deny: string[]
  suggestions: string[]
  sectionKey: string
  onAllow: (v: string[]) => void
  onDeny: (v: string[]) => void
}) {
  return (
    <div className="rounded-xl border border-border/60 bg-card/50 p-3.5 space-y-3">
      <div>
        <p className="text-xs font-semibold">{title}</p>
        <p className="text-[11px] text-muted-foreground">{subtitle}</p>
      </div>
      <div className="grid gap-3 sm:grid-cols-2">
        <PatternList label="Allow" patterns={allow} suggestions={suggestions} color="emerald" listId={`${sectionKey}-allow`} onChange={onAllow} />
        <PatternList label="Deny" patterns={deny} suggestions={suggestions} color="rose" listId={`${sectionKey}-deny`} onChange={onDeny} />
      </div>
    </div>
  )
}

function PatternList({
  label,
  patterns,
  suggestions,
  color,
  listId,
  onChange,
}: {
  label: string
  patterns: string[]
  suggestions: string[]
  color: 'emerald' | 'rose'
  listId: string
  onChange: (v: string[]) => void
}) {
  const isEmerald = color === 'emerald'
  const colorClass = isEmerald
    ? 'border-emerald-500/20 bg-emerald-500/5 text-emerald-600 dark:text-emerald-400'
    : 'border-rose-500/20 bg-rose-500/5 text-rose-600 dark:text-rose-400'
  const labelClass = isEmerald ? 'text-emerald-600 dark:text-emerald-400' : 'text-rose-600 dark:text-rose-400'

  return (
    <div className="space-y-1.5">
      <p className={`text-[10px] font-semibold uppercase tracking-wide ${labelClass}`}>{label}</p>
      <div className="space-y-1">
        {patterns.map((p, idx) => (
          <div key={idx} className="flex items-center gap-1">
            <input
              list={listId}
              value={p}
              onChange={(e) => onChange(patterns.map((v, i) => (i === idx ? e.target.value : v)))}
              autoCorrect="off"
              autoCapitalize="none"
              spellCheck={false}
              className="h-6 flex-1 rounded border border-border bg-background px-2 font-mono text-[11px] focus:outline-none focus:ring-1 focus:ring-primary/40"
            />
            <button
              onClick={() => onChange(patterns.filter((_, i) => i !== idx))}
              className="flex size-5 shrink-0 items-center justify-center rounded text-muted-foreground/60 hover:text-destructive"
            >
              ×
            </button>
          </div>
        ))}
        <datalist id={listId}>
          {suggestions.map((s) => <option key={s} value={s} />)}
        </datalist>
        <button
          onClick={() => onChange([...patterns, ''])}
          className={`flex items-center gap-1 rounded border px-2 py-0.5 text-[11px] font-medium transition hover:opacity-80 ${colorClass}`}
        >
          + {label}
        </button>
      </div>
    </div>
  )
}
