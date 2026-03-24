import { useState, type ReactNode } from 'react'
import { Plus, X } from 'lucide-react'
import type { HookConfig, HookTrigger } from '@ship/ui'

interface ClaudeSettings {
  model?: string
  defaultMode?: string
  alwaysThinkingEnabled?: boolean
  fastMode?: boolean
  autoMemoryEnabled?: boolean
  language?: string
  cleanupPeriodDays?: number
  syntaxHighlightingDisabled?: boolean
  attribution?: { commit?: string; pr?: string }
  env?: Record<string, string>
}

interface Props {
  claudeSettingsExtra: Record<string, unknown>
  hooks: HookConfig[]
  onClaudeSettingsChange: (extra: Record<string, unknown>) => void
  onHooksChange: (hooks: HookConfig[]) => void
}

// Schema-aligned values from permissions.schema.json
const DEFAULT_MODES = [
  { id: '', label: '— not set —' },
  { id: 'default', label: 'default' },
  { id: 'plan', label: 'plan' },
  { id: 'acceptEdits', label: 'acceptEdits' },
  { id: 'dontAsk', label: 'dontAsk' },
  { id: 'bypassPermissions', label: 'bypassPermissions' },
]

const HOOK_TRIGGERS: { id: HookTrigger; label: string }[] = [
  { id: 'PreToolUse', label: 'PreToolUse' },
  { id: 'PostToolUse', label: 'PostToolUse' },
  { id: 'Stop', label: 'Stop' },
  { id: 'Notification', label: 'Notification' },
  { id: 'SubagentStop', label: 'SubagentStop' },
  { id: 'PreCompact', label: 'PreCompact' },
]

function uid() { return Math.random().toString(36).slice(2, 10) }

function parse(extra: Record<string, unknown>): ClaudeSettings {
  return {
    model: (extra.model as string) || undefined,
    defaultMode: (extra.defaultMode as string) || undefined,
    alwaysThinkingEnabled: extra.alwaysThinkingEnabled as boolean | undefined,
    fastMode: extra.fastMode as boolean | undefined,
    autoMemoryEnabled: extra.autoMemoryEnabled as boolean | undefined,
    language: (extra.language as string) || undefined,
    cleanupPeriodDays: extra.cleanupPeriodDays as number | undefined,
    syntaxHighlightingDisabled: extra.syntaxHighlightingDisabled as boolean | undefined,
    attribution: extra.attribution as { commit?: string; pr?: string } | undefined,
    env: extra.env as Record<string, string> | undefined,
  }
}

function serialize(s: ClaudeSettings): Record<string, unknown> {
  const o: Record<string, unknown> = {}
  if (s.model) o.model = s.model
  if (s.defaultMode) o.defaultMode = s.defaultMode
  if (s.alwaysThinkingEnabled !== undefined) o.alwaysThinkingEnabled = s.alwaysThinkingEnabled
  if (s.fastMode !== undefined) o.fastMode = s.fastMode
  if (s.autoMemoryEnabled !== undefined) o.autoMemoryEnabled = s.autoMemoryEnabled
  if (s.language) o.language = s.language
  if (s.cleanupPeriodDays !== undefined) o.cleanupPeriodDays = s.cleanupPeriodDays
  if (s.syntaxHighlightingDisabled !== undefined) o.syntaxHighlightingDisabled = s.syntaxHighlightingDisabled
  if (s.attribution?.commit || s.attribution?.pr) o.attribution = s.attribution
  if (s.env && Object.keys(s.env).length > 0) o.env = s.env
  return o
}

const sx = {
  select: 'h-7 rounded-md border border-border bg-background px-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary/40',
  input: 'h-7 rounded-md border border-border bg-background px-2 text-xs focus:outline-none focus:ring-1 focus:ring-primary/40 placeholder:text-muted-foreground/60',
  addBtn: 'flex h-7 w-7 shrink-0 items-center justify-center rounded-md bg-primary text-primary-foreground disabled:opacity-40 hover:opacity-90 transition',
  removeBtn: 'shrink-0 rounded p-1 text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition',
}

