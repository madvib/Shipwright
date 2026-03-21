import { Settings2, ChevronDown, ChevronRight } from 'lucide-react'
import { useState } from 'react'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { SectionShell } from './SectionShell'

// ── Provider-specific field definitions ─────────────────────────────────────

const CLAUDE_FIELDS = [
  { key: 'include_co_authored_by', label: 'Include co-authored-by', type: 'toggle' as const },
]

const GEMINI_FIELDS = [
  {
    key: 'default_approval_mode',
    label: 'Default approval mode',
    type: 'select' as const,
    options: ['ask-every-time', 'auto-approve-reads', 'auto-approve-all'],
  },
  { key: 'max_session_turns', label: 'Max session turns', type: 'number' as const },
]

const CODEX_FIELDS = [
  {
    key: 'approval_policy',
    label: 'Approval policy',
    type: 'select' as const,
    options: ['ask-every-time', 'unless-allow-listed', 'auto-approve'],
  },
  {
    key: 'sandbox',
    label: 'Sandbox',
    type: 'select' as const,
    options: ['docker', 'none'],
  },
]

const PROVIDER_FIELDS: Record<string, { key: string; label: string; type: 'toggle' | 'select' | 'number'; options?: string[] }[]> = {
  claude: CLAUDE_FIELDS,
  gemini: GEMINI_FIELDS,
  codex: CODEX_FIELDS,
}

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
}

// ── Component ───────────────────────────────────────────────────────────────

interface ProviderSettingsSectionProps {
  providers: string[]
  providerSettings: Record<string, Record<string, unknown>>
  onChange: (settings: Record<string, Record<string, unknown>>) => void
}

export function ProviderSettingsSection({
  providers,
  providerSettings,
  onChange,
}: ProviderSettingsSectionProps) {
  const [expandedProviders, setExpandedProviders] = useState<Set<string>>(
    () => new Set(providers),
  )

  const toggleExpanded = (provider: string) => {
    setExpandedProviders((prev) => {
      const next = new Set(prev)
      if (next.has(provider)) next.delete(provider)
      else next.add(provider)
      return next
    })
  }

  const updateField = (provider: string, key: string, value: unknown) => {
    const current = providerSettings[provider] ?? {}
    onChange({
      ...providerSettings,
      [provider]: { ...current, [key]: value },
    })
  }

  if (providers.length === 0) return null

  return (
    <SectionShell
      icon={<Settings2 className="size-4" />}
      title="Provider Settings"
      count={`${providers.length} provider${providers.length !== 1 ? 's' : ''}`}
    >
      <div className="space-y-2">
        {providers.map((provider) => (
          <ProviderGroup
            key={provider}
            provider={provider}
            expanded={expandedProviders.has(provider)}
            onToggle={() => toggleExpanded(provider)}
            values={providerSettings[provider] ?? {}}
            onUpdateField={(key, value) => updateField(provider, key, value)}
          />
        ))}
      </div>
    </SectionShell>
  )
}

// ── Provider group ──────────────────────────────────────────────────────────

function ProviderGroup({
  provider,
  expanded,
  onToggle,
  values,
  onUpdateField,
}: {
  provider: string
  expanded: boolean
  onToggle: () => void
  values: Record<string, unknown>
  onUpdateField: (key: string, value: unknown) => void
}) {
  const fields = PROVIDER_FIELDS[provider]
  const hasFields = fields && fields.length > 0

  return (
    <div className="rounded-lg border border-border/40 bg-card/30 overflow-hidden">
      <button
        onClick={onToggle}
        className="flex w-full items-center gap-2 px-3 py-2.5 text-left hover:bg-muted/30 transition-colors"
      >
        {expanded ? (
          <ChevronDown className="size-3 text-muted-foreground/50 shrink-0" />
        ) : (
          <ChevronRight className="size-3 text-muted-foreground/50 shrink-0" />
        )}
        <ProviderLogo provider={provider} size="sm" />
        <span className="text-xs font-medium text-foreground/80">
          {PROVIDER_LABELS[provider] ?? provider}
        </span>
        {!hasFields && (
          <span className="ml-auto text-[10px] text-muted-foreground/40 italic">
            no settings available
          </span>
        )}
      </button>

      {expanded && hasFields && (
        <div className="border-t border-border/30 px-3 py-2 space-y-2">
          {fields.map((field) => (
            <ProviderField
              key={field.key}
              field={field}
              value={values[field.key]}
              onChange={(v) => onUpdateField(field.key, v)}
            />
          ))}
        </div>
      )}

      {expanded && !hasFields && (
        <div className="border-t border-border/30 px-3 py-2">
          <p className="text-[11px] text-muted-foreground/40 italic">
            Provider targeted for compilation. No additional settings.
          </p>
        </div>
      )}
    </div>
  )
}

// ── Individual field renderers ──────────────────────────────────────────────

function ProviderField({
  field,
  value,
  onChange,
}: {
  field: { key: string; label: string; type: 'toggle' | 'select' | 'number'; options?: string[] }
  value: unknown
  onChange: (v: unknown) => void
}) {
  if (field.type === 'toggle') {
    const checked = value === true
    return (
      <div className="flex items-center justify-between">
        <span className="text-[11px] text-foreground/70">{field.label}</span>
        <button
          onClick={() => onChange(!checked)}
          className={`relative h-4 w-8 shrink-0 rounded-full transition-colors ${
            checked ? 'bg-primary' : 'bg-muted'
          }`}
        >
          <span
            className={`absolute top-[2px] size-3 rounded-full bg-white transition-all ${
              checked ? 'left-[18px]' : 'left-[2px]'
            }`}
          />
        </button>
      </div>
    )
  }

  if (field.type === 'select' && field.options) {
    const current = (value as string) ?? field.options[0]
    return (
      <div className="flex items-center justify-between">
        <span className="text-[11px] text-foreground/70">{field.label}</span>
        <select
          value={current}
          onChange={(e) => onChange(e.target.value)}
          className="rounded-md border border-border/40 bg-card/50 px-2 py-1 text-[11px] text-foreground/80 outline-none focus:border-primary"
        >
          {field.options.map((opt) => (
            <option key={opt} value={opt}>{opt}</option>
          ))}
        </select>
      </div>
    )
  }

  if (field.type === 'number') {
    const numValue = typeof value === 'number' ? value : ''
    return (
      <div className="flex items-center justify-between">
        <span className="text-[11px] text-foreground/70">{field.label}</span>
        <input
          type="number"
          min={1}
          value={numValue}
          onChange={(e) => {
            const v = e.target.value
            onChange(v === '' ? undefined : Number(v))
          }}
          placeholder="unlimited"
          className="w-24 rounded-md border border-border/40 bg-card/50 px-2 py-1 text-[11px] text-foreground/80 outline-none focus:border-primary text-right"
        />
      </div>
    )
  }

  return null
}
