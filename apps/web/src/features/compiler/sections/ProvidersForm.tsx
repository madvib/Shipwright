import { PROVIDERS } from '../types'
import { ProviderLogo } from '../ProviderLogo'

interface Props {
  selected: string[]
  onToggle: (id: string) => void
}

export function ProvidersForm({ selected, onToggle }: Props) {
  return (
    <div className="space-y-2">
      {PROVIDERS.map((p) => {
        const checked = selected.includes(p.id)
        return (
          <label
            key={p.id}
            className={`flex cursor-pointer items-start gap-3 rounded-xl border p-3.5 transition ${
              checked
                ? 'border-primary/30 bg-primary/5'
                : 'border-border/60 bg-card/50 hover:border-border hover:bg-card'
            }`}
          >
            <input
              type="checkbox"
              checked={checked}
              onChange={() => onToggle(p.id)}
              className="mt-1 accent-primary"
            />
            <div className="flex min-w-0 flex-1 items-center gap-3">
              <div className="flex size-8 shrink-0 items-center justify-center rounded-lg border border-border bg-background p-1.5">
                <ProviderLogo provider={p.id} size="md" />
              </div>
              <div className="min-w-0">
                <p className="text-sm font-semibold leading-none">{p.name}</p>
                <p className="mt-0.5 text-[11px] text-muted-foreground">{p.description}</p>
              </div>
            </div>
            <div className="flex shrink-0 flex-col gap-0.5 pt-0.5">
              {p.files.map((f) => (
                <span key={f} className="block text-right font-mono text-[9px] text-muted-foreground/60">{f}</span>
              ))}
            </div>
          </label>
        )
      })}
    </div>
  )
}
