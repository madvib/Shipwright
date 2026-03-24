import { useState } from 'react'
import { SlidersHorizontal } from 'lucide-react'
import type { HookConfig } from '@ship/ui'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { SectionShell } from './SectionShell'
import { ProviderConfigModal } from '../dialogs/ProviderConfigModal'

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
  opencode: 'OpenCode',
}

const HOOK_PROVIDERS = new Set(['claude', 'gemini'])

export interface ProvidersSectionProps {
  providers: string[]
  model?: string | null
  env?: Record<string, string> | null
  availableModels?: string[] | null
  agentLimits?: { max_turns?: number; max_cost_per_session?: number } | null
  hooks: HookConfig[]
  providerSettings: Record<string, Record<string, unknown>>
  onChangeModel: (model: string | null) => void
  onChangeEnv: (env: Record<string, string>) => void
  onChangeAvailableModels: (models: string[]) => void
  onChangeAgentLimits: (limits: Record<string, unknown>) => void
  onChangeHooks: (hooks: HookConfig[]) => void
  onChangeProviderSettings: (settings: Record<string, Record<string, unknown>>) => void
}

export function ProvidersSection({
  providers,
  model,
  env,
  availableModels,
  agentLimits,
  hooks,
  providerSettings,
  onChangeModel,
  onChangeEnv,
  onChangeAvailableModels,
  onChangeAgentLimits,
  onChangeHooks,
  onChangeProviderSettings,
}: ProvidersSectionProps) {
  const [activeProvider, setActiveProvider] = useState<string | null>(null)

  if (providers.length === 0) return null

  const handleSave = (config: {
    model: string | null
    env: Record<string, string>
    availableModels: string[]
    agentLimits: Record<string, unknown>
    hooks: HookConfig[]
    providerSettings: Record<string, unknown>
  }) => {
    onChangeModel(config.model)
    onChangeEnv(config.env)
    onChangeAvailableModels(config.availableModels)
    onChangeAgentLimits(config.agentLimits)
    onChangeHooks(config.hooks)
    if (activeProvider) {
      onChangeProviderSettings({ ...providerSettings, [activeProvider]: config.providerSettings })
    }
  }

  return (
    <>
      <SectionShell
        icon={<SlidersHorizontal className="size-4" />}
        title="Providers"
        count={`${providers.length} provider${providers.length !== 1 ? 's' : ''}`}
      >
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
          {providers.map((provider) => (
            <ProviderCard
              key={provider}
              provider={provider}
              model={model}
              hookCount={HOOK_PROVIDERS.has(provider) ? hooks.length : 0}
              settingsCount={Object.keys(providerSettings[provider] ?? {}).length}
              onClick={() => setActiveProvider(provider)}
            />
          ))}
        </div>
      </SectionShell>

      {activeProvider && (
        <ProviderConfigModal
          open={activeProvider !== null}
          onOpenChange={(open) => { if (!open) setActiveProvider(null) }}
          provider={activeProvider}
          model={model ?? null}
          env={env ?? {}}
          availableModels={availableModels ?? []}
          agentLimits={agentLimits ?? {}}
          hooks={hooks}
          providerSettings={providerSettings[activeProvider] ?? {}}
          onSave={handleSave}
        />
      )}
    </>
  )
}

function ProviderCard({
  provider,
  model,
  hookCount,
  settingsCount,
  onClick,
}: {
  provider: string
  model?: string | null
  hookCount: number
  settingsCount: number
  onClick: () => void
}) {
  const parts: string[] = []
  if (model) parts.push(`model: ${model}`)
  if (hookCount > 0) parts.push(`${hookCount} hook${hookCount !== 1 ? 's' : ''}`)
  if (settingsCount > 0) parts.push(`${settingsCount} custom setting${settingsCount !== 1 ? 's' : ''}`)
  const summary = parts.length > 0 ? parts.join(', ') : 'defaults'

  return (
    <button
      onClick={onClick}
      className="flex items-center gap-3 rounded-lg border border-border/40 bg-card/30 px-3 py-3 text-left hover:border-border transition-colors"
    >
      <ProviderLogo provider={provider} size="md" />
      <div className="flex-1 min-w-0">
        <div className="text-xs font-medium text-foreground/80">
          {PROVIDER_LABELS[provider] ?? provider}
        </div>
        <div className="text-[10px] text-muted-foreground/50 truncate mt-0.5">
          {summary}
        </div>
      </div>
    </button>
  )
}
