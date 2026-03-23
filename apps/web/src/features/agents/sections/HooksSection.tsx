import { Link2 } from 'lucide-react'
import type { HookConfig } from '@ship/ui'
import { SectionShell } from './SectionShell'

interface HooksSectionProps {
  hooks: HookConfig[]
  onAdd?: () => void
  onEdit?: (index: number) => void
  onRemove?: (index: number) => void
}

export function HooksSection({ hooks, onAdd, onEdit, onRemove }: HooksSectionProps) {
  return (
    <SectionShell
      icon={<Link2 className="size-4" />}
      title="Hooks"
      count={`${hooks.length} configured`}
      actionLabel="Add"
      onAction={onAdd}
    >
      <div className="flex flex-col gap-1.5">
        {hooks.map((hook, i) => (
          <div
            key={i}
            onClick={() => onEdit?.(i)}
            className={`flex items-center gap-2 rounded-lg border border-border/40 bg-card/30 px-3 py-2 ${onEdit ? 'cursor-pointer hover:border-border transition-colors' : ''}`}
          >
            <span className="shrink-0 rounded bg-primary/10 px-2 py-0.5 text-[10px] font-semibold text-primary">
              {hook.trigger}
            </span>
            <span className="flex-1 truncate font-mono text-[11px] text-muted-foreground/60">
              {hook.command}
            </span>
            {hook.matcher && (
              <span className="shrink-0 rounded bg-muted px-1.5 py-0.5 text-[9px] font-mono text-muted-foreground/50">
                {hook.matcher}
              </span>
            )}
            {onRemove && (
              <button
                onClick={(e) => { e.stopPropagation(); onRemove(i) }}
                className="shrink-0 text-muted-foreground/30 hover:text-destructive transition-colors text-sm"
              >
                x
              </button>
            )}
          </div>
        ))}
      </div>
    </SectionShell>
  )
}
