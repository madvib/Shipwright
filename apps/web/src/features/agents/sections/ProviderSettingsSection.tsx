import { useState } from 'react'
import { Settings2, ChevronDown, ChevronRight } from 'lucide-react'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { SectionShell } from './SectionShell'

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
  opencode: 'OpenCode',
}

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

  const updateJson = (provider: string, json: string) => {
    try {
      const parsed = JSON.parse(json) as Record<string, unknown>
      onChange({ ...providerSettings, [provider]: parsed })
    } catch {
      // Invalid JSON — don't update
    }
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
          <ProviderCard
            key={provider}
            provider={provider}
            expanded={expandedProviders.has(provider)}
            onToggle={() => toggleExpanded(provider)}
            value={providerSettings[provider] ?? {}}
            onJsonChange={(json) => updateJson(provider, json)}
          />
        ))}
      </div>
    </SectionShell>
  )
}

function ProviderCard({
  provider,
  expanded,
  onToggle,
  value,
  onJsonChange,
}: {
  provider: string
  expanded: boolean
  onToggle: () => void
  value: Record<string, unknown>
  onJsonChange: (json: string) => void
}) {
  const hasValues = Object.keys(value).length > 0
  const [draft, setDraft] = useState(() => JSON.stringify(value, null, 2))
  const [parseError, setParseError] = useState<string | null>(null)

  const handleBlur = () => {
    try {
      JSON.parse(draft)
      setParseError(null)
      onJsonChange(draft)
    } catch (e) {
      setParseError(e instanceof Error ? e.message : 'Invalid JSON')
    }
  }

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
        {!hasValues && (
          <span className="ml-auto text-[10px] text-muted-foreground/40 italic">
            defaults
          </span>
        )}
      </button>

      {expanded && (
        <div className="border-t border-border/30 px-3 py-2 space-y-1.5">
          <p className="text-[10px] text-muted-foreground/40">
            Provider-specific overrides (JSON). Leave empty for defaults.
          </p>
          <textarea
            value={draft}
            onChange={(e) => { setDraft(e.target.value); setParseError(null) }}
            onBlur={handleBlur}
            spellCheck={false}
            className="w-full rounded-md border border-border/40 bg-background/60 px-2.5 py-2 font-mono text-[11px] text-foreground/80 leading-relaxed resize-y min-h-[80px] outline-none focus:border-primary/50"
            rows={4}
          />
          {parseError && (
            <p className="text-[10px] text-destructive">{parseError}</p>
          )}
        </div>
      )}
    </div>
  )
}
