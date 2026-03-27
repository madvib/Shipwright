import { useState } from 'react'

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
  opencode: 'OpenCode',
}

interface ProviderPassthroughTabProps {
  provider: string
  settings: Record<string, unknown>
  onChange: (settings: Record<string, unknown>) => void
}

export function ProviderPassthroughTab({ provider, settings, onChange }: ProviderPassthroughTabProps) {
  const [draft, setDraft] = useState(() => JSON.stringify(settings, null, 2))
  const [parseError, setParseError] = useState<string | null>(null)

  const handleBlur = () => {
    try {
      const parsed = JSON.parse(draft) as Record<string, unknown>
      setParseError(null)
      onChange(parsed)
    } catch (e) {
      setParseError(e instanceof Error ? e.message : 'Invalid JSON')
    }
  }

  const label = PROVIDER_LABELS[provider] ?? provider

  return (
    <div className="space-y-3">
      <p className="text-xs text-muted-foreground/60">
        Provider-specific settings passed directly to {label}&apos;s config.
        Ship-managed fields (model, permissions, hooks, env, mcp) are set in the General tab.
      </p>
      <textarea
        value={draft}
        onChange={(e) => { setDraft(e.target.value); setParseError(null) }}
        onBlur={handleBlur}
        spellCheck={false}
        className="w-full rounded-md border border-border/40 bg-background/60 px-2.5 py-2 font-mono text-[11px] text-foreground/80 leading-relaxed resize-y min-h-[200px] outline-none focus:border-primary/50"
        rows={10}
      />
      {parseError && (
        <p className="text-[10px] text-destructive">{parseError}</p>
      )}
    </div>
  )
}
