import { useState } from 'react'
import { X, Plus } from 'lucide-react'
import type { Permissions } from '#/features/compiler/types'
import { DEFAULT_PERMISSIONS } from '#/features/compiler/types'

type Preset = 'strict' | 'default' | 'permissive' | 'custom'

const PRESETS: Record<Preset, Permissions> = {
  strict: {
    tools: { allow: [], deny: [] },
    filesystem: { allow: [], deny: ['**/*'] },
    commands: { allow: [], deny: [] },
    network: { policy: 'none', allow_hosts: [] },
    agent: { require_confirmation: ['*'] },
  },
  default: DEFAULT_PERMISSIONS,
  permissive: {
    tools: { allow: ['*'], deny: [] },
    filesystem: { allow: ['**/*'], deny: [] },
    commands: { allow: ['*'], deny: [] },
    network: { policy: 'unrestricted', allow_hosts: [] },
    agent: { require_confirmation: [] },
  },
  custom: DEFAULT_PERMISSIONS,
}

function detectPreset(p: Permissions): Preset {
  if (JSON.stringify(p) === JSON.stringify(PRESETS.strict)) return 'strict'
  if (JSON.stringify(p) === JSON.stringify(PRESETS.default)) return 'default'
  if (JSON.stringify(p) === JSON.stringify(PRESETS.permissive)) return 'permissive'
  return 'custom'
}

interface PermissionsEditorProps {
  permissions: Permissions
  onChange: (p: Permissions) => void
}

export function PermissionsEditor({ permissions, onChange }: PermissionsEditorProps) {
  const preset = detectPreset(permissions)
  const [allowInput, setAllowInput] = useState('')
  const [denyInput, setDenyInput] = useState('')

  const setPreset = (p: Preset) => {
    if (p === 'custom') return
    onChange(PRESETS[p])
  }

  const addAllow = () => {
    const val = allowInput.trim()
    if (!val) return
    onChange({ ...permissions, tools: { ...permissions.tools, allow: [...permissions.tools.allow, val] } })
    setAllowInput('')
  }

  const removeAllow = (rule: string) => {
    onChange({ ...permissions, tools: { ...permissions.tools, allow: permissions.tools.allow.filter((r) => r !== rule) } })
  }

  const addDeny = () => {
    const val = denyInput.trim()
    if (!val) return
    onChange({ ...permissions, tools: { ...permissions.tools, deny: [...permissions.tools.deny, val] } })
    setDenyInput('')
  }

  const removeDeny = (rule: string) => {
    onChange({ ...permissions, tools: { ...permissions.tools, deny: permissions.tools.deny.filter((r) => r !== rule) } })
  }

  const PRESET_LABELS: { id: Preset; label: string }[] = [
    { id: 'strict', label: 'Strict' },
    { id: 'default', label: 'Default' },
    { id: 'permissive', label: 'Permissive' },
    { id: 'custom', label: 'Custom' },
  ]

  return (
    <div className="space-y-5">
      {/* Presets */}
      <div>
        <p className="mb-2 text-[9px] font-semibold uppercase tracking-wider text-muted-foreground">Preset</p>
        <div className="flex gap-2">
          {PRESET_LABELS.map(({ id, label }) => (
            <button
              key={id}
              onClick={() => setPreset(id)}
              disabled={id === 'custom' && preset !== 'custom'}
              className={`rounded-md border px-3 py-1.5 text-xs font-medium transition ${
                preset === id
                  ? 'border-primary/40 bg-primary/10 text-primary'
                  : 'border-border/60 text-muted-foreground hover:border-border hover:text-foreground'
              } disabled:opacity-40 disabled:cursor-default`}
            >
              {label}{preset === id && ' ✓'}
            </button>
          ))}
        </div>
      </div>

      {/* Allow rules */}
      <div>
        <p className="mb-2 text-[9px] font-semibold uppercase tracking-wider text-emerald-600 dark:text-emerald-400">Allow rules</p>
        <div className="flex flex-wrap gap-1.5 mb-2">
          {permissions.tools.allow.map((rule) => (
            <span
              key={rule}
              className="flex items-center gap-1 rounded border border-emerald-500/30 bg-emerald-500/10 px-2 py-0.5 font-mono text-[10px] text-emerald-700 dark:text-emerald-300"
            >
              {rule}
              <button onClick={() => removeAllow(rule)} className="transition hover:text-emerald-500">
                <X className="size-2.5" />
              </button>
            </span>
          ))}
        </div>
        <div className="flex items-center gap-2 rounded-md border border-border/60 bg-muted/30 pl-2 pr-1 py-1">
          <span className="text-xs text-emerald-500">+</span>
          <input
            value={allowInput}
            onChange={(e) => setAllowInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && addAllow()}
            placeholder="Bash(git:*)"
            className="flex-1 bg-transparent font-mono text-[11px] text-foreground placeholder-muted-foreground/40 focus:outline-none"
          />
          <button
            onClick={addAllow}
            disabled={!allowInput.trim()}
            className="flex size-5 items-center justify-center rounded text-muted-foreground transition hover:text-foreground disabled:opacity-30"
          >
            <Plus className="size-3" />
          </button>
        </div>
      </div>

      {/* Deny rules */}
      <div>
        <p className="mb-2 text-[9px] font-semibold uppercase tracking-wider text-rose-600 dark:text-rose-400">Deny rules</p>
        <div className="flex flex-wrap gap-1.5 mb-2">
          {permissions.tools.deny.map((rule) => (
            <span
              key={rule}
              className="flex items-center gap-1 rounded border border-rose-500/30 bg-rose-500/10 px-2 py-0.5 font-mono text-[10px] text-rose-700 dark:text-rose-300"
            >
              {rule}
              <button onClick={() => removeDeny(rule)} className="transition hover:text-rose-500">
                <X className="size-2.5" />
              </button>
            </span>
          ))}
          {permissions.tools.deny.length === 0 && (
            <span className="text-[10px] text-muted-foreground/50 italic">No deny rules</span>
          )}
        </div>
        <div className="flex items-center gap-2 rounded-md border border-border/60 bg-muted/30 pl-2 pr-1 py-1">
          <span className="text-xs text-rose-500">+</span>
          <input
            value={denyInput}
            onChange={(e) => setDenyInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && addDeny()}
            placeholder="Bash(rm -rf:*)"
            className="flex-1 bg-transparent font-mono text-[11px] text-foreground placeholder-muted-foreground/40 focus:outline-none"
          />
          <button
            onClick={addDeny}
            disabled={!denyInput.trim()}
            className="flex size-5 items-center justify-center rounded text-muted-foreground transition hover:text-foreground disabled:opacity-30"
          >
            <Plus className="size-3" />
          </button>
        </div>
      </div>
    </div>
  )
}