function Toggle({ value, onChange }: { value: boolean; onChange: (v: boolean) => void }) {
  return (
    <button type="button" role="switch" aria-checked={value} onClick={() => onChange(!value)}
      className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${value ? 'bg-primary' : 'bg-muted-foreground/30'}`}>
      <span className={`inline-block h-3.5 w-3.5 rounded-full bg-white shadow transition-transform ${value ? 'translate-x-4' : 'translate-x-0.5'}`} />
    </button>
  )
}

function Row({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex items-center gap-3 py-1.5">
      <label className="w-44 shrink-0 text-[11px] text-muted-foreground">{label}</label>
      {children}
    </div>
  )
}

function Block({ title, children }: { title: string; children: ReactNode }) {
  return (
    <div className="rounded-xl border border-border/60 bg-card/50 p-3.5 space-y-1">
      <p className="mb-2 text-xs font-semibold text-foreground">{title}</p>
      {children}
    </div>
  )
}

function HooksSection({ hooks, onChange }: { hooks: HookConfig[]; onChange: (h: HookConfig[]) => void }) {
  const [trigger, setTrigger] = useState<HookTrigger>('PreToolUse')
  const [matcher, setMatcher] = useState('')
  const [command, setCommand] = useState('')
  const add = () => {
    const cmd = command.trim()
    if (!cmd) return
    onChange([...hooks, { id: uid(), trigger, matcher: matcher.trim() || null, command: cmd }])
    setMatcher(''); setCommand('')
  }
  return (
    <div className="space-y-2">
      {hooks.map((h) => (
        <div key={h.id} className="flex items-start gap-2 rounded-lg border border-border/50 bg-card/40 px-3 py-2">
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <span className="rounded bg-primary/10 px-1.5 py-0.5 font-mono text-[10px] text-primary">{h.trigger}</span>
              {h.matcher && <span className="rounded bg-muted px-1.5 py-0.5 font-mono text-[10px] text-muted-foreground">{h.matcher}</span>}
            </div>
            <p className="mt-1 font-mono text-[11px] text-foreground/80 break-all">{h.command}</p>
          </div>
          <button onClick={() => onChange(hooks.filter((x) => x.id !== h.id))} className={sx.removeBtn}><X className="size-3" /></button>
        </div>
      ))}
      <div className="rounded-lg border border-dashed border-border/60 p-2.5 space-y-2">
        <div className="flex items-center gap-2">
          <select value={trigger} onChange={(e) => setTrigger(e.target.value as HookTrigger)} className={sx.select}>
            {HOOK_TRIGGERS.map((t) => <option key={t.id} value={t.id}>{t.label}</option>)}
          </select>
          <input value={matcher} onChange={(e) => setMatcher(e.target.value)} placeholder="matcher (optional)"
            className={`${sx.input} flex-1 font-mono`} />
        </div>
        <div className="flex items-center gap-2">
          <input value={command} onChange={(e) => setCommand(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && add()}
            placeholder="command" className={`${sx.input} flex-1 font-mono`} />
          <button onClick={add} disabled={!command.trim()} className={sx.addBtn}><Plus className="size-3" /></button>
        </div>
      </div>
    </div>
  )
}

function EnvSection({ env, onChange }: { env: Record<string, string>; onChange: (e: Record<string, string>) => void }) {
  const [k, setK] = useState(''); const [v, setV] = useState('')
  const add = () => {
    const key = k.trim()
    if (!key) return
    onChange({ ...env, [key]: v }); setK(''); setV('')
  }
  return (
    <div className="space-y-2">
      {Object.entries(env).map(([key, val]) => (
        <div key={key} className="flex items-center gap-2">
          <span className="w-36 shrink-0 rounded bg-muted px-2 py-1 font-mono text-[11px] truncate">{key}</span>
          <span className="flex-1 rounded bg-muted px-2 py-1 font-mono text-[11px] text-muted-foreground truncate">{val}</span>
          <button onClick={() => { const n = { ...env }; delete n[key]; onChange(n) }} className={sx.removeBtn}><X className="size-3" /></button>
        </div>
      ))}
      <div className="flex items-center gap-2">
        <input value={k} onChange={(e) => setK(e.target.value)} placeholder="KEY"
          className={`${sx.input} w-36 shrink-0 font-mono`} />
        <input value={v} onChange={(e) => setV(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && add()}
          placeholder="value" className={`${sx.input} flex-1 font-mono`} />
        <button onClick={add} disabled={!k.trim()} className={sx.addBtn}><Plus className="size-3" /></button>
      </div>
    </div>
  )
}

export function SettingsForm({ claudeSettingsExtra, hooks, onClaudeSettingsChange, onHooksChange }: Props) {
  const s = parse(claudeSettingsExtra)
  const upd = (patch: Partial<ClaudeSettings>) => onClaudeSettingsChange(serialize({ ...s, ...patch }))

  return (
    <div className="space-y-4">
      <Block title="Identity">
        <Row label="Model">
          <input value={s.model ?? ''} onChange={(e) => upd({ model: e.target.value || undefined })}
            placeholder="e.g. claude-sonnet-4-6" className={`${sx.input} flex-1 font-mono`} />
        </Row>
        <Row label="Language">
          <input value={s.language ?? ''} onChange={(e) => upd({ language: e.target.value || undefined })}
            placeholder="e.g. en, ja" className={`${sx.input} w-28`} />
        </Row>
      </Block>

      <Block title="Behavior">
        <Row label="Default mode">
          <select value={s.defaultMode ?? ''} onChange={(e) => upd({ defaultMode: e.target.value || undefined })} className={sx.select}>
            {DEFAULT_MODES.map((m) => <option key={m.id} value={m.id}>{m.label}</option>)}
          </select>
        </Row>
        <Row label="Always thinking"><Toggle value={s.alwaysThinkingEnabled ?? false} onChange={(v) => upd({ alwaysThinkingEnabled: v })} /></Row>
        <Row label="Fast mode"><Toggle value={s.fastMode ?? false} onChange={(v) => upd({ fastMode: v })} /></Row>
        <Row label="Auto memory"><Toggle value={s.autoMemoryEnabled ?? false} onChange={(v) => upd({ autoMemoryEnabled: v })} /></Row>
      </Block>

      <Block title="Hooks (Claude Code)">
        <HooksSection hooks={hooks} onChange={onHooksChange} />
      </Block>

      <Block title="Env vars (Claude Code)">
        <EnvSection env={(s.env ?? {}) as Record<string, string>} onChange={(env) => upd({ env })} />
      </Block>

      <Block title="Developer">
        <Row label="Attribution commit">
          <input value={s.attribution?.commit ?? ''} placeholder="leave blank to hide"
            onChange={(e) => upd({ attribution: { ...s.attribution, commit: e.target.value || undefined } })}
            className={`${sx.input} flex-1`} />
        </Row>
        <Row label="Attribution PR">
          <input value={s.attribution?.pr ?? ''} placeholder="leave blank to hide"
            onChange={(e) => upd({ attribution: { ...s.attribution, pr: e.target.value || undefined } })}
            className={`${sx.input} flex-1`} />
        </Row>
        <Row label="Cleanup period (days)">
          <input type="number" min={0} value={s.cleanupPeriodDays ?? ''} placeholder="—"
            onChange={(e) => upd({ cleanupPeriodDays: e.target.value ? Number(e.target.value) : undefined })}
            className={`${sx.input} w-20`} />
        </Row>
        <Row label="Disable syntax highlighting">
          <Toggle value={s.syntaxHighlightingDisabled ?? false} onChange={(v) => upd({ syntaxHighlightingDisabled: v })} />
        </Row>
      </Block>
    </div>
  )
}
