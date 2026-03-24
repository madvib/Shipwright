import { X } from 'lucide-react'
import { Input } from '@ship/primitives'
import { EnvVarEditor } from './EnvVarEditor'

interface ProviderGeneralTabProps {
  model: string
  onModelChange: (model: string) => void
  availableModels: string[]
  onAvailableModelsChange: (models: string[]) => void
  agentLimits: { max_turns?: number; max_cost_per_session?: number }
  onAgentLimitsChange: (limits: { max_turns?: number; max_cost_per_session?: number }) => void
  env: Record<string, string>
  onEnvChange: (env: Record<string, string>) => void
}

export function ProviderGeneralTab({
  model,
  onModelChange,
  availableModels,
  onAvailableModelsChange,
  agentLimits,
  onAgentLimitsChange,
  env,
  onEnvChange,
}: ProviderGeneralTabProps) {
  const envEntries = Object.entries(env).map(([key, value]) => ({ key, value }))
  const handleEnvChange = (entries: { key: string; value: string }[]) => {
    const record: Record<string, string> = {}
    for (const e of entries) {
      if (e.key.trim()) record[e.key.trim()] = e.value
    }
    onEnvChange(record)
  }

  const addModel = (value: string) => {
    const trimmed = value.trim()
    if (trimmed && !availableModels.includes(trimmed)) {
      onAvailableModelsChange([...availableModels, trimmed])
    }
  }

  const removeModel = (index: number) => {
    onAvailableModelsChange(availableModels.filter((_, i) => i !== index))
  }

  return (
    <div className="space-y-5">
      {/* Model */}
      <div className="space-y-1.5">
        <label className="text-xs font-medium text-foreground">Model override</label>
        <Input
          value={model}
          onChange={(e) => onModelChange(e.target.value)}
          placeholder="Leave empty for provider default"
          className="font-mono text-xs"
        />
        <p className="text-[10px] text-muted-foreground/50">
          Overrides the default model for this provider.
        </p>
      </div>

      {/* Available models */}
      <div className="space-y-1.5">
        <label className="text-xs font-medium text-foreground">Available models</label>
        {availableModels.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {availableModels.map((m, i) => (
              <span key={`${m}-${i}`} className="inline-flex items-center gap-1 rounded-md border border-border/40 bg-muted px-1.5 py-0.5 text-[11px] font-mono text-muted-foreground">
                {m}
                <button type="button" onClick={() => removeModel(i)} className="opacity-60 hover:opacity-100 transition-opacity">
                  <X className="size-2.5" />
                </button>
              </span>
            ))}
          </div>
        )}
        <Input
          placeholder="Add model and press Enter..."
          className="font-mono text-xs"
          onKeyDown={(e) => {
            if (e.key !== 'Enter') return
            e.preventDefault()
            addModel(e.currentTarget.value)
            e.currentTarget.value = ''
          }}
        />
      </div>

      {/* Agent limits */}
      <div className="space-y-3">
        <label className="text-xs font-medium text-foreground">Agent limits</label>
        <div className="grid grid-cols-2 gap-3">
          <div className="space-y-1">
            <label className="text-[11px] text-muted-foreground">Max turns</label>
            <Input
              type="number"
              value={agentLimits.max_turns ?? ''}
              onChange={(e) => onAgentLimitsChange({
                ...agentLimits,
                max_turns: e.target.value ? Number(e.target.value) : undefined,
              })}
              placeholder="No limit"
              className="text-xs"
            />
          </div>
          <div className="space-y-1">
            <label className="text-[11px] text-muted-foreground">Max cost per session</label>
            <div className="relative">
              <span className="absolute left-2.5 top-1/2 -translate-y-1/2 text-xs text-muted-foreground/50">$</span>
              <Input
                type="number"
                step="0.01"
                value={agentLimits.max_cost_per_session ?? ''}
                onChange={(e) => onAgentLimitsChange({
                  ...agentLimits,
                  max_cost_per_session: e.target.value ? Number(e.target.value) : undefined,
                })}
                placeholder="No limit"
                className="pl-6 text-xs"
              />
            </div>
          </div>
        </div>
      </div>

      {/* Environment variables */}
      <EnvVarEditor entries={envEntries} onChange={handleEnvChange} />
    </div>
  )
}
