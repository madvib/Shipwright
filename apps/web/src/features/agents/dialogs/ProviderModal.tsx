import { useState, useEffect, useCallback } from 'react'
import { X, Settings2, Link2, Code2, Plus, Trash2 } from 'lucide-react'
import type { HookConfig } from '@ship/ui'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { HookEditorDialog } from './HookEditorDialog'

// ── Provider field definitions ───────────────────────────────────────────────

interface FieldDef {
  key: string
  label: string
  type: 'string' | 'number' | 'integer' | 'boolean' | 'select' | 'string-array'
  options?: string[]
  min?: number
  placeholder?: string
}

const PROVIDER_FIELD_DEFS: Record<string, FieldDef[]> = {
  claude: [
    { key: 'maxCostPerSession', label: 'Max cost per session ($)', type: 'number', min: 0 },
    { key: 'maxTurns', label: 'Max turns', type: 'integer', min: 1 },
    { key: 'defaultMode', label: 'Default mode', type: 'select', options: ['default', 'plan', 'acceptEdits', 'dontAsk', 'bypassPermissions'] },
    { key: 'additionalDirectories', label: 'Additional directories', type: 'string-array', placeholder: '/path/to/dir' },
    { key: 'contextWindowTokens', label: 'Context window tokens', type: 'integer' },
    { key: 'autoMemoryEnabled', label: 'Auto memory', type: 'boolean' },
    { key: 'theme', label: 'Theme', type: 'select', options: ['dark', 'light', 'light-daltonized', 'dark-daltonized'] },
  ],
  gemini: [
    { key: 'maxSessionTurns', label: 'Max session turns', type: 'integer', min: 1 },
    { key: 'theme', label: 'Theme', type: 'select', options: ['system', 'dark', 'light'] },
    { key: 'yolo', label: 'YOLO mode', type: 'boolean' },
  ],
  codex: [
    { key: 'approval_policy', label: 'Approval policy', type: 'select', options: ['suggest', 'auto-edit', 'full-auto'] },
    { key: 'notify', label: 'Notifications', type: 'boolean' },
    { key: 'disable_response_storage', label: 'Disable response storage', type: 'boolean' },
  ],
  cursor: [],
  opencode: [],
}

const PROVIDER_HOOK_TRIGGERS: Record<string, string[]> = {
  claude: ['PreToolUse', 'PostToolUse', 'Stop', 'Notification', 'SubagentStop', 'PreCompact'],
  gemini: ['BeforeTool', 'AfterTool', 'SessionEnd', 'Notification', 'PreCompress'],
  cursor: ['beforeMCPExecution', 'afterMCPExecution', 'sessionEnd'],
  codex: [],
  opencode: [],
}

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude Code',
  gemini: 'Gemini CLI',
  codex: 'Codex CLI',
  cursor: 'Cursor',
  opencode: 'OpenCode',
}

// ── ProviderModal ────────────────────────────────────────────────────────────

interface ProviderModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  provider: string
  settings: Record<string, unknown>
  hooks: HookConfig[]
  onSettingsChange: (settings: Record<string, unknown>) => void
  onHooksChange: (hooks: HookConfig[]) => void
}

type Tab = 'settings' | 'hooks' | 'json'

