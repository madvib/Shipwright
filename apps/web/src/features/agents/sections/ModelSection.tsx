import { Cpu } from 'lucide-react'
import { SectionShell } from './SectionShell'

interface ModelSectionProps {
  model: string
  onChange: (model: string) => void
}

export function ModelSection({ model, onChange }: ModelSectionProps) {
  return (
    <SectionShell
      icon={<Cpu className="size-4" />}
      title="Model"
    >
      <input
        type="text"
        value={model}
        onChange={(e) => onChange(e.target.value)}
        placeholder="e.g. claude-sonnet-4-20250514, gemini-2.5-pro"
        className="w-full rounded-lg border border-border/60 bg-background px-3 py-2 font-mono text-sm text-foreground placeholder:text-muted-foreground/40 outline-none focus:border-primary/50 focus:ring-1 focus:ring-primary/25 transition"
      />
      <p className="text-[10px] text-muted-foreground/50 mt-1.5">
        Model identifier passed to all target providers. Leave empty for provider default.
      </p>
    </SectionShell>
  )
}