export function ProviderModal({
  open,
  onOpenChange,
  provider,
  settings,
  hooks,
  onSettingsChange,
  onHooksChange,
}: ProviderModalProps) {
  const [tab, setTab] = useState<Tab>('settings')
  const [localSettings, setLocalSettings] = useState<Record<string, unknown>>({})
  const [localHooks, setLocalHooks] = useState<HookConfig[]>([])
  const [jsonDraft, setJsonDraft] = useState('')
  const [jsonError, setJsonError] = useState('')

  // Hook editor state
  const [hookOpen, setHookOpen] = useState(false)
  const [hookEdit, setHookEdit] = useState<{ index: number; hook: HookConfig } | null>(null)

  useEffect(() => {
    if (open) {
      setLocalSettings(structuredClone(settings))
      setLocalHooks(structuredClone(hooks))
      setJsonDraft(JSON.stringify(settings, null, 2))
      setJsonError('')
      setTab('settings')
    }
  }, [open, settings, hooks])

  const close = useCallback(() => onOpenChange(false), [onOpenChange])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') close() }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [open, close])

  const handleSave = () => {
    if (tab === 'json') {
      // Parse JSON tab and apply
      try {
        const parsed = JSON.parse(jsonDraft)
        onSettingsChange(parsed)
      } catch {
        setJsonError('Invalid JSON')
        return
      }
    } else {
      onSettingsChange(localSettings)
    }
    onHooksChange(localHooks)
    close()
  }

  const updateField = (key: string, value: unknown) => {
    setLocalSettings((prev) => {
      const next = { ...prev }
      if (value === '' || value === undefined || value === null) {
        delete next[key]
      } else {
        next[key] = value
      }
      return next
    })
  }

  if (!open) return null

  const fields = PROVIDER_FIELD_DEFS[provider] ?? []
  const triggers = PROVIDER_HOOK_TRIGGERS[provider] ?? []
  const hasTypedFields = fields.length > 0
  const hasHooks = triggers.length > 0

  const tabs: { id: Tab; label: string; icon: React.ReactNode }[] = [
    { id: 'settings', label: 'Settings', icon: <Settings2 className="size-3.5" /> },
    { id: 'hooks', label: 'Hooks', icon: <Link2 className="size-3.5" /> },
    { id: 'json', label: 'Raw JSON', icon: <Code2 className="size-3.5" /> },
  ]

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={close} />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          className="w-full max-w-xl rounded-xl border border-border/60 bg-card shadow-2xl flex flex-col max-h-[80vh]"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2.5">
              <ProviderLogo provider={provider} size="sm" />
              <h2 className="font-display text-sm font-semibold text-foreground">
                {PROVIDER_LABELS[provider] ?? provider} Settings
              </h2>
            </div>
            <button onClick={close} aria-label="Close" className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition">
              <X className="size-4" />
            </button>
          </div>

          {/* Tab bar */}
          <div className="flex border-b border-border/40 px-5">
            {tabs.map((t) => (
              <button
                key={t.id}
                onClick={() => setTab(t.id)}
                className={`flex items-center gap-1.5 px-3 py-2.5 text-[11px] font-medium border-b-2 transition-colors -mb-px ${
                  tab === t.id
                    ? 'border-primary text-primary'
                    : 'border-transparent text-muted-foreground/60 hover:text-muted-foreground'
                }`}
              >
                {t.icon}
                {t.label}
              </button>
            ))}
          </div>

          {/* Body */}
          <div className="flex-1 overflow-y-auto px-5 py-4">
            {tab === 'settings' && (
              hasTypedFields ? (
                <div className="space-y-4">
                  {fields.map((field) => (
                    <FieldRenderer
                      key={field.key}
                      field={field}
                      value={localSettings[field.key]}
                      onChange={(v) => updateField(field.key, v)}
                    />
                  ))}
                </div>
              ) : (
                <div className="py-8 text-center text-xs text-muted-foreground/50">
                  No typed settings available for {PROVIDER_LABELS[provider] ?? provider}. Use the Raw JSON tab for configuration.
                </div>
              )
            )}

            {tab === 'hooks' && (
              hasHooks ? (
                <div className="space-y-2">
                  {localHooks.length === 0 && (
                    <p className="text-xs text-muted-foreground/50 py-4 text-center">No hooks configured.</p>
                  )}
                  {localHooks.map((hook, i) => (
                    <div key={i} className="flex items-center gap-2.5 rounded-lg border border-border/40 bg-card/30 px-3 py-2.5">
                      <span className="shrink-0 rounded bg-muted px-2 py-0.5 font-mono text-[11px] text-blue-400">
                        {hook.trigger}
                      </span>
                      <span className="flex-1 truncate font-mono text-[11px] text-muted-foreground/60">
                        {hook.command}
                      </span>
                      <button
                        onClick={() => { setHookEdit({ index: i, hook }); setHookOpen(true) }}
                        className="shrink-0 text-[11px] text-muted-foreground/30 hover:text-primary transition-colors"
                      >
                        edit
                      </button>
                      <button
                        onClick={() => setLocalHooks((prev) => prev.filter((_, j) => j !== i))}
                        className="shrink-0 text-muted-foreground/30 hover:text-destructive transition-colors"
                      >
                        <Trash2 className="size-3" />
                      </button>
                    </div>
                  ))}
                  <button
                    onClick={() => { setHookEdit(null); setHookOpen(true) }}
                    className="inline-flex items-center gap-1.5 rounded-lg border border-dashed border-border/40 px-3 py-2 text-[11px] text-muted-foreground/50 hover:border-border hover:text-muted-foreground transition-colors"
                  >
                    <Plus className="size-3" /> Add hook
                  </button>
                </div>
              ) : (
                <div className="py-8 text-center text-xs text-muted-foreground/50">
                  {PROVIDER_LABELS[provider] ?? provider} does not support hooks.
                </div>
              )
            )}

            {tab === 'json' && (
              <div className="space-y-2">
                <p className="text-[10px] text-muted-foreground/50">
                  Raw JSON pass-through. Ship-managed keys (model, permissions) are set via Ship's own fields.
                </p>
                <textarea
                  value={jsonDraft}
                  onChange={(e) => { setJsonDraft(e.target.value); setJsonError('') }}
                  rows={12}
                  className="w-full rounded-lg border border-border/60 bg-background/60 px-3 py-2 font-mono text-[11px] text-foreground outline-none focus:border-primary/50 resize-y"
                />
                {jsonError && <p className="text-[10px] text-destructive">{jsonError}</p>}
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-2 border-t border-border/60 px-5 py-3.5">
            <button
              onClick={close}
              className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90"
            >
              Save
            </button>
          </div>
        </div>
      </div>

      {/* Hook editor sub-dialog */}
      <HookEditorDialog
        open={hookOpen}
        onOpenChange={setHookOpen}
        hook={hookEdit?.hook ?? null}
        triggers={triggers}
        onSave={(hook) => {
          if (hookEdit) {
            setLocalHooks((prev) => prev.map((h, i) => i === hookEdit.index ? hook : h))
          } else {
            setLocalHooks((prev) => [...prev, hook])
          }
        }}
        onDelete={hookEdit ? () => {
          setLocalHooks((prev) => prev.filter((_, i) => i !== hookEdit.index))
        } : undefined}
      />
    </>
  )
}

// ── Field Renderer ───────────────────────────────────────────────────────────

function FieldRenderer({
  field,
  value,
  onChange,
}: {
  field: FieldDef
  value: unknown
  onChange: (value: unknown) => void
}) {
  const inputCls = 'w-full rounded-lg border border-border/60 bg-background px-3 py-2 text-sm text-foreground outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/25 transition'

  switch (field.type) {
    case 'string':
      return (
        <div className="space-y-1.5">
          <label className="text-xs font-medium text-muted-foreground">{field.label}</label>
          <input
            type="text"
            value={(value as string) ?? ''}
            onChange={(e) => onChange(e.target.value)}
            placeholder={field.placeholder}
            className={inputCls}
          />
        </div>
      )

    case 'number':
    case 'integer':
      return (
        <div className="space-y-1.5">
          <label className="text-xs font-medium text-muted-foreground">{field.label}</label>
          <input
            type="number"
            value={value !== undefined && value !== null ? String(value) : ''}
            onChange={(e) => {
              const v = e.target.value
              if (v === '') { onChange(undefined); return }
              onChange(field.type === 'integer' ? parseInt(v, 10) : parseFloat(v))
            }}
            min={field.min}
            step={field.type === 'integer' ? 1 : undefined}
            className={inputCls}
          />
        </div>
      )

    case 'boolean':
      return (
        <label className="flex items-center gap-2 cursor-pointer select-none py-1">
          <input
            type="checkbox"
            checked={!!value}
            onChange={(e) => onChange(e.target.checked || undefined)}
            className="size-3.5 rounded border-border/60 accent-primary"
          />
          <span className="text-xs font-medium text-muted-foreground">{field.label}</span>
        </label>
      )

    case 'select':
      return (
        <div className="space-y-1.5">
          <label className="text-xs font-medium text-muted-foreground">{field.label}</label>
          <select
            value={(value as string) ?? ''}
            onChange={(e) => onChange(e.target.value || undefined)}
            className={inputCls}
          >
            <option value="">—</option>
            {(field.options ?? []).map((opt) => (
              <option key={opt} value={opt}>{opt}</option>
            ))}
          </select>
        </div>
      )

    case 'string-array': {
      const items = Array.isArray(value) ? (value as string[]) : []
      return (
        <div className="space-y-1.5">
          <label className="text-xs font-medium text-muted-foreground">{field.label}</label>
          {items.map((item, i) => (
            <div key={i} className="flex items-center gap-1.5">
              <input
                type="text"
                value={item}
                onChange={(e) => {
                  const next = [...items]
                  next[i] = e.target.value
                  onChange(next)
                }}
                className={inputCls}
              />
              <button
                onClick={() => onChange(items.filter((_, j) => j !== i))}
                className="shrink-0 rounded p-1 text-muted-foreground/30 hover:text-destructive transition"
              >
                <X className="size-3" />
              </button>
            </div>
          ))}
          <button
            onClick={() => onChange([...items, ''])}
            className="text-[11px] text-primary hover:text-primary/80 transition"
          >
            + Add
          </button>
        </div>
      )
    }
  }
}
